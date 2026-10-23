//! Implements the [`quickcheck::Arbitrary`] trait for [`CompactString`]

use alloc::boxed::Box;
use alloc::vec::Vec;

use quickcheck::{Arbitrary, Gen};

use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "quickcheck")))]
impl Arbitrary for CompactString {
    fn arbitrary(g: &mut Gen) -> CompactString {
        let max = g.size();

        // pick some value in [0, max]
        let x = usize::arbitrary(g);
        let ratio = (x as f64) / (usize::MAX as f64);
        let size = (ratio * max as f64) as usize;

        (0..size).map(|_| char::arbitrary(g)).collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = CompactString>> {
        // Shrink a string by shrinking a vector of its characters.
        let chars: Vec<char> = self.chars().collect();
        Box::new(
            chars
                .shrink()
                .map(|x| x.into_iter().collect::<CompactString>()),
        )
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/quickcheck/test.rs"]
#[cfg(test)]
mod test;
