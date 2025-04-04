//! AMF0 decoder

use num_traits::FromPrimitive;
use scuffle_bytes_util::StringCow;

use crate::{Amf0Array, Amf0Error, Amf0Marker, Amf0Object, Amf0Value};

/// AMF0 decoder.
///
/// Provides various functions to decode different types of AMF0 values from a [`Bytes`] buffer.
///
/// Cheaply cloneable because it only contains an `io::Cursor<Bytes>`.
/// See [`Bytes`] for more details about cloning.
#[derive(Debug, Clone)]
pub struct Amf0Decoder<B> {
    pub(crate) buf: B,
    pub(crate) next_marker: Option<Amf0Marker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ObjectHeader<'a> {
    Object,
    TypedObject { name: StringCow<'a> },
    EcmaArray { size: u32 },
}

impl<B> Amf0Decoder<B>
where
    B: bytes::Buf,
{
    /// Create a new deserializer from a [`Bytes`] buffer.
    pub fn new(buf: B) -> Self {
        Self { buf, next_marker: None }
    }

    /// Check if there are remaining bytes to read.
    #[inline]
    pub fn has_remaining(&self) -> bool {
        self.buf.has_remaining()
    }

    /// Decode a [`Amf0Value`] from the buffer.
    pub fn decode_value(&mut self) -> Result<Amf0Value<'static>, Amf0Error> {
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
    pub fn decode_all(&mut self) -> Result<Vec<Amf0Value<'static>>, Amf0Error> {
        let mut values = Vec::new();

        while self.buf.has_remaining() {
            let value = self.decode_value()?;
            values.push(value);
        }

        Ok(values)
    }

    /// Peek the next marker in the buffer without consuming it.
    pub fn peek_marker(&mut self) -> Result<Amf0Marker, Amf0Error> {
        let marker = self.read_marker()?;
        // Buffer the marker for the next read
        self.next_marker = Some(marker);

        Ok(marker)
    }

    fn read_marker(&mut self) -> Result<Amf0Marker, Amf0Error> {
        if let Some(marker) = self.next_marker.take() {
            return Ok(marker);
        }

        let marker = self.buf.get_u8();
        let marker = Amf0Marker::from_u8(marker).ok_or(Amf0Error::UnknownMarker(marker))?;
        Ok(marker)
    }

    fn expect_marker(&mut self, expect: &'static [Amf0Marker]) -> Result<Amf0Marker, Amf0Error> {
        let marker = self.read_marker()?;

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

        let number = self.buf.get_f64();

        if marker == Amf0Marker::Date {
            // Skip the timezone
            self.buf.get_i16();
        }

        Ok(number)
    }

    /// Decode a boolean from the buffer.
    pub fn decode_boolean(&mut self) -> Result<bool, Amf0Error> {
        self.expect_marker(&[Amf0Marker::Boolean])?;
        let value = self.buf.get_u8();
        Ok(value != 0)
    }

    pub(crate) fn decode_normal_string(&mut self) -> Result<StringCow<'static>, Amf0Error> {
        let len = self.buf.get_u16() as usize;

        let bytes = self.buf.copy_to_bytes(len);
        Ok(StringCow::from_bytes(bytes.try_into()?))
    }

    /// Decode a string from the buffer.
    ///
    /// This function can decode both normal strings and long strings.
    pub fn decode_string(&mut self) -> Result<StringCow<'static>, Amf0Error> {
        let marker = self.expect_marker(&[Amf0Marker::String, Amf0Marker::LongString, Amf0Marker::XmlDocument])?;

        let len = if marker == Amf0Marker::String {
            self.buf.get_u16() as usize
        } else {
            // LongString or XmlDocument
            self.buf.get_u32() as usize
        };

        let bytes = self.buf.copy_to_bytes(len);
        Ok(StringCow::from_bytes(bytes.try_into()?))
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

    // --- Object and Ecma array ---

    pub(crate) fn decode_object_header(&mut self) -> Result<ObjectHeader<'static>, Amf0Error> {
        let marker = self.expect_marker(&[Amf0Marker::Object, Amf0Marker::TypedObject, Amf0Marker::EcmaArray])?;

        if marker == Amf0Marker::Object {
            Ok(ObjectHeader::Object)
        } else if marker == Amf0Marker::TypedObject {
            let name = self.decode_normal_string()?;
            Ok(ObjectHeader::TypedObject { name })
        } else {
            // EcmaArray
            let size = self.buf.get_u32();
            Ok(ObjectHeader::EcmaArray { size })
        }
    }

    pub(crate) fn decode_object_key(&mut self) -> Result<Option<StringCow<'static>>, Amf0Error> {
        // Object keys are not preceeded with a marker and are always normal strings
        let key = self.decode_normal_string()?;

        // The object end marker is preceeded by an empty string
        if key.as_str().is_empty() {
            // Check if the next marker is an object end marker
            if self.peek_marker()? == Amf0Marker::ObjectEnd {
                // Clear the next marker buffer
                self.next_marker = None;

                return Ok(None);
            }
        }

        Ok(Some(key))
    }

    /// Decode an object from the buffer.
    ///
    /// This function can decode normal objects, typed objects and ECMA arrays.
    pub fn decode_object(&mut self) -> Result<Amf0Object<'static>, Amf0Error> {
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

                // There might be an object end marker after the last key
                if self.peek_marker()? == Amf0Marker::ObjectEnd {
                    // Clear the next marker buffer
                    self.next_marker = None;
                }

                Ok(object)
            }
        }
    }

    // --- Strict array ---

    pub(crate) fn decode_strict_array_header(&mut self) -> Result<u32, Amf0Error> {
        self.expect_marker(&[Amf0Marker::StrictArray])?;
        let size = self.buf.get_u32();

        Ok(size)
    }

    /// Decode a strict array from the buffer.
    pub fn decode_strict_array(&mut self) -> Result<Amf0Array<'static>, Amf0Error> {
        let size = self.decode_strict_array_header()? as usize;

        let mut array = Vec::with_capacity(size);

        for _ in 0..size {
            let value = self.decode_value()?;
            array.push(value);
        }

        Ok(Amf0Array::from(array))
    }
}
