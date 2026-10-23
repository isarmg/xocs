#![cfg_attr(docsrs, doc(cfg(feature = "rkyv")))]

use rkyv::rancor::{Fallible, Source};
use rkyv::string::{ArchivedString, StringResolver};
use rkyv::{Archive, Deserialize, DeserializeUnsized, Place, Serialize, SerializeUnsized};

use crate::CompactString;

impl Archive for CompactString {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    #[inline]
    fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
        ArchivedString::resolve_from_str(self.as_str(), resolver, out);
    }
}

impl<S: Fallible + ?Sized> Serialize<S> for CompactString
where
    str: SerializeUnsized<S>,
    S::Error: Source,
{
    #[inline]
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        ArchivedString::serialize_from_str(self.as_str(), serializer)
    }
}

impl<D: Fallible + ?Sized> Deserialize<CompactString, D> for ArchivedString
where
    str: DeserializeUnsized<str, D>,
{
    #[inline]
    fn deserialize(&self, _: &mut D) -> Result<CompactString, D::Error> {
        Ok(self.as_str().into())
    }
}

impl PartialEq<CompactString> for ArchivedString {
    #[inline]
    fn eq(&self, other: &CompactString) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialOrd<CompactString> for ArchivedString {
    #[inline]
    fn partial_cmp(&self, other: &CompactString) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_str(), other.as_str())
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/rkyv/tests.rs"]
#[cfg(test)]
mod tests;
