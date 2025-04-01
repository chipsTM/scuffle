use super::{BytesCow, StringCow};

impl serde::Serialize for BytesCow<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'de> serde::Deserialize<'de> for BytesCow<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesCowVisitor;

        impl<'de> serde::de::Visitor<'de> for BytesCowVisitor {
            type Value = BytesCow<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte slice")
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_vec(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_vec(v.to_vec()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_vec(v.as_bytes().to_vec()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_vec(v.into_bytes()))
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_slice(v))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BytesCow::from_slice(v.as_bytes()))
            }
        }

        deserializer.deserialize_bytes(BytesCowVisitor)
    }
}

impl serde::Serialize for StringCow<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for StringCow<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StringCowVisitor;

        impl<'de> serde::de::Visitor<'de> for StringCowVisitor {
            type Value = StringCow<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from_ref(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from_string(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from_string(v.to_string()))
            }
        }

        deserializer.deserialize_any(StringCowVisitor)
    }
}

impl<'de, E> serde::de::IntoDeserializer<'de, E> for StringCow<'de>
where
    E: serde::de::Error,
{
    type Deserializer = StringCowDeserializer<'de, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        StringCowDeserializer::<E>::new(self)
    }
}

/// A deserializer for [`StringCow`].
pub struct StringCowDeserializer<'a, E> {
    cow: StringCow<'a>,
    _marker: std::marker::PhantomData<E>,
}

impl<'a, E> StringCowDeserializer<'a, E> {
    /// Creates a new [`StringCowDeserializer`].
    pub fn new(cow: StringCow<'a>) -> Self {
        StringCowDeserializer {
            cow,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, E> serde::de::Deserializer<'de> for StringCowDeserializer<'de, E>
where
    E: serde::de::Error,
{
    type Error = E;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.cow {
            StringCow::Ref(slice) => visitor.visit_borrowed_str(slice),
            StringCow::StaticRef(slice) => visitor.visit_borrowed_str(slice),
            StringCow::String(string) => visitor.visit_string(string),
            StringCow::Bytes(bytes) => visitor.visit_str(&bytes),
        }
    }
}
