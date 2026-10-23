//! Implements the [`arbitrary::Arbitrary`] trait for [`CompactString`]

use arbitrary::{Arbitrary, Result, Unstructured};

use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "arbitrary")))]
impl<'a> Arbitrary<'a> for CompactString {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        <&str as Arbitrary>::arbitrary(u).map(CompactString::new)
    }

    fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
        <&str as Arbitrary>::arbitrary_take_rest(u).map(CompactString::new)
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <&str as Arbitrary>::size_hint(depth)
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/arbitrary/test.rs"]
#[cfg(test)]
mod test;
