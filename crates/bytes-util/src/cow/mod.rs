use std::borrow::Cow;
use std::fmt::Display;
use std::hash::Hash;

use bytes::Bytes;
use bytestring::ByteString;

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub(crate) mod serde;

/// A [`Cow`] type for bytes.
#[derive(Debug, Clone, Eq)]
pub enum BytesCow<'a> {
    /// A borrowed [`Bytes`] object.
    Slice(&'a [u8]),
    /// A staticly borrowed [`Bytes`] object.
    StaticSlice(&'static [u8]),
    /// An owned [`Vec`] of bytes.
    Vec(Vec<u8>),
    /// An owned [`Bytes`] object.
    Bytes(Bytes),
}

impl<T> PartialEq<T> for BytesCow<'_>
where
    T: AsRef<[u8]>,
{
    fn eq(&self, other: &T) -> bool {
        self.as_bytes() == other.as_ref()
    }
}

impl Hash for BytesCow<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl Default for BytesCow<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> BytesCow<'a> {
    /// Creates an empty [`BytesCow`] object.
    pub fn new() -> Self {
        Self::from_static(b"")
    }

    /// Creates a new [`BytesCow`] from a static slice.
    pub fn from_static(slice: &'static [u8]) -> Self {
        Self::StaticSlice(slice)
    }

    /// Creates a new [`BytesCow`] from a slice of bytes.
    pub fn from_slice(slice: &'a [u8]) -> Self {
        Self::Slice(slice)
    }

    /// Creates a new [`BytesCow`] from a [`Bytes`] object.
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self::Bytes(bytes)
    }

    /// Creates a new [`BytesCow`] from a [`Cow`] of a [`Bytes`] object.
    pub fn from_cow(cow: Cow<'a, [u8]>) -> Self {
        match cow {
            Cow::Borrowed(slice) => Self::Slice(slice),
            Cow::Owned(bytes) => Self::Vec(bytes),
        }
    }

    /// Creates a new [`BytesCow`] from a [`Vec`] of bytes.
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        Self::Vec(bytes)
    }

    /// Converts the object into a [`Bytes`] object.
    pub fn into_bytes(self) -> Bytes {
        match self {
            Self::Slice(slice) => Bytes::copy_from_slice(slice),
            Self::StaticSlice(slice) => Bytes::from_static(slice),
            Self::Vec(bytes) => Bytes::from(bytes),
            Self::Bytes(bytes) => bytes,
        }
    }

    /// Returns a reference to the inner data as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Slice(slice) => slice,
            Self::StaticSlice(slice) => slice,
            Self::Vec(bytes) => bytes.as_slice(),
            Self::Bytes(bytes) => bytes.as_ref(),
        }
    }
}

impl AsRef<[u8]> for BytesCow<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            BytesCow::Slice(slice) => slice,
            BytesCow::StaticSlice(slice) => slice,
            BytesCow::Vec(bytes) => bytes.as_slice(),
            BytesCow::Bytes(bytes) => bytes.as_ref(),
        }
    }
}

impl<'a> From<Cow<'a, [u8]>> for BytesCow<'a> {
    fn from(cow: Cow<'a, [u8]>) -> Self {
        BytesCow::from_cow(cow)
    }
}

impl From<Bytes> for BytesCow<'_> {
    fn from(bytes: Bytes) -> Self {
        BytesCow::from_bytes(bytes)
    }
}

impl<'a> From<&'a [u8]> for BytesCow<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        BytesCow::from_slice(bytes)
    }
}

impl From<Vec<u8>> for BytesCow<'_> {
    fn from(bytes: Vec<u8>) -> Self {
        BytesCow::from_vec(bytes)
    }
}

/// A [`Cow`] type for strings.
#[derive(Debug, Clone, Eq)]
pub enum StringCow<'a> {
    /// A borrowed [`ByteString`] object.
    Ref(&'a str),
    /// A staticly borrowed [`ByteString`] object.
    StaticRef(&'static str),
    /// An owned [`String`] object.
    String(String),
    /// An owned [`ByteString`] object.
    Bytes(ByteString),
}

impl Default for StringCow<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq<str> for StringCow<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl Hash for StringCow<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for StringCow<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for StringCow<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<T> PartialEq<T> for StringCow<'_>
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl Display for StringCow<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringCow::Ref(slice) => slice.fmt(f),
            StringCow::StaticRef(slice) => slice.fmt(f),
            StringCow::String(string) => string.fmt(f),
            StringCow::Bytes(bytes) => bytes.fmt(f),
        }
    }
}

impl<'a> StringCow<'a> {
    /// Creates an empty [`StringCow`] object.
    pub fn new() -> Self {
        Self::from_static("")
    }

    /// Creates a new [`StringCow`] from a static slice.
    pub fn from_static(slice: &'static str) -> Self {
        StringCow::StaticRef(slice)
    }

    /// Creates a new [`StringCow`] from a [`ByteString`] object.
    pub fn from_bytes(bytes: ByteString) -> Self {
        StringCow::Bytes(bytes)
    }

    /// Creates a new [`StringCow`] from a [`Cow`] of a [`str`] object.
    pub fn from_cow(cow: Cow<'a, str>) -> Self {
        match cow {
            Cow::Borrowed(slice) => StringCow::Ref(slice),
            Cow::Owned(string) => StringCow::String(string),
        }
    }

    /// Creates a new [`StringCow`] from a static slice.
    pub fn from_ref(slice: &'a str) -> Self {
        StringCow::Ref(slice)
    }

    /// Creates a new [`StringCow`] from a [`String`] object.
    pub fn from_string(string: String) -> Self {
        StringCow::String(string)
    }

    /// Converts the object into a [`ByteString`] object.
    pub fn into_bytes(self) -> ByteString {
        match self {
            StringCow::Ref(slice) => ByteString::from(slice),
            StringCow::StaticRef(slice) => ByteString::from_static(slice),
            StringCow::String(string) => ByteString::from(string),
            StringCow::Bytes(bytes) => bytes,
        }
    }

    /// Converts this [`StringCow`] into an owned version with a static lifetime.
    pub fn into_owned(self) -> StringCow<'static> {
        match self {
            StringCow::Ref(slice) => StringCow::from(slice.to_owned()),
            StringCow::StaticRef(slice) => StringCow::StaticRef(slice),
            StringCow::String(string) => StringCow::String(string),
            StringCow::Bytes(bytes) => StringCow::Bytes(bytes),
        }
    }

    /// Returns a reference to the inner data as a slice.
    pub fn as_str(&self) -> &str {
        match self {
            StringCow::Ref(slice) => slice,
            StringCow::StaticRef(slice) => slice,
            StringCow::String(string) => string.as_str(),
            StringCow::Bytes(bytes) => bytes.as_ref(),
        }
    }
}

impl AsRef<str> for StringCow<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> From<Cow<'a, str>> for StringCow<'a> {
    fn from(cow: Cow<'a, str>) -> Self {
        StringCow::from_cow(cow)
    }
}

impl<'a> From<&'a str> for StringCow<'a> {
    fn from(slice: &'a str) -> Self {
        StringCow::from_ref(slice)
    }
}

impl From<String> for StringCow<'_> {
    fn from(string: String) -> Self {
        StringCow::from_string(string)
    }
}

impl From<ByteString> for StringCow<'_> {
    fn from(bytes: ByteString) -> Self {
        StringCow::from_bytes(bytes)
    }
}
