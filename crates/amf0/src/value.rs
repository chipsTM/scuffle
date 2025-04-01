//! AMF0 value types.

use std::borrow::Cow;
use std::collections::BTreeMap;

use scuffle_bytes_util::StringCow;
use serde::ser::{SerializeMap, SerializeSeq};

/// Represents any AMF0 object.
pub type Amf0Object<'a> = Cow<'a, [(StringCow<'a>, Amf0Value<'a>)]>;
/// Represents any AMF0 array.
pub type Amf0Array<'a> = Cow<'a, [Amf0Value<'a>]>;

/// Represents any AMF0 value.
#[derive(Debug, PartialEq, Clone)]
pub enum Amf0Value<'a> {
    /// AMF0 Number.
    Number(f64),
    /// AMF0 Boolean.
    Boolean(bool),
    /// AMF0 String.
    String(StringCow<'a>),
    /// AMF0 Object.
    Object(Amf0Object<'a>),
    /// AMF0 Null.
    Null,
    /// AMF0 Array.
    Array(Amf0Array<'a>),
}

impl<'a> Amf0Value<'a> {
    /// Converts this AMF0 value into an owned version (static lifetime).
    pub fn into_owned(self) -> Amf0Value<'static> {
        match self {
            Amf0Value::Number(v) => Amf0Value::Number(v),
            Amf0Value::Boolean(v) => Amf0Value::Boolean(v),
            Amf0Value::String(v) => Amf0Value::String(v.into_owned()),
            Amf0Value::Object(v) => Amf0Value::Object(Cow::Owned(
                v.into_owned()
                    .into_iter()
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect(),
            )),
            Amf0Value::Null => Amf0Value::Null,
            Amf0Value::Array(v) => Amf0Value::Array(v.into_owned().into_iter().map(|v| v.into_owned()).collect()),
        }
    }
}

impl From<f64> for Amf0Value<'_> {
    fn from(value: f64) -> Self {
        Amf0Value::Number(value)
    }
}

impl From<bool> for Amf0Value<'_> {
    fn from(value: bool) -> Self {
        Amf0Value::Boolean(value)
    }
}

impl<'a> From<StringCow<'a>> for Amf0Value<'a> {
    fn from(value: StringCow<'a>) -> Self {
        Amf0Value::String(value)
    }
}

// owned object
impl<'a> From<BTreeMap<String, Amf0Value<'a>>> for Amf0Value<'a> {
    fn from(value: BTreeMap<String, Amf0Value<'a>>) -> Self {
        Amf0Value::Object(Cow::Owned(value.into_iter().map(|(k, v)| (k.into(), v)).collect()))
    }
}

// borrowed object
impl<'a> From<&'a [(StringCow<'a>, Amf0Value<'a>)]> for Amf0Value<'a> {
    fn from(value: &'a [(StringCow<'a>, Amf0Value<'a>)]) -> Self {
        Amf0Value::Object(Cow::Borrowed(value))
    }
}

// cow object
impl<'a> From<Amf0Object<'a>> for Amf0Value<'a> {
    fn from(value: Amf0Object<'a>) -> Self {
        Amf0Value::Object(value)
    }
}

impl<'a> FromIterator<(StringCow<'a>, Amf0Value<'a>)> for Amf0Value<'a> {
    fn from_iter<T: IntoIterator<Item = (StringCow<'a>, Amf0Value<'a>)>>(iter: T) -> Self {
        Amf0Value::Object(Cow::Owned(iter.into_iter().collect()))
    }
}

// owned array
impl<'a> From<Vec<Amf0Value<'a>>> for Amf0Value<'a> {
    fn from(value: Vec<Amf0Value<'a>>) -> Self {
        Amf0Value::Array(Cow::Owned(value))
    }
}

// borrowed array
impl<'a> From<&'a [Amf0Value<'a>]> for Amf0Value<'a> {
    fn from(value: &'a [Amf0Value<'a>]) -> Self {
        Amf0Value::Array(Cow::Borrowed(value))
    }
}

// cow array
impl<'a> From<Amf0Array<'a>> for Amf0Value<'a> {
    fn from(value: Amf0Array<'a>) -> Self {
        Amf0Value::Array(value)
    }
}

impl<'a> FromIterator<Amf0Value<'a>> for Amf0Value<'a> {
    fn from_iter<T: IntoIterator<Item = Amf0Value<'a>>>(iter: T) -> Self {
        Amf0Value::Array(Cow::Owned(iter.into_iter().collect()))
    }
}

impl<'de> serde::de::Deserialize<'de> for Amf0Value<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Amf0ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for Amf0ValueVisitor {
            type Value = Amf0Value<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an AMF0 value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Boolean(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v as f64)
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v as f64)
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Number(v))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(v.to_owned())
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from(v).into())
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from(v).into())
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Null)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(value) = seq.next_element()? {
                    vec.push(value);
                }

                Ok(vec.into())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut object = BTreeMap::new();

                while let Some((key, value)) = map.next_entry()? {
                    object.insert(key, value);
                }

                Ok(object.into())
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let array = v.iter().map(|b| Amf0Value::Number(*b as f64)).collect();
                Ok(Amf0Value::Array(array))
            }
        }

        deserializer.deserialize_any(Amf0ValueVisitor)
    }
}

impl<'a> serde::ser::Serialize for Amf0Value<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Amf0Value::Number(v) => serializer.serialize_f64(*v),
            Amf0Value::Boolean(v) => serializer.serialize_bool(*v),
            Amf0Value::String(v) => v.serialize(serializer),
            Amf0Value::Object(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;

                for (key, value) in v.iter() {
                    map.serialize_entry(key, value)?;
                }

                map.end()
            }
            Amf0Value::Null => serializer.serialize_none(),
            Amf0Value::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;

                for value in v.iter() {
                    seq.serialize_element(value)?;
                }

                seq.end()
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Bytes;

    use super::Amf0Value;
    use crate::{Amf0Marker, from_bytes};

    #[test]
    fn string() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 3, // length
            b'a', b'b', b'c',
        ];

        let value: Amf0Value = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Amf0Value::String("abc".into()));
    }

    #[test]
    fn bool() {
        let bytes = [Amf0Marker::Boolean as u8, 0];

        let value: Amf0Value = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Amf0Value::Boolean(false));
    }

    #[test]
    fn object() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::Object as u8,
            0, 1,
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        let value: Amf0Value = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(
            value,
            Amf0Value::Object([("a".into(), Amf0Value::Boolean(true))].into_iter().collect())
        );
    }
}
