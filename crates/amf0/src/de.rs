//! Deserialize AMF0 data to a Rust data structure.

use std::io::{self, Seek};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use num_traits::FromPrimitive;
use scuffle_bytes_util::{BytesCursorExt, StringCow};
use serde::de::{EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess};

use crate::{Amf0Error, Amf0Marker, Amf0Value};

/// Deserialize a value from bytes.
pub fn from_bytes<'de, T>(bytes: Bytes) -> crate::Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new(bytes);
    let value = T::deserialize(&mut de)?;
    Ok(value)
}

/// Deserializer for AMF0 data.
pub struct Deserializer {
    reader: io::Cursor<Bytes>,
}

impl Deserializer {
    /// Create a new deserializer from a reader.
    pub fn new(bytes: Bytes) -> Self {
        Deserializer {
            reader: io::Cursor::new(bytes),
        }
    }

    fn expect_marker(&mut self, expect: &'static [Amf0Marker]) -> Result<(), Amf0Error> {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        if !expect.contains(&marker) {
            Err(Amf0Error::UnexpectedType {
                expected: expect,
                got: marker,
            })
        } else {
            Ok(())
        }
    }

    fn read_number(&mut self) -> Result<f64, Amf0Error> {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        if marker != Amf0Marker::Number && marker != Amf0Marker::Date {
            return Err(Amf0Error::UnexpectedType {
                expected: &[Amf0Marker::Number, Amf0Marker::Date],
                got: marker,
            });
        }

        let number = self.reader.read_f64::<BigEndian>()?;

        if marker == Amf0Marker::Date {
            // Skip the timezone
            self.reader.read_i16::<BigEndian>()?;
        }

        Ok(number)
    }

    fn read_normal_string<'de>(&mut self) -> Result<StringCow<'de>, Amf0Error> {
        let len = self.reader.read_u16::<BigEndian>()? as usize;

        Ok(StringCow::from_bytes(self.reader.extract_bytes(len)?.try_into()?))
    }

    fn read_string<'de>(&mut self) -> Result<StringCow<'de>, Amf0Error> {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        let len = if marker == Amf0Marker::String {
            self.reader.read_u16::<BigEndian>()? as usize
        } else if marker == Amf0Marker::LongString || marker == Amf0Marker::XmlDocument {
            self.reader.read_u32::<BigEndian>()? as usize
        } else {
            return Err(Amf0Error::UnexpectedType {
                expected: &[Amf0Marker::String, Amf0Marker::LongString],
                got: marker,
            });
        };

        let s = StringCow::from_bytes(self.reader.extract_bytes(len)?.try_into()?);
        Ok(s)
    }
}

impl Deserializer {
    /// Deserialize the remaining values from the reader and return them as a vector of [`Amf0Value`]s.
    pub fn deserialize_all<'de>(&mut self) -> Result<Vec<Amf0Value<'de>>, Amf0Error> {
        let mut values = Vec::new();

        while self.reader.has_remaining() {
            let value = serde::de::Deserialize::deserialize(&mut *self)?;
            values.push(value);
        }

        Ok(values)
    }

    /// Check if there are remaining bytes to read in this deserializer.
    pub fn has_remaining(&self) -> bool {
        self.reader.has_remaining()
    }
}

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer {
    type Error = Amf0Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        self.reader.seek_relative(-1)?;

        match marker {
            Amf0Marker::Boolean => self.deserialize_bool(visitor),
            Amf0Marker::Number | Amf0Marker::Date => self.deserialize_f64(visitor),
            Amf0Marker::String | Amf0Marker::LongString | Amf0Marker::XmlDocument => self.deserialize_str(visitor),
            Amf0Marker::Null | Amf0Marker::Undefined => self.deserialize_unit(visitor),
            Amf0Marker::Object | Amf0Marker::TypedObject | Amf0Marker::EcmaArray => self.deserialize_map(visitor),
            Amf0Marker::StrictArray => self.deserialize_seq(visitor),
            _ => Err(Amf0Error::UnsupportedMarker(marker)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.expect_marker(&[Amf0Marker::Boolean])?;
        let value = self.reader.read_u8()?;
        visitor.visit_bool(value != 0)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.read_number()?;
        visitor.visit_i64(value as i64)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.read_number()?;
        visitor.visit_u64(value as u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.read_number()?;
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.read_number()?;
        visitor.visit_char(value as u8 as char)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let s = self.read_string()?;
        s.into_deserializer().deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let s = self.read_string()?;
        s.into_deserializer().deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        if marker == Amf0Marker::Null || marker == Amf0Marker::Undefined {
            visitor.visit_none()
        } else {
            // We have to seek back because the marker is part of the next value
            self.reader.seek_relative(-1)?;
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        if marker == Amf0Marker::Null || marker == Amf0Marker::Undefined {
            visitor.visit_unit()
        } else {
            Err(Amf0Error::UnexpectedType {
                expected: &[Amf0Marker::Null, Amf0Marker::Undefined],
                got: marker,
            })
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.expect_marker(&[Amf0Marker::StrictArray])?;
        let size = self.reader.read_u32::<BigEndian>()? as usize;
        visitor.visit_seq(StrictArray {
            de: self,
            remaining: size,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.expect_marker(&[Amf0Marker::StrictArray])?;
        let size = self.reader.read_u32::<BigEndian>()? as usize;

        if len != size {
            return Err(Amf0Error::WrongArrayLength {
                expected: len,
                got: size,
            });
        }

        visitor.visit_seq(StrictArray {
            de: self,
            remaining: size,
        })
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        if marker == Amf0Marker::TypedObject {
            // Skip the class name
            self.read_normal_string()?;
        }

        if marker == Amf0Marker::Object || marker == Amf0Marker::TypedObject {
            visitor.visit_map(Object { de: self })
        } else if marker == Amf0Marker::EcmaArray {
            let size = self.reader.read_u32::<BigEndian>()? as usize;

            visitor.visit_map(EcmaArray {
                de: self,
                remaining: size,
            })
        } else {
            Err(Amf0Error::UnexpectedType {
                expected: &[Amf0Marker::Object, Amf0Marker::TypedObject, Amf0Marker::EcmaArray],
                got: marker,
            })
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(Enum { de: self })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let s = self.read_string()?;
        s.into_deserializer().deserialize_identifier(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct StrictArray<'a> {
    de: &'a mut Deserializer,
    remaining: usize,
}

impl<'a, 'de> SeqAccess<'de> for StrictArray<'a> {
    type Error = Amf0Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

struct Object<'a> {
    de: &'a mut Deserializer,
}

impl<'a, 'de> MapAccess<'de> for Object<'a> {
    type Error = Amf0Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let end_marker = self.de.reader.read_u24::<BigEndian>()?;
        if end_marker == Amf0Marker::ObjectEnd as u32 {
            return Ok(None);
        }

        // Seek back to the start of the key
        self.de.reader.seek_relative(-3)?;

        // Object keys are not preceeded with a marker and are always normal strings
        let s = self.de.read_normal_string()?;
        let string_de = IntoDeserializer::<Self::Error>::into_deserializer(s);
        seed.deserialize(string_de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct EcmaArray<'a> {
    de: &'a mut Deserializer,
    remaining: usize,
}

impl<'a, 'de> MapAccess<'de> for EcmaArray<'a> {
    type Error = Amf0Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            // It seems like the object end marker is optional here?
            // Anyway, we don't need it because we know the length of the array

            let maybe_end_marker = self.de.reader.read_u24::<BigEndian>()?;
            if maybe_end_marker != Amf0Marker::ObjectEnd as u32 {
                // Seek back if this wasn't an end marker
                self.de.reader.seek_relative(-3)?;
            }

            return Ok(None);
        }

        self.remaining -= 1;

        // Object keys are not preceeded with a marker and are always normal strings
        let s = self.de.read_normal_string()?;
        let string_de = IntoDeserializer::<Self::Error>::into_deserializer(s);
        seed.deserialize(string_de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

struct Enum<'a> {
    de: &'a mut Deserializer,
}

impl<'a, 'de> EnumAccess<'de> for Enum<'a> {
    type Error = Amf0Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = self.de.read_string()?;
        let string_de = IntoDeserializer::<Self::Error>::into_deserializer(variant);
        let value = seed.deserialize(string_de)?;

        Ok((value, self))
    }
}

impl<'a, 'de> VariantAccess<'de> for Enum<'a> {
    type Error = Amf0Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use core::f64;
    use std::collections::HashMap;
    use std::fmt::Debug;

    use bytes::Bytes;
    use scuffle_bytes_util::StringCow;

    use super::Deserializer;
    use crate::{Amf0Error, Amf0Marker, Amf0Value, from_bytes};

    #[test]
    fn string() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];

        let value: String = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, "hello");
    }

    #[test]
    fn bool() {
        let bytes = [Amf0Marker::Boolean as u8, 1];
        let value: bool = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert!(value);
    }

    fn number_test<'de, T>(one: T)
    where
        T: serde::Deserialize<'de> + PartialEq + Debug,
    {
        const NUMBER_ONE: [u8; 9] = const {
            let one = 1.0f64.to_be_bytes();
            [
                Amf0Marker::Number as u8,
                one[0],
                one[1],
                one[2],
                one[3],
                one[4],
                one[5],
                one[6],
                one[7],
            ]
        };

        let value: T = from_bytes(Bytes::from_static(&NUMBER_ONE)).unwrap();
        assert_eq!(value, one);
    }

    #[test]
    fn numbers() {
        number_test(1u8);
        number_test(1u8 as char);
        number_test(1u16);
        number_test(1u32);
        number_test(1u64);
        number_test(1i8);
        number_test(1i16);
        number_test(1i32);
        number_test(1i64);
        number_test(1f32);
        number_test(1f64);
    }

    #[test]
    fn optional() {
        let bytes = [Amf0Marker::Null as u8];
        let value: Option<bool> = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, None);

        let bytes = [Amf0Marker::Null as u8];
        from_bytes::<()>(Bytes::from_owner(bytes)).unwrap();

        let bytes = [Amf0Marker::String as u8];
        let err = from_bytes::<()>(Bytes::from_owner(bytes)).unwrap_err();
        assert!(matches!(
            err,
            Amf0Error::UnexpectedType {
                expected: [Amf0Marker::Null, Amf0Marker::Undefined],
                got: Amf0Marker::String
            }
        ));

        let bytes = [Amf0Marker::Undefined as u8];
        let value: Option<bool> = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, None);

        let bytes = [Amf0Marker::Boolean as u8, 0];
        let value: Option<bool> = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Some(false));

        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct Unit;

        let bytes = [Amf0Marker::Null as u8];
        let value: Unit = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Unit);
    }

    #[test]
    fn newtype_struct() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Test(String);

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test("hello".to_string()));
    }

    #[test]
    fn tuple_struct() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Test(bool, String);

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 2, // length
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test(true, "hello".to_string()));
    }

    #[test]
    fn typed_object() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Test {
            a: bool,
            b: String,
        }

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::TypedObject as u8,
            0, 1, // name length
            b'a', // name
            0, 1, // length
            b'a', // key
            Amf0Marker::Boolean as u8,
            1,
            0, 1, // length
            b'b', // key
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(
            value,
            Test {
                a: true,
                b: "hello".to_string()
            }
        );
    }

    #[test]
    fn simple_struct() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Test {
            a: bool,
            b: String,
            c: f64,
        }

        #[rustfmt::skip]
        let mut bytes = vec![
            Amf0Marker::Object as u8,
            0, 1, // length
            b'a', // key
            Amf0Marker::Boolean as u8, // value
            1,
            0, 1, // length
            b'b', // key
            Amf0Marker::String as u8, // value
            0, 1, // length
            b'b', // value
            0, 1, // length
            b'c', // key
            Amf0Marker::Number as u8, // value
        ];
        bytes.extend_from_slice(&f64::consts::PI.to_be_bytes());
        bytes.extend_from_slice(&[0, 0, Amf0Marker::ObjectEnd as u8]);
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();

        assert_eq!(
            value,
            Test {
                a: true,
                b: "b".to_string(),
                c: f64::consts::PI,
            }
        );
    }

    #[test]
    fn simple_enum() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        enum Test {
            A,
            B,
        }

        #[rustfmt::skip]
        let bytes = vec![
            Amf0Marker::String as u8,
            0, 1, // length
            b'A',
        ];
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test::A);

        #[rustfmt::skip]
        let bytes = vec![
            Amf0Marker::String as u8,
            0, 1, // length
            b'B',
        ];
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test::B);
    }

    #[test]
    fn complex_enum() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        enum Test {
            A(bool),
            B { a: String, b: String },
            C(bool, String),
        }

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 1, // length
            b'A',
            Amf0Marker::Boolean as u8,
            1,
        ];
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test::A(true));

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 1, // length
            b'B',
            Amf0Marker::Object as u8,
            0, 1, // length
            b'a',
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            0, 1, // length
            b'b',
            Amf0Marker::String as u8,
            0, 5, // length
            b'w', b'o', b'r', b'l', b'd',
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(
            value,
            Test::B {
                a: "hello".to_string(),
                b: "world".to_string()
            }
        );

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 1, // length
            b'C',
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 2, // array length
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];
        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test::C(true, "hello".to_string()));
    }

    #[test]
    fn series() {
        #[rustfmt::skip]
        let mut bytes = vec![
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::Number as u8,
        ];
        bytes.extend_from_slice(&f64::consts::PI.to_be_bytes());

        let mut de = Deserializer::new(Bytes::from_owner(bytes));
        let value: String = serde::de::Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(value, "hello");
        let value: bool = serde::de::Deserialize::deserialize(&mut de).unwrap();
        assert!(value);
        let value: f64 = serde::de::Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(value, f64::consts::PI);
    }

    #[test]
    fn flatten() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::Object as u8,
            0, 1, // length
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0, 1, // length
            b'b',
            Amf0Marker::String as u8,
            0, 1, // length
            b'b',
            0, 1, // length
            b'c',
            Amf0Marker::String as u8,
            0, 1, // length
            b'c',
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct Test<'a> {
            b: String,
            #[serde(flatten, borrow)]
            other: HashMap<StringCow<'a>, Amf0Value<'a>>,
        }

        let value: Test = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(
            value,
            Test {
                b: "b".to_string(),
                other: vec![
                    ("a".into(), Amf0Value::from(true)),
                    ("c".into(), StringCow::from_static("c").into())
                ]
                .into_iter()
                .collect(),
            }
        );
    }

    #[test]
    fn all() {
        let bytes = [
            Amf0Marker::String as u8,
            0,
            5, // length
            b'h',
            b'e',
            b'l',
            b'l',
            b'o',
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::Object as u8,
            0,
            1, // length
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0,
            0,
            Amf0Marker::ObjectEnd as u8,
        ];

        let mut de = Deserializer::new(Bytes::from_owner(bytes));
        let values = de.deserialize_all().unwrap();
        assert_eq!(
            values,
            vec![
                Amf0Value::String("hello".into()),
                Amf0Value::Boolean(true),
                Amf0Value::Object([("a".into(), Amf0Value::Boolean(true))].into_iter().collect())
            ]
        );
    }
}
