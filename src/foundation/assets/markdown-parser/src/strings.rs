use std::borrow::{Borrow, Cow};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::str::from_utf8;

const MAX_INLINE_STR_LEN: usize = 3 * std::mem::size_of::<isize>() - 2;

/// Returned when trying to convert a `&str` into a `InlineStr`
/// but it fails because it doesn't fit.
#[derive(Debug)]
pub struct StringTooLongError;

/// An inline string that can contain almost three words
/// of utf-8 text.
#[derive(Debug, Clone, Copy, Eq)]
pub struct InlineStr {
    inner: [u8; MAX_INLINE_STR_LEN],
    len: u8,
}

impl AsRef<str> for InlineStr {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl Hash for InlineStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl From<char> for InlineStr {
    fn from(c: char) -> Self {
        let mut inner = [0u8; MAX_INLINE_STR_LEN];
        c.encode_utf8(&mut inner);
        let len = c.len_utf8() as u8;
        Self { inner, len }
    }
}

impl std::cmp::PartialEq<InlineStr> for InlineStr {
    fn eq(&self, other: &InlineStr) -> bool {
        self.deref() == other.deref()
    }
}

impl TryFrom<&str> for InlineStr {
    type Error = StringTooLongError;

    fn try_from(s: &str) -> Result<InlineStr, StringTooLongError> {
        let len = s.len();
        if len <= MAX_INLINE_STR_LEN {
            let mut inner = [0u8; MAX_INLINE_STR_LEN];
            inner[..len].copy_from_slice(s.as_bytes());
            let len = len as u8;
            Ok(Self { inner, len })
        } else {
            Err(StringTooLongError)
        }
    }
}

impl Deref for InlineStr {
    type Target = str;

    fn deref(&self) -> &str {
        let len = self.len as usize;
        from_utf8(&self.inner[..len]).unwrap()
    }
}

impl fmt::Display for InlineStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// A copy-on-write string that can be owned, borrowed
/// or inlined.
///
/// It is three words long.
#[derive(Debug, Eq)]
pub enum CowStr<'a> {
    /// An owned, immutable string.
    Boxed(Box<str>),
    /// A borrowed string.
    Borrowed(&'a str),
    /// A short inline string.
    Inlined(InlineStr),
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::CowStr;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;

    impl<'a> Serialize for CowStr<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.as_ref())
        }
    }

    struct CowStrVisitor;

    impl<'de> de::Visitor<'de> for CowStrVisitor {
        type Value = CowStr<'de>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(CowStr::Borrowed(v))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v.try_into() {
                Ok(it) => Ok(CowStr::Inlined(it)),
                Err(_) => Ok(CowStr::Boxed(String::from(v).into_boxed_str())),
            }
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(CowStr::Boxed(v.into_boxed_str()))
        }
    }

    impl<'a, 'de: 'a> Deserialize<'de> for CowStr<'a> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(CowStrVisitor)
        }
    }
}

impl<'a> AsRef<str> for CowStr<'a> {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl<'a> Hash for CowStr<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl<'a> std::clone::Clone for CowStr<'a> {
    fn clone(&self) -> Self {
        match self {
            CowStr::Boxed(s) => match InlineStr::try_from(&**s) {
                Ok(inline) => CowStr::Inlined(inline),
                Err(..) => CowStr::Boxed(s.clone()),
            },
            CowStr::Borrowed(s) => CowStr::Borrowed(s),
            CowStr::Inlined(s) => CowStr::Inlined(*s),
        }
    }
}

impl<'a> std::cmp::PartialEq<CowStr<'a>> for CowStr<'a> {
    fn eq(&self, other: &CowStr<'_>) -> bool {
        self.deref() == other.deref()
    }
}

impl<'a> From<&'a str> for CowStr<'a> {
    fn from(s: &'a str) -> Self {
        CowStr::Borrowed(s)
    }
}

impl<'a> From<String> for CowStr<'a> {
    fn from(s: String) -> Self {
        CowStr::Boxed(s.into_boxed_str())
    }
}

impl<'a> From<char> for CowStr<'a> {
    fn from(c: char) -> Self {
        CowStr::Inlined(c.into())
    }
}

impl<'a> From<Cow<'a, str>> for CowStr<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        match s {
            Cow::Borrowed(s) => CowStr::Borrowed(s),
            Cow::Owned(s) => CowStr::Boxed(s.into_boxed_str()),
        }
    }
}

impl<'a> From<CowStr<'a>> for Cow<'a, str> {
    fn from(s: CowStr<'a>) -> Self {
        match s {
            CowStr::Boxed(s) => Cow::Owned(s.to_string()),
            CowStr::Inlined(s) => Cow::Owned(s.to_string()),
            CowStr::Borrowed(s) => Cow::Borrowed(s),
        }
    }
}

impl<'a> From<Cow<'a, char>> for CowStr<'a> {
    fn from(s: Cow<'a, char>) -> Self {
        CowStr::Inlined(InlineStr::from(*s))
    }
}

impl<'a> From<CowStr<'a>> for String {
    fn from(s: CowStr<'a>) -> Self {
        match s {
            CowStr::Boxed(s) => s.into(),
            CowStr::Inlined(s) => s.as_ref().into(),
            CowStr::Borrowed(s) => s.into(),
        }
    }
}

impl<'a> Deref for CowStr<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            CowStr::Boxed(ref b) => b,
            CowStr::Borrowed(b) => b,
            CowStr::Inlined(ref s) => s.deref(),
        }
    }
}

impl<'a> Borrow<str> for CowStr<'a> {
    fn borrow(&self) -> &str {
        self.deref()
    }
}

impl<'a> CowStr<'a> {
    pub fn into_string(self) -> String {
        match self {
            CowStr::Boxed(b) => b.into(),
            CowStr::Borrowed(b) => b.to_owned(),
            CowStr::Inlined(s) => s.deref().to_owned(),
        }
    }

    pub fn into_static(self) -> CowStr<'static> {
        match self {
            CowStr::Boxed(b) => CowStr::Boxed(b),
            CowStr::Borrowed(b) => match InlineStr::try_from(b) {
                Ok(inline) => CowStr::Inlined(inline),
                Err(_) => CowStr::Boxed(b.into()),
            },
            CowStr::Inlined(s) => CowStr::Inlined(s),
        }
    }
}

impl<'a> fmt::Display for CowStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[path = "../../../../../test/foundation/assets/markdown-parser/unit/src/strings/test_special_string.rs"]
#[cfg(test)]
mod test_special_string;
