//! AMF0 decoder

use std::io::{self, Seek};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, Bytes};
use num_traits::FromPrimitive;
use scuffle_bytes_util::{BytesCursorExt, StringCow};

use crate::{Amf0Array, Amf0Error, Amf0Marker, Amf0Object, Amf0Value};

/// AMF0 decoder.
///
/// Provides various functions to decode different types of AMF0 values from a [`Bytes`] buffer.
///
/// Cheaply cloneable because it only contains an `io::Cursor<Bytes>`.
/// See [`Bytes`] for more details about cloning.
#[derive(Debug, Clone)]
pub struct Amf0Decoder {
    pub(crate) reader: io::Cursor<Bytes>,
}

impl Amf0Decoder {
    /// Create a new deserializer from a [`Bytes`] buffer.
    pub fn new(bytes: Bytes) -> Self {
        Self {
            reader: io::Cursor::new(bytes),
        }
    }

    /// Check if there are remaining bytes to read.
    #[inline]
    pub fn has_remaining(&self) -> bool {
        self.reader.has_remaining()
    }

    /// Peek the next marker in the buffer without consuming it.
    pub fn peek_marker(&mut self) -> Result<Amf0Marker, Amf0Error> {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        // Seek back to the start of the marker
        self.reader.seek_relative(-1)?;

        Ok(marker)
    }

    /// Decode a [`Amf0Value`] from the buffer.
    pub fn decode_value<'de>(&mut self) -> Result<Amf0Value<'de>, Amf0Error> {
        let marker = self.peek_marker()?;

        match marker {
            Amf0Marker::Boolean => self.decode_boolean().map(Into::into),
            Amf0Marker::Number | Amf0Marker::Date => self.decode_number().map(Into::into),
            Amf0Marker::String | Amf0Marker::LongString | Amf0Marker::XmlDocument => self.decode_string().map(Into::into),
            Amf0Marker::Null | Amf0Marker::Undefined => self.decode_null().map(|()| Amf0Value::Null),
            Amf0Marker::Object | Amf0Marker::TypedObject | Amf0Marker::EcmaArray => self.decode_object().map(Into::into),
            Amf0Marker::StrictArray => self.decode_strict_array().map(Into::into),
            _ => Err(Amf0Error::UnsupportedMarker(marker)),
        }
    }

    /// Decode all values from the buffer until the end.
    pub fn decode_all<'de>(&mut self) -> Result<Vec<Amf0Value<'de>>, Amf0Error> {
        let mut values = Vec::new();

        while self.reader.has_remaining() {
            let value = self.decode_value()?;
            values.push(value);
        }

        Ok(values)
    }

    fn expect_marker(&mut self, expect: &'static [Amf0Marker]) -> Result<Amf0Marker, Amf0Error> {
        let marker = self.reader.read_u8()?;
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;

        if !expect.contains(&marker) {
            Err(Amf0Error::UnexpectedType {
                expected: expect,
                got: marker,
            })
        } else {
            Ok(marker)
        }
    }

    /// Decode a number from the buffer.
    pub fn decode_number(&mut self) -> Result<f64, Amf0Error> {
        let marker = self.expect_marker(&[Amf0Marker::Number, Amf0Marker::Date])?;

        let number = self.reader.read_f64::<BigEndian>()?;

        if marker == Amf0Marker::Date {
            // Skip the timezone
            self.reader.read_i16::<BigEndian>()?;
        }

        Ok(number)
    }

    /// Decode a boolean from the buffer.
    pub fn decode_boolean(&mut self) -> Result<bool, Amf0Error> {
        self.expect_marker(&[Amf0Marker::Boolean])?;
        let value = self.reader.read_u8()?;
        Ok(value != 0)
    }

    pub(crate) fn decode_normal_string<'de>(&mut self) -> Result<StringCow<'de>, Amf0Error> {
        let len = self.reader.read_u16::<BigEndian>()? as usize;

        Ok(StringCow::from_bytes(self.reader.extract_bytes(len)?.try_into()?))
    }

    /// Decode a string from the buffer.
    ///
    /// This function can decode both normal strings and long strings.
    pub fn decode_string<'de>(&mut self) -> Result<StringCow<'de>, Amf0Error> {
        let marker = self.expect_marker(&[Amf0Marker::String, Amf0Marker::LongString, Amf0Marker::XmlDocument])?;

        let len = if marker == Amf0Marker::String {
            self.reader.read_u16::<BigEndian>()? as usize
        } else {
            // LongString or XmlDocument
            self.reader.read_u32::<BigEndian>()? as usize
        };

        let s = StringCow::from_bytes(self.reader.extract_bytes(len)?.try_into()?);
        Ok(s)
    }

    /// Decode a null value from the buffer.
    ///
    /// This function can also decode undefined values.
    pub fn decode_null(&mut self) -> Result<(), Amf0Error> {
        self.expect_marker(&[Amf0Marker::Null, Amf0Marker::Undefined])?;
        Ok(())
    }

    /// Deserialize a value from the buffer using [serde].
    #[cfg(feature = "serde")]
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn deserialize<'de, T>(&mut self) -> Result<T, Amf0Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        T::deserialize(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ObjectHeader<'a> {
    Object,
    TypedObject { name: StringCow<'a> },
    EcmaArray { size: u32 },
}

impl Amf0Decoder {
    pub(crate) fn decode_object_header<'de>(&mut self) -> Result<ObjectHeader<'de>, Amf0Error> {
        let marker = self.expect_marker(&[Amf0Marker::Object, Amf0Marker::TypedObject, Amf0Marker::EcmaArray])?;

        if marker == Amf0Marker::Object {
            Ok(ObjectHeader::Object)
        } else if marker == Amf0Marker::TypedObject {
            let name = self.decode_normal_string()?;
            Ok(ObjectHeader::TypedObject { name })
        } else {
            // EcmaArray
            let size = self.reader.read_u32::<BigEndian>()?;
            Ok(ObjectHeader::EcmaArray { size })
        }
    }

    pub(crate) fn decode_object_key<'de>(&mut self) -> Result<Option<StringCow<'de>>, Amf0Error> {
        if self.decode_optional_object_end()? {
            return Ok(None);
        }

        // Object keys are not preceeded with a marker and are always normal strings
        self.decode_normal_string().map(Some)
    }

    pub(crate) fn decode_optional_object_end(&mut self) -> Result<bool, Amf0Error> {
        if self.reader.remaining() >= 3 && self.reader.read_u24::<BigEndian>()? != Amf0Marker::ObjectEnd as u32 {
            // Seek back if this wasn't an end marker
            self.reader.seek_relative(-3)?;

            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Decode an object from the buffer.
    ///
    /// This function can decode normal objects, typed objects and ECMA arrays.
    pub fn decode_object<'de>(&mut self) -> Result<Amf0Object<'de>, Amf0Error> {
        let header = self.decode_object_header()?;

        match header {
            ObjectHeader::Object | ObjectHeader::TypedObject { .. } => {
                let mut object = Amf0Object::new();

                while let Some(key) = self.decode_object_key()? {
                    let value = self.decode_value()?;
                    object.insert(key, value);
                }

                Ok(object)
            }
            ObjectHeader::EcmaArray { size } => {
                let mut object = Amf0Object::with_capacity(size as usize);

                for _ in 0..size {
                    // Object keys are not preceeded with a marker and are always normal strings
                    let key = self.decode_normal_string()?;
                    let value = self.decode_value()?;
                    object.insert(key, value);
                }

                // It seems like the object end marker is optional here?
                // Anyway, we don't need it because we are already at the end of the object here.
                self.decode_optional_object_end()?;

                Ok(object)
            }
        }
    }
}

impl Amf0Decoder {
    pub(crate) fn decode_strict_array_header(&mut self) -> Result<u32, Amf0Error> {
        self.expect_marker(&[Amf0Marker::StrictArray])?;
        let size = self.reader.read_u32::<BigEndian>()?;

        Ok(size)
    }

    /// Decode a strict array from the buffer.
    pub fn decode_strict_array<'de>(&mut self) -> Result<Amf0Array<'de>, Amf0Error> {
        let size = self.decode_strict_array_header()? as usize;

        let mut array = Vec::with_capacity(size);

        for _ in 0..size {
            let value = self.decode_value()?;
            array.push(value);
        }

        Ok(Amf0Array::from(array))
    }
}
