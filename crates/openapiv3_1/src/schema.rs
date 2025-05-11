//! Implements [OpenAPI Schema Object][schema] types which can be
//! used to define field properties, enum values, array or object types.
//!
//! [schema]: https://spec.openapis.org/oas/latest.html#schema-object
use indexmap::IndexMap;
use ordered_float::OrderedFloat;
use serde_derive::{Deserialize, Serialize};

use super::extensions::Extensions;
use super::security::SecurityScheme;
use super::{RefOr, Response};

/// Create an _`empty`_ [`Schema`] that serializes to _`null`_.
///
/// Can be used in places where an item can be serialized as `null`. This is used with unit type
/// enum variants and tuple unit types.
pub fn empty() -> Schema {
    Schema::object(Object::builder().default(serde_json::Value::Null).build())
}

/// Implements [OpenAPI Components Object][components] which holds supported
/// reusable objects.
///
/// Components can hold either reusable types themselves or references to other reusable
/// types.
///
/// [components]: https://spec.openapis.org/oas/latest.html#components-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Components {
    /// Map of reusable [OpenAPI Schema Object][schema]s.
    ///
    /// [schema]: https://spec.openapis.org/oas/latest.html#schema-object
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(field)]
    pub schemas: IndexMap<String, Schema>,

    /// Map of reusable response name, to [OpenAPI Response Object][response]s or [OpenAPI
    /// Reference][reference]s to [OpenAPI Response Object][response]s.
    ///
    /// [response]: https://spec.openapis.org/oas/latest.html#response-object
    /// [reference]: https://spec.openapis.org/oas/latest.html#reference-object
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(field)]
    pub responses: IndexMap<String, RefOr<Response>>,

    /// Map of reusable [OpenAPI Security Scheme Object][security_scheme]s.
    ///
    /// [security_scheme]: https://spec.openapis.org/oas/latest.html#security-scheme-object
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(field)]
    pub security_schemes: IndexMap<String, SecurityScheme>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", default, flatten)]
    pub extensions: Option<Extensions>,
}

impl Components {
    /// Construct a new [`Components`].
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    /// Add [`SecurityScheme`] to [`Components`].
    ///
    /// Accepts two arguments where first is the name of the [`SecurityScheme`]. This is later when
    /// referenced by [`SecurityRequirement`][requirement]s. Second parameter is the [`SecurityScheme`].
    ///
    /// [requirement]: ../security/struct.SecurityRequirement.html
    pub fn add_security_scheme<N: Into<String>, S: Into<SecurityScheme>>(&mut self, name: N, security_scheme: S) {
        self.security_schemes.insert(name.into(), security_scheme.into());
    }

    /// Add iterator of [`SecurityScheme`]s to [`Components`].
    ///
    /// Accepts two arguments where first is the name of the [`SecurityScheme`]. This is later when
    /// referenced by [`SecurityRequirement`][requirement]s. Second parameter is the [`SecurityScheme`].
    pub fn add_security_schemes_from_iter<N: Into<String>, S: Into<SecurityScheme>>(
        &mut self,
        schemas: impl IntoIterator<Item = (N, S)>,
    ) {
        self.security_schemes
            .extend(schemas.into_iter().map(|(name, item)| (name.into(), item.into())));
    }

    /// Add [`Schema`] to [`Components`].
    ///
    /// Accepts two arguments where first is the name of the [`Schema`]. This is later when
    /// referenced by [`Ref`] [ref_location]s. Second parameter is the [`Schema`].
    pub fn add_schema<N: Into<String>, S: Into<Schema>>(&mut self, name: N, scheme: S) {
        self.schemas.insert(name.into(), scheme.into());
    }

    /// Add iterator of [`Schema`]s to [`Components`].
    ///
    /// Accepts two arguments where first is the name of the [`Schema`]. This is later when
    /// referenced by [`Ref`] [ref_location]s. Second parameter is the [`Schema`].
    ///
    /// [requirement]: ../security/struct.SecurityRequirement.html
    pub fn add_schemas_from_iter<N: Into<String>, S: Into<Schema>>(&mut self, schemas: impl IntoIterator<Item = (N, S)>) {
        self.schemas
            .extend(schemas.into_iter().map(|(name, item)| (name.into(), item.into())));
    }
}

impl<S: components_builder::State> ComponentsBuilder<S> {
    /// Add [`Schema`] to [`Components`].
    ///
    /// Accepts two arguments where first is name of the schema and second is the schema itself.
    pub fn schema(mut self, name: impl Into<String>, schema: impl Into<Schema>) -> Self {
        self.schemas.insert(name.into(), schema.into());
        self
    }

    /// Add [`Schema`]s from iterator.
    ///
    /// # Examples
    /// ```rust
    /// # use openapiv3_1::schema::{Components, Object, Type, Schema};
    /// Components::builder().schemas_from_iter([(
    ///     "Pet",
    ///     Schema::from(
    ///         Object::builder()
    ///             .property(
    ///                 "name",
    ///                 Object::builder().schema_type(Type::String),
    ///             )
    ///             .required(["name"])
    ///     ),
    /// )]);
    /// ```
    pub fn schemas_from_iter<I: IntoIterator<Item = (S2, C)>, C: Into<Schema>, S2: Into<String>>(
        mut self,
        schemas: I,
    ) -> Self {
        self.schemas
            .extend(schemas.into_iter().map(|(name, schema)| (name.into(), schema.into())));

        self
    }

    /// Add [`struct@Response`] to [`Components`].
    ///
    /// Method accepts tow arguments; `name` of the reusable response and `response` which is the
    /// reusable response itself.
    pub fn response<S2: Into<String>, R: Into<RefOr<Response>>>(mut self, name: S2, response: R) -> Self {
        self.responses.insert(name.into(), response.into());
        self
    }

    /// Add multiple [`struct@Response`]s to [`Components`] from iterator.
    ///
    /// Like the [`ComponentsBuilder::schemas_from_iter`] this allows adding multiple responses by
    /// any iterator what returns tuples of (name, response) values.
    pub fn responses_from_iter<I: IntoIterator<Item = (S2, R)>, S2: Into<String>, R: Into<RefOr<Response>>>(
        mut self,
        responses: I,
    ) -> Self {
        self.responses
            .extend(responses.into_iter().map(|(name, response)| (name.into(), response.into())));

        self
    }

    /// Add [`SecurityScheme`] to [`Components`].
    ///
    /// Accepts two arguments where first is the name of the [`SecurityScheme`]. This is later when
    /// referenced by [`SecurityRequirement`][requirement]s. Second parameter is the [`SecurityScheme`].
    ///
    /// [requirement]: ../security/struct.SecurityRequirement.html
    pub fn security_scheme<N: Into<String>, S2: Into<SecurityScheme>>(mut self, name: N, security_scheme: S2) -> Self {
        self.security_schemes.insert(name.into(), security_scheme.into());

        self
    }
}

impl<S: components_builder::IsComplete> From<ComponentsBuilder<S>> for Components {
    fn from(value: ComponentsBuilder<S>) -> Self {
        value.build()
    }
}

impl Default for Schema {
    fn default() -> Self {
        Schema::Bool(true)
    }
}

/// OpenAPI [Discriminator][discriminator] object which can be optionally used together with
/// [`Object`] composite object.
///
/// [discriminator]: https://spec.openapis.org/oas/latest.html#discriminator-object
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Discriminator {
    /// Defines a discriminator property name which must be found within all composite
    /// objects.
    pub property_name: String,

    /// An object to hold mappings between payload values and schema names or references.
    /// This field can only be populated manually. There is no macro support and no
    /// validation.
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    pub mapping: IndexMap<String, String>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl Discriminator {
    /// Construct a new [`Discriminator`] object with property name.
    ///
    /// # Examples
    ///
    /// Create a new [`Discriminator`] object for `pet_type` property.
    /// ```rust
    /// # use openapiv3_1::schema::Discriminator;
    /// let discriminator = Discriminator::new("pet_type");
    /// ```
    pub fn new<I: Into<String>>(property_name: I) -> Self {
        Self {
            property_name: property_name.into(),
            mapping: IndexMap::new(),
            ..Default::default()
        }
    }

    /// Construct a new [`Discriminator`] object with property name and mappings.
    ///
    ///
    /// Method accepts two arguments. First _`property_name`_ to use as `discriminator` and
    /// _`mapping`_ for custom property name mappings.
    ///
    /// # Examples
    ///
    /// _**Construct an ew [`Discriminator`] with custom mapping.**_
    ///
    /// ```rust
    /// # use openapiv3_1::schema::Discriminator;
    /// let discriminator = Discriminator::with_mapping("pet_type", [
    ///     ("cat","#/components/schemas/Cat")
    /// ]);
    /// ```
    pub fn with_mapping<P: Into<String>, M: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<String>>(
        property_name: P,
        mapping: M,
    ) -> Self {
        Self {
            property_name: property_name.into(),
            mapping: IndexMap::from_iter(mapping.into_iter().map(|(key, val)| (key.into(), val.into()))),
            ..Default::default()
        }
    }
}

/// Implements [OpenAPI Reference Object][reference] that can be used to reference
/// reusable components such as [`Schema`]s or [`Response`]s.
///
/// [reference]: https://spec.openapis.org/oas/latest.html#reference-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[builder(on(_, into))]
pub struct Ref {
    /// Reference location of the actual component.
    #[serde(rename = "$ref")]
    pub ref_location: String,

    /// A description which by default should override that of the referenced component.
    /// Description supports markdown syntax. If referenced object type does not support
    /// description this field does not have effect.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default)]
    pub description: String,

    /// A short summary which by default should override that of the referenced component. If
    /// referenced component does not support summary field this does not have effect.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default)]
    pub summary: String,
}

impl Ref {
    /// Construct a new [`Ref`] with custom ref location. In most cases this is not necessary
    /// and [`Ref::from_schema_name`] could be used instead.
    pub fn new<I: Into<String>>(ref_location: I) -> Self {
        Self {
            ref_location: ref_location.into(),
            ..Default::default()
        }
    }

    /// Construct a new [`Ref`] from provided schema name. This will create a [`Ref`] that
    /// references the the reusable schemas.
    pub fn from_schema_name<I: Into<String>>(schema_name: I) -> Self {
        Self::new(format!("#/components/schemas/{}", schema_name.into()))
    }

    /// Construct a new [`Ref`] from provided response name. This will create a [`Ref`] that
    /// references the reusable response.
    pub fn from_response_name<I: Into<String>>(response_name: I) -> Self {
        Self::new(format!("#/components/responses/{}", response_name.into()))
    }
}

impl<S: ref_builder::IsComplete> From<RefBuilder<S>> for Schema {
    fn from(builder: RefBuilder<S>) -> Self {
        Self::from(builder.build())
    }
}

impl From<Ref> for Schema {
    fn from(r: Ref) -> Self {
        Self::object(
            Object::builder()
                .reference(r.ref_location)
                .description(r.description)
                .summary(r.summary)
                .build(),
        )
    }
}

impl<T> From<T> for RefOr<T> {
    fn from(t: T) -> Self {
        Self::T(t)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Copy)]
#[non_exhaustive]
pub enum Type {
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "string")]
    String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Types {
    Single(Type),
    Multi(Vec<Type>),
}

impl From<Type> for Types {
    fn from(value: Type) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<Type>> for Types {
    fn from(mut value: Vec<Type>) -> Self {
        if value.len() == 1 {
            Self::Single(value.remove(0))
        } else {
            Self::Multi(value)
        }
    }
}

trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl<T> IsEmpty for Option<T> {
    fn is_empty(&self) -> bool {
        self.is_none()
    }
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}

impl<K, V> IsEmpty for IndexMap<K, V> {
    fn is_empty(&self) -> bool {
        IndexMap::is_empty(self)
    }
}

impl IsEmpty for String {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Clone, PartialEq, Default, bon::Builder)]
#[serde(default, deny_unknown_fields)]
#[builder(on(_, into))]
#[non_exhaustive]
pub struct Object {
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(field)]
    pub properties: IndexMap<String, Schema>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(field)]
    pub examples: Vec<serde_json::Value>,
    #[serde(rename = "prefixItems", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(field)]
    pub prefix_items: Option<Vec<Schema>>,
    #[serde(rename = "enum", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(field)]
    pub enum_values: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(field)]
    pub required: Vec<String>,
    #[serde(rename = "allOf", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(field)]
    pub all_of: Vec<Schema>,
    #[serde(rename = "anyOf", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(field)]
    pub any_of: Option<Vec<Schema>>,
    #[serde(rename = "oneOf", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(field)]
    pub one_of: Option<Vec<Schema>>,
    #[serde(rename = "$id", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub id: String,
    #[serde(rename = "$schema", skip_serializing_if = "IsEmpty::is_empty")]
    pub schema: Option<Schema>,
    #[serde(rename = "$ref", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default, name = "reference")]
    pub reference: String,
    #[serde(rename = "$comment", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub comment: String,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub title: String,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub description: String,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub summary: String,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub default: Option<serde_json::Value>,
    #[serde(rename = "readOnly", skip_serializing_if = "IsEmpty::is_empty")]
    pub read_only: Option<bool>,
    #[serde(rename = "deprecated", skip_serializing_if = "IsEmpty::is_empty")]
    pub deprecated: Option<bool>,
    #[serde(rename = "writeOnly", skip_serializing_if = "IsEmpty::is_empty")]
    pub write_only: Option<bool>,
    #[serde(rename = "multipleOf", skip_serializing_if = "IsEmpty::is_empty")]
    pub multiple_of: Option<OrderedFloat<f64>>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub maximum: Option<OrderedFloat<f64>>,
    #[serde(rename = "exclusiveMaximum", skip_serializing_if = "IsEmpty::is_empty")]
    pub exclusive_maximum: Option<OrderedFloat<f64>>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub minimum: Option<OrderedFloat<f64>>,
    #[serde(rename = "exclusiveMinimum", skip_serializing_if = "IsEmpty::is_empty")]
    pub exclusive_minimum: Option<OrderedFloat<f64>>,
    #[serde(rename = "maxLength", skip_serializing_if = "IsEmpty::is_empty")]
    pub max_length: Option<u64>,
    #[serde(rename = "minLength", skip_serializing_if = "IsEmpty::is_empty")]
    pub min_length: Option<u64>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub pattern: Option<String>,
    #[serde(rename = "additionalItems", skip_serializing_if = "IsEmpty::is_empty")]
    pub additional_items: Option<Schema>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub items: Option<Schema>,
    #[serde(rename = "maxItems", skip_serializing_if = "IsEmpty::is_empty")]
    pub max_items: Option<u64>,
    #[serde(rename = "minItems", skip_serializing_if = "IsEmpty::is_empty")]
    pub min_items: Option<u64>,
    #[serde(rename = "uniqueItems", skip_serializing_if = "IsEmpty::is_empty")]
    pub unique_items: Option<bool>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub contains: Option<Schema>,
    #[serde(rename = "maxProperties", skip_serializing_if = "IsEmpty::is_empty")]
    pub max_properties: Option<u64>,
    #[serde(rename = "minProperties", skip_serializing_if = "IsEmpty::is_empty")]
    pub min_properties: Option<u64>,
    #[serde(rename = "maxContains", skip_serializing_if = "IsEmpty::is_empty")]
    pub max_contains: Option<u64>,
    #[serde(rename = "minContains", skip_serializing_if = "IsEmpty::is_empty")]
    pub min_contains: Option<u64>,
    #[serde(rename = "additionalProperties", skip_serializing_if = "IsEmpty::is_empty")]
    pub additional_properties: Option<Schema>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub definitions: IndexMap<String, Schema>,
    #[serde(rename = "patternProperties", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub pattern_properties: IndexMap<String, Schema>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub dependencies: IndexMap<String, Schema>,
    #[serde(rename = "propertyNames", skip_serializing_if = "IsEmpty::is_empty")]
    pub property_names: Option<Schema>,
    #[serde(rename = "const", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(name = "const_value")]
    pub const_value: Option<serde_json::Value>,
    #[serde(rename = "type", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(name = "schema_type")]
    pub schema_type: Option<Types>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub format: String,
    #[serde(rename = "contentMediaType", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub content_media_type: String,
    #[serde(rename = "contentEncoding", skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(default)]
    pub content_encoding: String,
    #[serde(rename = "contentSchema", skip_serializing_if = "IsEmpty::is_empty")]
    pub content_schema: Option<Schema>,
    #[serde(rename = "if", skip_serializing_if = "IsEmpty::is_empty")]
    pub if_cond: Option<Schema>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    #[builder(name = "then_cond")]
    pub then: Option<Schema>,
    #[serde(rename = "else_cond", skip_serializing_if = "IsEmpty::is_empty")]
    pub else_cond: Option<Schema>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub not: Option<Schema>,
    #[serde(rename = "unevaluatedItems", skip_serializing_if = "IsEmpty::is_empty")]
    pub unevaluated_items: Option<Schema>,
    #[serde(rename = "unevaluatedProperties", skip_serializing_if = "IsEmpty::is_empty")]
    pub unevaluated_properties: Option<Schema>,
    #[serde(skip_serializing_if = "IsEmpty::is_empty")]
    pub discriminator: Option<Discriminator>,
    #[serde(flatten)]
    pub extensions: Option<Extensions>,
}

impl From<Ref> for Object {
    fn from(value: Ref) -> Self {
        Self::builder()
            .reference(value.ref_location)
            .description(value.description)
            .summary(value.summary)
            .build()
    }
}

impl<S: object_builder::State> ObjectBuilder<S> {
    pub fn properties<P: Into<String>, C: Into<Schema>>(mut self, properties: impl IntoIterator<Item = (P, C)>) -> Self {
        self.properties
            .extend(properties.into_iter().map(|(p, s)| (p.into(), s.into())));
        self
    }

    pub fn property(mut self, name: impl Into<String>, schema: impl Into<Schema>) -> Self {
        self.properties.insert(name.into(), schema.into());
        self
    }

    pub fn all_of(mut self, all_of: impl Into<Schema>) -> Self {
        self.all_of.push(all_of.into());
        self
    }

    pub fn all_ofs<C: Into<Schema>>(mut self, all_ofs: impl IntoIterator<Item = C>) -> Self {
        self.all_of.extend(all_ofs.into_iter().map(|s| s.into()));
        self
    }

    pub fn any_ofs<C: Into<Schema>>(self, any_ofs: impl IntoIterator<Item = C>) -> Self {
        any_ofs.into_iter().fold(self, |this, c| this.any_of(c))
    }

    pub fn any_of(mut self, any_of: impl Into<Schema>) -> Self {
        self.any_of.get_or_insert_default().push(any_of.into());
        self
    }

    pub fn one_ofs<C: Into<Schema>>(self, one_ofs: impl IntoIterator<Item = C>) -> Self {
        one_ofs.into_iter().fold(self, |this, c| this.one_of(c))
    }

    pub fn one_of(mut self, one_of: impl Into<Schema>) -> Self {
        self.one_of.get_or_insert_default().push(one_of.into());
        self
    }

    pub fn enum_value(mut self, enum_value: impl Into<serde_json::Value>) -> Self {
        self.enum_values.get_or_insert_default().push(enum_value.into());
        self
    }

    pub fn enum_values<E: Into<serde_json::Value>>(self, enum_values: impl IntoIterator<Item = E>) -> Self {
        enum_values.into_iter().fold(self, |this, e| this.enum_value(e))
    }

    pub fn require(mut self, require: impl Into<String>) -> Self {
        self.required.push(require.into());
        self
    }

    pub fn required<R: Into<String>>(self, required: impl IntoIterator<Item = R>) -> Self {
        required.into_iter().fold(self, |this, e| this.require(e))
    }

    pub fn example(mut self, example: impl Into<serde_json::Value>) -> Self {
        self.examples.push(example.into());
        self
    }

    pub fn examples<E: Into<serde_json::Value>>(self, examples: impl IntoIterator<Item = E>) -> Self {
        examples.into_iter().fold(self, |this, e| this.example(e))
    }
}

impl<S: object_builder::IsComplete> ObjectBuilder<S> {
    pub fn to_array(self) -> ObjectBuilder<object_builder::SetItems<object_builder::SetSchemaType>> {
        Object::builder().schema_type(Type::Array).items(self)
    }
}

impl<S: object_builder::IsComplete> From<ObjectBuilder<S>> for Object {
    fn from(value: ObjectBuilder<S>) -> Self {
        value.build()
    }
}

impl<S: object_builder::IsComplete> From<ObjectBuilder<S>> for Schema {
    fn from(value: ObjectBuilder<S>) -> Self {
        value.build().into()
    }
}

impl Object {
    pub fn with_type(ty: impl Into<Types>) -> ObjectBuilder<object_builder::SetSchemaType> {
        Object::builder().schema_type(ty)
    }

    pub fn int32() -> Object {
        Object::builder()
            .schema_type(Type::Integer)
            .maximum(i32::MAX as f64)
            .minimum(i32::MIN as f64)
            .build()
    }

    pub fn int64() -> Object {
        Object::builder()
            .schema_type(Type::Integer)
            .maximum(i64::MAX as f64)
            .minimum(i64::MIN as f64)
            .build()
    }

    pub fn uint32() -> Object {
        Object::builder()
            .schema_type(Type::Integer)
            .maximum(u32::MAX as f64)
            .minimum(u32::MIN as f64)
            .build()
    }

    pub fn uint64() -> Object {
        Object::builder()
            .schema_type(Type::Integer)
            .maximum(u64::MAX as f64)
            .minimum(u64::MIN as f64)
            .build()
    }

    pub fn to_array(self) -> Self {
        Self::builder().schema_type(Type::Array).items(self).build()
    }

    pub fn all_ofs<S: Into<Schema>>(all_ofs: impl IntoIterator<Item = S>) -> Object {
        Object::builder().all_ofs(all_ofs).build()
    }
}

macro_rules! impl_debug {
    (
        $self:ident, $debug:ident, [$($field:ident),*$(,)?]$(,)?
    ) => {
        $(
            if !IsEmpty::is_empty(&$self.$field) {
                $debug.field(stringify!($field), &$self.$field);
            }
        )*
    };
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct(stringify!(SchemaObject));

        #[rustfmt::skip]
        impl_debug!(self, debug, [
            id, schema, reference, comment, title, description, default, read_only,
            examples, multiple_of, maximum, exclusive_maximum, minimum,
            exclusive_minimum, max_length, min_length, pattern,
            additional_items, items, prefix_items, max_items, min_items, unique_items,
            contains, max_properties, min_properties, max_contains, min_contains, required,
            additional_properties, definitions, properties, pattern_properties,
            dependencies, property_names, const_value, enum_values, schema_type, format,
            content_media_type, content_encoding, if_cond, then, else_cond, all_of,
            any_of, one_of, not, unevaluated_items, unevaluated_properties,
            deprecated, write_only, content_schema, summary,
        ]);

        debug.finish()
    }
}

macro_rules! iter_chain {
    ($($item:expr),*$(,)?) => {
        std::iter::empty()
            $(.chain($item))*
    };
}

macro_rules! merge_item {
    ([$self:ident, $other:ident] => { $($item:ident => $merge_behaviour:expr),*$(,)? }) => {$({
        let self_item = &mut $self.$item;
        let other_item = &mut $other.$item;
        if IsEmpty::is_empty(self_item) {
            *self_item = std::mem::take(other_item);
        } else if self_item == other_item {
            std::mem::take(other_item);
        } else if !IsEmpty::is_empty(other_item) {
            $merge_behaviour(self_item, other_item);
        }
    })*};
}

macro_rules! impl_is_empty {
    (
        $self:ident, [$($field:ident),*$(,)?]$(,)?
    ) => {
        true $(
            && IsEmpty::is_empty(&$self.$field)
        )*
    };
}

fn dedupe_array<T: PartialEq>(items: &mut Vec<T>) {
    let mut dedupe = Vec::new();
    for item in items.drain(..) {
        if !dedupe.contains(&item) {
            dedupe.push(item);
        }
    }

    *items = dedupe;
}

impl Object {
    pub fn optimize(&mut self) {
        // Collect allofs.
        let mut all_ofs = Vec::new();
        self.take_all_ofs(&mut all_ofs);

        all_ofs
            .iter_mut()
            .filter_map(|schema| schema.as_object_mut())
            .for_each(|schema| self.merge(schema));

        // recursively call optimize
        let sub_schemas = iter_chain!(
            self.schema.iter_mut(),
            self.additional_items.iter_mut(),
            self.contains.iter_mut(),
            self.additional_properties.iter_mut(),
            self.items.iter_mut(),
            self.prefix_items.iter_mut().flatten(),
            self.definitions.values_mut(),
            self.properties.values_mut(),
            self.pattern_properties.values_mut(),
            self.dependencies.values_mut(),
            self.property_names.iter_mut(),
            self.if_cond.iter_mut(),
            self.then.iter_mut(),
            self.else_cond.iter_mut(),
            self.any_of.iter_mut().flatten(),
            self.one_of.iter_mut().flatten(),
            self.not.iter_mut(),
            self.unevaluated_items.iter_mut(),
            self.unevaluated_properties.iter_mut(),
            self.content_schema.iter_mut(),
        );

        for schema in sub_schemas {
            schema.optimize();
        }

        self.all_of = all_ofs.into_iter().filter(|schema| !schema.is_empty()).collect();
        dedupe_array(&mut self.examples);
        dedupe_array(&mut self.required);
        if let Some(_enum) = &mut self.enum_values {
            dedupe_array(_enum);
        }
        dedupe_array(&mut self.all_of);
        if let Some(any_of) = &mut self.any_of {
            dedupe_array(any_of);
        }
        if let Some(one_of) = &mut self.one_of {
            dedupe_array(one_of);
        }
    }

    pub fn into_optimized(mut self) -> Self {
        self.optimize();
        self
    }

    pub fn is_empty(&self) -> bool {
        #[rustfmt::skip]
        let empty = impl_is_empty!(self, [
            id, schema, reference, comment, title, description, default, read_only,
            examples, multiple_of, maximum, exclusive_maximum, minimum,
            exclusive_minimum, max_length, min_length, pattern,
            additional_items, items, prefix_items, max_items, min_items, unique_items,
            contains, max_properties, min_properties, max_contains, min_contains, required,
            additional_properties, definitions, properties, pattern_properties,
            dependencies, property_names, const_value, enum_values, schema_type, format,
            content_media_type, content_encoding, if_cond, then, else_cond, all_of,
            any_of, one_of, not, unevaluated_items, unevaluated_properties,
            deprecated, write_only, content_schema, summary,
        ]);

        empty
    }

    fn take_all_ofs(&mut self, collection: &mut Vec<Schema>) {
        for mut schema in self.all_of.drain(..) {
            schema.take_all_ofs(collection);
            collection.push(schema);
        }
    }

    fn merge(&mut self, other: &mut Self) {
        merge_item!(
            [self, other] => {
                id => merge_skip,
                schema => merge_sub_schema,
                reference => merge_skip,
                comment => merge_drop_second,
                title => merge_drop_second,
                description => merge_drop_second,
                summary => merge_drop_second,
                default => merge_drop_second,
                read_only => merge_set_true,
                examples => merge_array_combine,
                multiple_of => merge_multiple_of,
                maximum => merge_min,
                exclusive_maximum => merge_min,
                minimum => merge_max,
                exclusive_minimum => merge_min,
                max_length => merge_min,
                min_length => merge_max,
                pattern => merge_skip,
                additional_items => merge_sub_schema,
                items => merge_sub_schema,
                prefix_items => merge_prefix_items,
                max_items => merge_min,
                min_items => merge_max,
                unique_items => merge_set_true,
                contains => merge_sub_schema,
                max_properties => merge_min,
                min_properties => merge_max,
                max_contains => merge_min,
                min_contains => merge_max,
                required => merge_array_combine,
                additional_properties => merge_sub_schema,
                definitions => merge_schema_map,
                properties => merge_schema_map,
                pattern_properties => merge_schema_map,
                dependencies => merge_schema_map,
                property_names => merge_sub_schema,
                const_value => merge_skip,
                enum_values => merge_array_union_optional,
                schema_type => merge_type,
                format => merge_skip,
                content_media_type => merge_skip,
                content_encoding => merge_skip,
                // _if
                // then
                // _else
                any_of => merge_array_combine_optional,
                one_of => merge_array_combine_optional,
                not => merge_sub_schema,
                unevaluated_items => merge_sub_schema,
                unevaluated_properties => merge_sub_schema,
                deprecated => merge_set_true,
                write_only => merge_set_true,
                content_schema => merge_sub_schema,
            }
        );
    }
}

fn merge_skip<T>(_: &mut T, _: &mut T) {}

fn merge_drop_second<T: Default>(_: &mut T, other: &mut T) {
    std::mem::take(other);
}

fn merge_min<T: Ord + Copy>(value: &mut Option<T>, other: &mut Option<T>) {
    let value = value.as_mut().unwrap();
    let other = other.take().unwrap();
    *value = (*value).min(other);
}

fn merge_max<T: Ord + Copy>(value: &mut Option<T>, other: &mut Option<T>) {
    let value = value.as_mut().unwrap();
    let other = other.take().unwrap();
    *value = (*value).max(other);
}

fn merge_set_true(value: &mut Option<bool>, other: &mut Option<bool>) {
    other.take();
    value.replace(true);
}

fn merge_sub_schema(value: &mut Option<Schema>, other_opt: &mut Option<Schema>) {
    let value = value.as_mut().unwrap();
    let mut other = other_opt.take().unwrap();
    value.merge(&mut other);
    if !other.is_empty() {
        other_opt.replace(other);
    }
}

fn merge_array_combine<T: PartialEq>(value: &mut Vec<T>, other: &mut Vec<T>) {
    value.append(other);
}

fn merge_array_union<T: PartialEq>(value: &mut Vec<T>, other: &mut Vec<T>) {
    let other = std::mem::take(other);
    value.retain(|v| other.contains(v));
}

fn merge_array_union_optional<T: PartialEq>(value: &mut Option<Vec<T>>, other: &mut Option<Vec<T>>) {
    merge_array_union(value.as_mut().unwrap(), other.as_mut().unwrap());
    if other.as_ref().is_some_and(|o| o.is_empty()) {
        other.take();
    }
}

fn merge_array_combine_optional<T: PartialEq>(value: &mut Option<Vec<T>>, other: &mut Option<Vec<T>>) {
    merge_array_combine(value.as_mut().unwrap(), other.as_mut().unwrap());
    if other.as_ref().is_some_and(|o| o.is_empty()) {
        other.take();
    }
}

fn merge_schema_map(value: &mut IndexMap<String, Schema>, other: &mut IndexMap<String, Schema>) {
    for (key, mut other) in other.drain(..) {
        match value.entry(key) {
            indexmap::map::Entry::Occupied(mut value) => {
                value.get_mut().merge(&mut other);
                if !other.is_empty() {
                    if let Some(obj) = value.get_mut().as_object_mut() {
                        obj.all_of.push(other);
                    }
                }
            }
            indexmap::map::Entry::Vacant(v) => {
                v.insert(other);
            }
        }
    }
}

fn merge_type(value: &mut Option<Types>, other: &mut Option<Types>) {
    match (value.as_mut().unwrap(), other.take().unwrap()) {
        (Types::Single(s), Types::Single(ref o)) if s != o => {
            value.replace(Types::Multi(Vec::new()));
        }
        (Types::Single(_), Types::Single(_)) => {}
        (Types::Multi(s), Types::Multi(ref mut o)) => {
            merge_array_union(s, o);
        }
        (&mut Types::Single(s), Types::Multi(ref o)) | (&mut Types::Multi(ref o), Types::Single(s)) => {
            if o.contains(&s) {
                value.replace(Types::Single(s));
            } else {
                value.replace(Types::Multi(Vec::new()));
            }
        }
    }
}

fn merge_prefix_items(value: &mut Option<Vec<Schema>>, other: &mut Option<Vec<Schema>>) {
    let mut other = other.take().unwrap_or_default();
    let value = value.as_mut().unwrap();
    value.extend(other.drain(value.len()..));
    for (value, mut other) in value.iter_mut().zip(other) {
        value.merge(&mut other);
        if !other.is_empty() {
            if let Some(obj) = value.as_object_mut() {
                obj.all_of.push(other);
            }
        }
    }
}

fn merge_multiple_of(value: &mut Option<OrderedFloat<f64>>, other: &mut Option<OrderedFloat<f64>>) {
    let value = value.as_mut().unwrap().as_mut();
    let other = other.take().unwrap().into_inner();

    fn gcd_f64(mut a: f64, mut b: f64) -> f64 {
        a = a.abs();
        b = b.abs();
        // if either is zero, gcd is the other
        if a == 0.0 {
            return b;
        }
        if b == 0.0 {
            return a;
        }
        // Euclid’s algorithm via remainer
        while b > 0.0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }

    /// lcm(a, b) = |a * b| / gcd(a, b)
    fn lcm_f64(a: f64, b: f64) -> f64 {
        if a == 0.0 || b == 0.0 {
            return 0.0;
        }
        let g = gcd_f64(a, b);
        // (a / g) * b is a bit safer against overflow than a * (b / g)
        (a / g * b).abs()
    }

    *value = lcm_f64(*value, other);
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Schema {
    Object(Box<Object>),
    Bool(bool),
}

impl From<Object> for Schema {
    fn from(value: Object) -> Self {
        Self::object(value)
    }
}

impl From<bool> for Schema {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl Schema {
    pub fn to_array(self) -> Self {
        Self::object(Object::builder().schema_type(Type::Array).items(self))
    }

    pub fn optimize(&mut self) {
        match self {
            Self::Bool(_) => {}
            Self::Object(obj) => obj.optimize(),
        }
    }

    pub fn object(value: impl Into<Object>) -> Self {
        Self::Object(value.into().into())
    }

    fn take_all_ofs(&mut self, collection: &mut Vec<Schema>) {
        match self {
            Self::Bool(_) => {}
            Self::Object(obj) => obj.take_all_ofs(collection),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Bool(result) => *result,
            Self::Object(obj) => obj.is_empty(),
        }
    }

    fn as_object_mut(&mut self) -> Option<&mut Object> {
        match self {
            Self::Bool(_) => None,
            Self::Object(obj) => Some(obj.as_mut()),
        }
    }

    fn merge(&mut self, other: &mut Self) {
        match (self, other) {
            (this @ Schema::Bool(false), _) | (this, Schema::Bool(false)) => {
                *this = Schema::Bool(false);
            }
            (this @ Schema::Bool(true), other) => {
                std::mem::swap(this, other);
            }
            (_, Schema::Bool(true)) => {}
            (Schema::Object(value), Schema::Object(other)) => {
                value.merge(other.as_mut());
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use insta::assert_json_snapshot;
    use serde_json::{Value, json};

    use super::*;
    use crate::*;

    #[test]
    fn create_schema_serializes_json() -> Result<(), serde_json::Error> {
        let openapi = OpenApi::builder()
            .info(Info::new("My api", "1.0.0"))
            .paths(Paths::new())
            .components(
                Components::builder()
                    .schema("Person", Ref::new("#/components/PersonModel"))
                    .schema(
                        "Credential",
                        Schema::from(
                            Object::builder()
                                .property(
                                    "id",
                                    Object::builder()
                                        .schema_type(Type::Integer)
                                        .format("int32")
                                        .description("Id of credential")
                                        .default(1i32),
                                )
                                .property(
                                    "name",
                                    Object::builder().schema_type(Type::String).description("Name of credential"),
                                )
                                .property(
                                    "status",
                                    Object::builder()
                                        .schema_type(Type::String)
                                        .default("Active")
                                        .description("Credential status")
                                        .enum_values(["Active", "NotActive", "Locked", "Expired"]),
                                )
                                .property("history", Schema::from(Ref::from_schema_name("UpdateHistory")).to_array())
                                .property("tags", Object::builder().schema_type(Type::String).build().to_array()),
                        ),
                    )
                    .build(),
            )
            .build();

        let serialized = serde_json::to_string_pretty(&openapi)?;
        println!("serialized json:\n {serialized}");

        let value = serde_json::to_value(&openapi)?;
        let credential = get_json_path(&value, "components.schemas.Credential.properties");
        let person = get_json_path(&value, "components.schemas.Person");

        assert!(
            credential.get("id").is_some(),
            "could not find path: components.schemas.Credential.properties.id"
        );
        assert!(
            credential.get("status").is_some(),
            "could not find path: components.schemas.Credential.properties.status"
        );
        assert!(
            credential.get("name").is_some(),
            "could not find path: components.schemas.Credential.properties.name"
        );
        assert!(
            credential.get("history").is_some(),
            "could not find path: components.schemas.Credential.properties.history"
        );
        assert_eq!(
            credential.get("id").unwrap_or(&serde_json::value::Value::Null).to_string(),
            r#"{"default":1,"description":"Id of credential","format":"int32","type":"integer"}"#,
            "components.schemas.Credential.properties.id did not match"
        );
        assert_eq!(
            credential.get("name").unwrap_or(&serde_json::value::Value::Null).to_string(),
            r#"{"description":"Name of credential","type":"string"}"#,
            "components.schemas.Credential.properties.name did not match"
        );
        assert_eq!(
            credential
                .get("status")
                .unwrap_or(&serde_json::value::Value::Null)
                .to_string(),
            r#"{"default":"Active","description":"Credential status","enum":["Active","NotActive","Locked","Expired"],"type":"string"}"#,
            "components.schemas.Credential.properties.status did not match"
        );
        assert_eq!(
            credential
                .get("history")
                .unwrap_or(&serde_json::value::Value::Null)
                .to_string(),
            r###"{"items":{"$ref":"#/components/schemas/UpdateHistory"},"type":"array"}"###,
            "components.schemas.Credential.properties.history did not match"
        );
        assert_eq!(
            person.to_string(),
            r###"{"$ref":"#/components/PersonModel"}"###,
            "components.schemas.Person.ref did not match"
        );

        Ok(())
    }

    // Examples taken from https://spec.openapis.org/oas/latest.html#model-with-map-dictionary-properties
    #[test]
    fn test_property_order() {
        let json_value = Object::builder()
            .property(
                "id",
                Object::builder()
                    .schema_type(Type::Integer)
                    .format("int32")
                    .description("Id of credential")
                    .default(1i32),
            )
            .property(
                "name",
                Object::builder().schema_type(Type::String).description("Name of credential"),
            )
            .property(
                "status",
                Object::builder()
                    .schema_type(Type::String)
                    .default("Active")
                    .description("Credential status")
                    .enum_values(["Active", "NotActive", "Locked", "Expired"]),
            )
            .property("history", Schema::from(Ref::from_schema_name("UpdateHistory")).to_array())
            .property("tags", Object::builder().schema_type(Type::String).to_array())
            .build();

        assert_eq!(
            json_value.properties.keys().collect::<Vec<_>>(),
            vec!["id", "name", "status", "history", "tags"]
        );
    }

    // Examples taken from https://spec.openapis.org/oas/latest.html#model-with-map-dictionary-properties
    #[test]
    fn test_additional_properties() {
        let json_value = Object::builder()
            .schema_type(Type::Object)
            .additional_properties(Object::builder().schema_type(Type::String))
            .build();
        assert_json_snapshot!(json_value, @r#"
        {
          "additionalProperties": {
            "type": "string"
          },
          "type": "object"
        }
        "#);

        let json_value = Object::builder()
            .schema_type(Type::Object)
            .additional_properties(Object::builder().schema_type(Type::Number).to_array())
            .build();

        assert_json_snapshot!(json_value, @r#"
        {
          "additionalProperties": {
            "items": {
              "type": "number"
            },
            "type": "array"
          },
          "type": "object"
        }
        "#);

        let json_value = Object::builder()
            .schema_type(Type::Object)
            .additional_properties(Ref::from_schema_name("ComplexModel"))
            .build();
        assert_json_snapshot!(json_value, @r##"
        {
          "additionalProperties": {
            "$ref": "#/components/schemas/ComplexModel"
          },
          "type": "object"
        }
        "##);
    }

    #[test]
    fn test_object_with_title() {
        let json_value = Object::builder().schema_type(Type::Object).title("SomeName").build();
        assert_json_snapshot!(json_value, @r#"
        {
          "title": "SomeName",
          "type": "object"
        }
        "#);
    }

    #[test]
    fn derive_object_with_examples() {
        let json_value = Object::builder()
            .schema_type(Type::Object)
            .examples([json!({"age": 20, "name": "bob the cat"})])
            .build();
        assert_json_snapshot!(json_value, @r#"
        {
          "examples": [
            {
              "age": 20,
              "name": "bob the cat"
            }
          ],
          "type": "object"
        }
        "#);
    }

    fn get_json_path<'a>(value: &'a Value, path: &str) -> &'a Value {
        path.split('.').fold(value, |acc, fragment| {
            acc.get(fragment).unwrap_or(&serde_json::value::Value::Null)
        })
    }

    #[test]
    fn test_array_new() {
        let array = Object::builder()
            .property(
                "id",
                Object::builder()
                    .schema_type(Type::Integer)
                    .format("int32")
                    .description("Id of credential")
                    .default(json!(1i32)),
            )
            .to_array()
            .build();

        assert!(matches!(array.schema_type, Some(Types::Single(Type::Array))));
    }

    #[test]
    fn test_array_builder() {
        let array = Object::builder()
            .schema_type(Type::Array)
            .items(
                Object::builder().property(
                    "id",
                    Object::builder()
                        .schema_type(Type::Integer)
                        .format("int32")
                        .description("Id of credential")
                        .default(1i32),
                ),
            )
            .build();

        assert!(matches!(array.schema_type, Some(Types::Single(Type::Array))));
    }

    #[test]
    fn reserialize_deserialized_schema_components() {
        let components = Components::builder()
            .schemas_from_iter([(
                "Comp",
                Schema::from(
                    Object::builder()
                        .property("name", Object::builder().schema_type(Type::String))
                        .required(["name"]),
                ),
            )])
            .responses_from_iter(vec![("200", Response::builder().description("Okay").build())])
            .security_scheme(
                "TLS",
                SecurityScheme::MutualTls {
                    description: None,
                    extensions: None,
                },
            )
            .build();

        let serialized_components = serde_json::to_string(&components).unwrap();

        let deserialized_components: Components = serde_json::from_str(serialized_components.as_str()).unwrap();

        assert_eq!(
            serialized_components,
            serde_json::to_string(&deserialized_components).unwrap()
        )
    }

    #[test]
    fn reserialize_deserialized_object_component() {
        let prop = Object::builder()
            .property("name", Object::builder().schema_type(Type::String))
            .required(["name"])
            .build();

        let serialized_components = serde_json::to_string(&prop).unwrap();
        let deserialized_components: Object = serde_json::from_str(serialized_components.as_str()).unwrap();

        assert_eq!(
            serialized_components,
            serde_json::to_string(&deserialized_components).unwrap()
        )
    }

    #[test]
    fn reserialize_deserialized_property() {
        let prop = Object::builder().schema_type(Type::String).build();

        let serialized_components = serde_json::to_string(&prop).unwrap();
        let deserialized_components: Object = serde_json::from_str(serialized_components.as_str()).unwrap();

        assert_eq!(
            serialized_components,
            serde_json::to_string(&deserialized_components).unwrap()
        )
    }

    #[test]
    fn deserialize_reserialize_one_of_default_type() {
        let a = Object::builder()
            .one_ofs([
                Object::builder().property("element", Ref::new("#/test")),
                Object::builder().property("foobar", Ref::new("#/foobar")),
            ])
            .build();

        let serialized_json = serde_json::to_string(&a).expect("should serialize to json");
        let b: Object = serde_json::from_str(&serialized_json).expect("should deserialize OneOf");
        let reserialized_json = serde_json::to_string(&b).expect("reserialized json");

        println!("{serialized_json}");
        println!("{reserialized_json}",);
        assert_eq!(serialized_json, reserialized_json);
    }

    #[test]
    fn serialize_deserialize_any_of_of_within_ref_or_t_object_builder() {
        let ref_or_schema = Object::builder()
            .property(
                "test",
                Object::builder()
                    .any_ofs([
                        Object::builder().property("element", Ref::new("#/test")).build().to_array(),
                        Object::builder().property("foobar", Ref::new("#/foobar")).build(),
                    ])
                    .build(),
            )
            .build();

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");
        assert!(json_str.contains("\"anyOf\""));
        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_schema_array_ref_or_t() {
        let ref_or_schema = Object::builder()
            .property("element", Ref::new("#/test"))
            .to_array()
            .to_array()
            .build();

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_schema_array_builder() {
        let ref_or_schema = Object::builder().property("element", Ref::new("#/test")).build().to_array();

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_schema_with_additional_properties() {
        let schema = Object::builder()
            .property("map", Object::builder().additional_properties(true))
            .build();

        let json_str = serde_json::to_string(&schema).unwrap();
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).unwrap();

        let json_de_str = serde_json::to_string(&deserialized).unwrap();
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_schema_with_additional_properties_object() {
        let schema = Object::builder()
            .property(
                "map",
                Object::builder()
                    .additional_properties(Object::builder().property("name", Object::builder().schema_type(Type::String))),
            )
            .build();

        let json_str = serde_json::to_string(&schema).unwrap();
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).unwrap();

        let json_de_str = serde_json::to_string(&deserialized).unwrap();
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_discriminator_with_mapping() {
        let mut discriminator = Discriminator::new("type");
        discriminator.mapping = [("int".to_string(), "#/components/schemas/MyInt".to_string())]
            .into_iter()
            .collect::<IndexMap<_, _>>();
        let one_of = Object::builder()
            .one_of(Ref::from_schema_name("MyInt"))
            .discriminator(discriminator)
            .build();
        assert_json_snapshot!(one_of, @r##"
        {
          "oneOf": [
            {
              "$ref": "#/components/schemas/MyInt"
            }
          ],
          "discriminator": {
            "propertyName": "type",
            "mapping": {
              "int": "#/components/schemas/MyInt"
            }
          }
        }
        "##);
    }

    #[test]
    fn serialize_deserialize_object_with_multiple_schema_types() {
        let object = Object::builder().schema_type(vec![Type::Object, Type::Null]).build();

        let json_str = serde_json::to_string(&object).unwrap();
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: Object = serde_json::from_str(&json_str).unwrap();

        let json_de_str = serde_json::to_string(&deserialized).unwrap();
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn object_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::Extensions::default().add("x-some-extension", expected.clone());
        let json_value = Object::builder().extensions(extensions).build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value.get("x-some-extension"), Some(&expected));
    }

    #[test]
    fn array_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::Extensions::default().add("x-some-extension", expected.clone());
        let json_value = Object::builder().extensions(extensions).to_array().build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value["items"].get("x-some-extension"), Some(&expected));
    }

    #[test]
    fn oneof_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::Extensions::default().add("x-some-extension", expected.clone());
        let json_value = Object::builder()
            .one_of(Object::builder().extensions(extensions).build())
            .build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value["oneOf"][0].get("x-some-extension"), Some(&expected));
    }

    #[test]
    fn allof_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::Extensions::default().add("x-some-extension", expected.clone());
        let json_value = Object::builder()
            .all_of(Object::builder().extensions(extensions).build())
            .build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value["allOf"][0].get("x-some-extension"), Some(&expected));
    }

    #[test]
    fn anyof_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::Extensions::default().add("x-some-extension", expected.clone());
        let json_value = Object::builder()
            .any_of(Object::builder().extensions(extensions).build())
            .build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value["anyOf"][0].get("x-some-extension"), Some(&expected));
    }
}
