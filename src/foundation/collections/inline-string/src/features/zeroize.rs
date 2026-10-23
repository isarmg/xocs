//! Implements the [`zeroize::Zeroize`] trait for [`CompactString`]

use crate::CompactString;
use zeroize::Zeroize;

#[cfg_attr(docsrs, doc(cfg(feature = "zeroize")))]
impl Zeroize for CompactString {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/zeroize/tests.rs"]
#[cfg(test)]
mod tests;
