use std::collections::BTreeMap;

use anyhow::Context;
use base64::Engine;
use indexmap::IndexMap;
use openapiv3_1::{Object, Ref, Schema, Type};
use tinc_cel::{CelValue, NumberTy};

use crate::codegen::cel::compiler::{CompiledExpr, Compiler, CompilerTarget, ConstantCompiledExpr};
use crate::codegen::cel::{CelExpression, CelExpressions, functions};
use crate::types::{
    ProtoMessageType, ProtoModifiedValueType, ProtoType, ProtoTypeRegistry, ProtoValueType, ProtoWellKnownType,
};

fn cel_to_json(cel: &CelValue<'static>, type_registry: &ProtoTypeRegistry) -> anyhow::Result<serde_json::Value> {
    match cel {
        CelValue::Null => Ok(serde_json::Value::Null),
        CelValue::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        CelValue::Map(map) => Ok(serde_json::Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if let CelValue::String(key) = key {
                        Ok((key.to_string(), cel_to_json(value, type_registry)?))
                    } else {
                        anyhow::bail!("map keys must be a string")
                    }
                })
                .collect::<anyhow::Result<_>>()?,
        )),
        CelValue::List(list) => Ok(serde_json::Value::Array(
            list.iter()
                .map(|i| cel_to_json(i, type_registry))
                .collect::<anyhow::Result<_>>()?,
        )),
        CelValue::String(s) => Ok(serde_json::Value::String(s.to_string())),
        CelValue::Number(NumberTy::F64(f)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(*f).context("f64 is not a valid float")?,
        )),
        CelValue::Number(NumberTy::I64(i)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_i128(*i as i128).context("i64 is not a valid int")?,
        )),
        CelValue::Number(NumberTy::U64(u)) => Ok(serde_json::Value::Number(
            serde_json::Number::from_u128(*u as u128).context("u64 is not a valid uint")?,
        )),
        CelValue::Duration(duration) => Ok(serde_json::Value::String(duration.to_string())),
        CelValue::Timestamp(timestamp) => Ok(serde_json::Value::String(timestamp.to_rfc3339())),
        CelValue::Bytes(bytes) => Ok(serde_json::Value::String(
            base64::engine::general_purpose::STANDARD.encode(bytes),
        )),
        CelValue::Enum(cel_enum) => {
            let enum_ty = type_registry
                .get_enum(&cel_enum.tag)
                .with_context(|| format!("couldnt find enum {}", cel_enum.tag.as_ref()))?;
            if enum_ty.options.repr_enum {
                Ok(serde_json::Value::from(cel_enum.value))
            } else {
                let variant = enum_ty
                    .variants
                    .values()
                    .find(|v| v.value == cel_enum.value)
                    .with_context(|| format!("{} has no value for {}", cel_enum.tag.as_ref(), cel_enum.value))?;
                Ok(serde_json::Value::from(variant.options.serde_name.clone()))
            }
        }
    }
}
fn parse_resolve(compiler: &Compiler, expr: &str) -> anyhow::Result<CelValue<'static>> {
    let expr = cel_parser::parse(expr).context("parse")?;
    let resolved = compiler.resolve(&expr).context("resolve")?;
    match resolved {
        CompiledExpr::Constant(ConstantCompiledExpr { value }) => Ok(value),
        CompiledExpr::Runtime(_) => anyhow::bail!("expression needs runtime evaluation"),
    }
}

fn handle_expr(mut ctx: Compiler, ty: &ProtoType, expr: &CelExpression) -> anyhow::Result<Vec<Schema>> {
    ctx.set_target(CompilerTarget::Serde);

    if let Some(this) = expr.this.clone() {
        ctx.add_variable("this", CompiledExpr::constant(this));
    }

    match ty {
        ProtoType::Modified(ProtoModifiedValueType::Optional(ProtoValueType::Enum(path)))
        | ProtoType::Value(ProtoValueType::Enum(path)) => {
            ctx.register_function(functions::Enum(Some(path.clone())));
        }
        _ => {}
    }

    let mut schemas = Vec::new();
    for schema in &expr.jsonschemas {
        let value = parse_resolve(&ctx, schema)?;
        let value = cel_to_json(&value, ctx.registry())?;
        if !value.is_null() {
            schemas.push(serde_json::from_value(value).context("bad openapi schema")?);
        }
    }

    Ok(schemas)
}

#[derive(Debug)]
pub(crate) enum ExcludePaths {
    True,
    Child(BTreeMap<String, ExcludePaths>),
}

pub(super) fn exclude_path(paths: &mut BTreeMap<String, ExcludePaths>, path: &str) -> anyhow::Result<()> {
    let mut parts = path.split('.').peekable();
    let first_part = parts.next().expect("parts empty").to_owned();

    // Start with the first part of the path
    let mut current_map = paths.entry(first_part).or_insert(if parts.peek().is_none() {
        ExcludePaths::True
    } else {
        ExcludePaths::Child(BTreeMap::new())
    });

    // Iterate over the remaining parts of the path
    while let Some(part) = parts.next() {
        match current_map {
            ExcludePaths::True => anyhow::bail!("duplicate path: {path}"),
            ExcludePaths::Child(map) => {
                current_map = map.entry(part.to_owned()).or_insert(if parts.peek().is_none() {
                    ExcludePaths::True
                } else {
                    ExcludePaths::Child(BTreeMap::new())
                });
            }
        }
    }

    anyhow::ensure!(matches!(current_map, ExcludePaths::True), "duplicate path: {path}");

    Ok(())
}

pub(super) fn generate_query_parameter(
    type_registry: &ProtoTypeRegistry,
    component_schemas: &mut openapiv3_1::Components,
    ty: &ProtoMessageType,
    exclude_paths: &BTreeMap<String, ExcludePaths>,
) -> anyhow::Result<Vec<openapiv3_1::path::Parameter>> {
    let mut params = Vec::new();

    for (name, field) in &ty.fields {
        let exclude_paths = match exclude_paths.get(name) {
            Some(ExcludePaths::True) => continue,
            Some(ExcludePaths::Child(child)) => Some(child),
            None => None,
        };
        params.push(
            openapiv3_1::path::Parameter::builder()
                .name(field.options.serde_name.clone())
                .required(!field.options.serde_omittable.is_true())
                .explode(true)
                .style(openapiv3_1::path::ParameterStyle::DeepObject)
                .schema(generate_optimized(
                    type_registry,
                    component_schemas,
                    field.ty.clone(),
                    &field.options.cel_exprs,
                    exclude_paths.unwrap_or(&BTreeMap::new()),
                    GenerateDirection::Input,
                    BytesEncoding::Base64,
                )?)
                .parameter_in(openapiv3_1::path::ParameterIn::Query)
                .build(),
        )
    }

    Ok(params)
}

pub(super) fn generate_path_parameter(
    type_registry: &ProtoTypeRegistry,
    component_schemas: &mut openapiv3_1::Components,
    paths: &BTreeMap<String, (ProtoValueType, CelExpressions)>,
) -> anyhow::Result<Vec<openapiv3_1::path::Parameter>> {
    let mut params = Vec::new();

    for (path, (ty, cel)) in paths {
        params.push(
            openapiv3_1::path::Parameter::builder()
                .name(path)
                .required(true)
                .schema(generate_optimized(
                    type_registry,
                    component_schemas,
                    ProtoType::Value(ty.clone()),
                    cel,
                    &BTreeMap::new(),
                    GenerateDirection::Input,
                    BytesEncoding::Base64,
                )?)
                .parameter_in(openapiv3_1::path::ParameterIn::Path)
                .build(),
        )
    }

    Ok(params)
}

#[derive(Debug, Clone, Copy)]
pub(super) enum BytesEncoding {
    Base64,
    Binary,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum GenerateDirection {
    Input,
    Output,
}

pub(super) fn generate_optimized(
    type_registry: &ProtoTypeRegistry,
    components: &mut openapiv3_1::Components,
    ty: ProtoType,
    cel: &CelExpressions,
    exclude_paths: &BTreeMap<String, ExcludePaths>,
    direction: GenerateDirection,
    bytes: BytesEncoding,
) -> anyhow::Result<Schema> {
    let mut schema = generate(type_registry, components, ty, cel, exclude_paths, direction, bytes)?;
    schema.optimize();
    Ok(schema)
}

fn generate(
    type_registry: &ProtoTypeRegistry,
    components: &mut openapiv3_1::Components,
    ty: ProtoType,
    cel: &CelExpressions,
    exclude_paths: &BTreeMap<String, ExcludePaths>,
    direction: GenerateDirection,
    bytes: BytesEncoding,
) -> anyhow::Result<Schema> {
    let mut schemas = Vec::new();

    let compiler = Compiler::new(type_registry);
    if !matches!(ty, ProtoType::Modified(ProtoModifiedValueType::Optional(_))) {
        schemas.reserve(cel.field.len());
        for expr in &cel.field {
            schemas.extend(handle_expr(compiler.child(), &ty, expr)?);
        }
    } else {
        schemas.reserve(1);
    }

    schemas.push(match ty {
        ProtoType::Modified(ProtoModifiedValueType::Map(key, value)) => Schema::object(
            Object::builder()
                .schema_type(Type::Object)
                .property_names(match key {
                    ProtoValueType::String => {
                        let mut schemas = Vec::with_capacity(1 + cel.map_key.len());

                        for expr in &cel.map_key {
                            schemas.extend(handle_expr(compiler.child(), &ProtoType::Value(key.clone()), expr)?);
                        }

                        schemas.push(Schema::object(Object::builder().schema_type(Type::String)));

                        Object::all_ofs(schemas)
                    }
                    ProtoValueType::Int32 | ProtoValueType::Int64 => {
                        Object::builder().schema_type(Type::String).pattern("^-?[0-9]+$").build()
                    }
                    ProtoValueType::UInt32 | ProtoValueType::UInt64 => {
                        Object::builder().schema_type(Type::String).pattern("^[0-9]+$").build()
                    }
                    ProtoValueType::Bool => Object::builder()
                        .schema_type(Type::String)
                        .enum_values(["true", "false"])
                        .build(),
                    _ => Object::builder().schema_type(Type::String).build(),
                })
                .additional_properties({
                    let mut schemas = Vec::with_capacity(1 + cel.map_value.len());
                    for expr in &cel.map_value {
                        schemas.extend(handle_expr(compiler.child(), &ProtoType::Value(value.clone()), expr)?);
                    }

                    schemas.push(generate(
                        type_registry,
                        components,
                        ProtoType::Value(value),
                        &CelExpressions::default(),
                        &BTreeMap::new(),
                        direction,
                        bytes,
                    )?);

                    Object::all_ofs(schemas)
                })
                .build(),
        ),
        ProtoType::Modified(ProtoModifiedValueType::Repeated(item)) => Schema::object(
            Object::builder()
                .schema_type(Type::Array)
                .items(generate(
                    type_registry,
                    components,
                    ProtoType::Value(item),
                    cel,
                    exclude_paths,
                    direction,
                    bytes,
                )?)
                .build(),
        ),
        ProtoType::Modified(ProtoModifiedValueType::OneOf(oneof)) => Schema::object(
            Object::builder()
                .schema_type(Type::Object)
                .title(oneof.full_name.to_string())
                .one_ofs(if let Some(tagged) = oneof.options.tagged {
                    oneof
                        .fields
                        .into_iter()
                        .filter(|(_, field)| match direction {
                            GenerateDirection::Input => field.options.visibility.has_input(),
                            GenerateDirection::Output => field.options.visibility.has_output(),
                        })
                        .map(|(name, field)| {
                            let ty = generate(
                                type_registry,
                                components,
                                ProtoType::Value(field.ty),
                                &field.options.cel_exprs,
                                &BTreeMap::new(),
                                direction,
                                bytes,
                            )?;

                            anyhow::Ok(Schema::object(
                                Object::builder()
                                    .schema_type(Type::Object)
                                    .title(name)
                                    .description(field.comments.to_string())
                                    .properties({
                                        let mut properties = IndexMap::new();
                                        properties.insert(
                                            tagged.tag.clone(),
                                            Schema::object(
                                                Object::builder()
                                                    .schema_type(Type::String)
                                                    .const_value(field.options.serde_name)
                                                    .build(),
                                            ),
                                        );
                                        properties.insert(tagged.content.clone(), ty);
                                        properties
                                    })
                                    .unevaluated_properties(false)
                                    .build(),
                            ))
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?
                } else {
                    oneof
                        .fields
                        .into_iter()
                        .filter(|(_, field)| match direction {
                            GenerateDirection::Input => field.options.visibility.has_input(),
                            GenerateDirection::Output => field.options.visibility.has_output(),
                        })
                        .map(|(name, field)| {
                            let ty = generate(
                                type_registry,
                                components,
                                ProtoType::Value(field.ty),
                                &field.options.cel_exprs,
                                &BTreeMap::new(),
                                direction,
                                bytes,
                            )?;

                            anyhow::Ok(Schema::object(
                                Object::builder()
                                    .schema_type(Type::Object)
                                    .title(name)
                                    .description(field.comments.to_string())
                                    .properties({
                                        let mut properties = IndexMap::new();
                                        properties.insert(field.options.serde_name, ty);
                                        properties
                                    })
                                    .unevaluated_properties(false)
                                    .build(),
                            ))
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?
                })
                .unevaluated_properties(false)
                .build(),
        ),
        ProtoType::Modified(ProtoModifiedValueType::Optional(value)) => Schema::object(
            Object::builder()
                .one_ofs([
                    Schema::object(Object::builder().schema_type(Type::Null).build()),
                    generate(
                        type_registry,
                        components,
                        ProtoType::Value(value),
                        cel,
                        exclude_paths,
                        direction,
                        bytes,
                    )?,
                ])
                .build(),
        ),
        ProtoType::Value(ProtoValueType::Bool) => Schema::object(Object::builder().schema_type(Type::Boolean).build()),
        ProtoType::Value(ProtoValueType::Bytes) => Schema::object(
            Object::builder()
                .schema_type(Type::String)
                .content_encoding(match bytes {
                    BytesEncoding::Base64 => "base64",
                    BytesEncoding::Binary => "binary",
                })
                .build(),
        ),
        ProtoType::Value(ProtoValueType::Double | ProtoValueType::Float) => {
            Schema::object(Object::builder().schema_type(Type::Number).build())
        }
        ProtoType::Value(ProtoValueType::Int32) => Schema::object(Object::int32()),
        ProtoType::Value(ProtoValueType::UInt32) => Schema::object(Object::uint32()),
        ProtoType::Value(ProtoValueType::Int64) => Schema::object(Object::int64()),
        ProtoType::Value(ProtoValueType::UInt64) => Schema::object(Object::uint64()),
        ProtoType::Value(ProtoValueType::String) => Schema::object(Object::builder().schema_type(Type::String).build()),
        ProtoType::Value(ProtoValueType::Enum(enum_path)) => {
            let schema_name = format!("{direction:?}.{enum_path}");
            if !components.schemas.contains_key(enum_path.as_ref()) {
                let ety = type_registry
                    .get_enum(&enum_path)
                    .with_context(|| format!("missing enum: {enum_path}"))?;
                components.add_schema(
                    schema_name.clone(),
                    Schema::object(
                        Object::builder()
                            .schema_type(if ety.options.repr_enum { Type::Integer } else { Type::String })
                            .enum_values(
                                ety.variants
                                    .values()
                                    .filter(|v| match direction {
                                        GenerateDirection::Input => v.options.visibility.has_input(),
                                        GenerateDirection::Output => v.options.visibility.has_output(),
                                    })
                                    .map(|v| {
                                        if ety.options.repr_enum {
                                            serde_json::Value::from(v.value)
                                        } else {
                                            serde_json::Value::from(v.options.serde_name.clone())
                                        }
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .title(enum_path.to_string())
                            .description(ety.comments.to_string())
                            .build(),
                    ),
                );
            }

            Schema::object(Ref::from_schema_name(schema_name))
        }
        ProtoType::Value(ProtoValueType::Message(message_path)) => {
            let message_ty = type_registry
                .get_message(&message_path)
                .with_context(|| format!("missing message: {message_path}"))?;

            let schema_name = format!("{direction:?}.{message_path}");

            if !components.schemas.contains_key(&schema_name) || !exclude_paths.is_empty() {
                if exclude_paths.is_empty() {
                    components.schemas.insert(schema_name.clone(), Schema::Bool(false));
                }
                let mut properties = IndexMap::new();
                let mut required = Vec::new();
                let mut schemas = Vec::with_capacity(1);
                for (name, field) in message_ty.fields.iter().filter(|(_, field)| match direction {
                    GenerateDirection::Input => field.options.visibility.has_input(),
                    GenerateDirection::Output => field.options.visibility.has_output(),
                }) {
                    let exclude_paths = match exclude_paths.get(name) {
                        Some(ExcludePaths::True) => continue,
                        Some(ExcludePaths::Child(child)) => Some(child),
                        None => None,
                    };
                    if !field.options.serde_omittable.is_true() {
                        required.push(field.options.serde_name.clone());
                    }

                    let ty = match (!field.options.nullable || field.options.flatten, &field.ty) {
                        (true, ProtoType::Modified(ProtoModifiedValueType::Optional(ty))) => ProtoType::Value(ty.clone()),
                        _ => field.ty.clone(),
                    };

                    let field_schema = generate(
                        type_registry,
                        components,
                        ty,
                        &field.options.cel_exprs,
                        exclude_paths.unwrap_or(&BTreeMap::new()),
                        direction,
                        bytes,
                    )?;

                    if field.options.flatten {
                        schemas.push(field_schema);
                    } else {
                        let schema = if field.options.nullable
                            && !matches!(&field.ty, ProtoType::Modified(ProtoModifiedValueType::Optional(_)))
                        {
                            Schema::object(
                                Object::builder()
                                    .one_ofs([Object::builder().schema_type(Type::Null).build().into(), field_schema])
                                    .build(),
                            )
                        } else {
                            field_schema
                        };

                        properties.insert(
                            field.options.serde_name.clone(),
                            Schema::object(Object::all_ofs([
                                schema,
                                Schema::object(Object::builder().description(field.comments.to_string()).build()),
                            ])),
                        );
                    }
                }

                schemas.push(Schema::object(
                    Object::builder()
                        .schema_type(Type::Object)
                        .title(message_path.to_string())
                        .description(message_ty.comments.to_string())
                        .properties(properties)
                        .required(required)
                        .unevaluated_properties(false)
                        .build(),
                ));

                if exclude_paths.is_empty() {
                    components.add_schema(schema_name.clone(), Object::all_ofs(schemas).into_optimized());
                    Schema::object(Ref::from_schema_name(schema_name))
                } else {
                    Schema::object(Object::all_ofs(schemas))
                }
            } else {
                Schema::object(Ref::from_schema_name(schema_name))
            }
        }
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Timestamp)) => {
            Schema::object(Object::builder().schema_type(Type::String).format("date-time").build())
        }
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Duration)) => {
            Schema::object(Object::builder().schema_type(Type::String).format("duration").build())
        }
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Empty)) => Schema::object(
            Object::builder()
                .schema_type(Type::Object)
                .unevaluated_properties(false)
                .build(),
        ),
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::ListValue)) => {
            Schema::object(Object::builder().schema_type(Type::Array).build())
        }
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Value)) => Schema::object(
            Object::builder()
                .schema_type(vec![
                    Type::Null,
                    Type::Boolean,
                    Type::Object,
                    Type::Array,
                    Type::Number,
                    Type::String,
                ])
                .build(),
        ),
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Struct)) => {
            Schema::object(Object::builder().schema_type(Type::Object).build())
        }
        ProtoType::Value(ProtoValueType::WellKnown(ProtoWellKnownType::Any)) => Schema::object(
            Object::builder()
                .schema_type(Type::Object)
                .property("@type", Object::builder().schema_type(Type::String))
                .build(),
        ),
    });

    Ok(Schema::object(Object::all_ofs(schemas)))
}
