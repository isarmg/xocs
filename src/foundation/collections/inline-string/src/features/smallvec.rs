use smallvec::SmallVec;

use crate::repr::MAX_SIZE;
use crate::CompactString;

impl CompactString {
    /// Converts a [`CompactString`] into a byte vector
    ///
    /// This consumes the [`CompactString`] and returns a [`SmallVec`], so we do not need to copy
    /// contents
    ///
    /// Note: [`SmallVec`] is an inline-able version [`Vec`](alloc::vec::Vec), just like
    /// [`CompactString`] is an inline-able version of [`String`](alloc::string::String).
    ///
    /// # Example
    /// ```
    /// use compact_str::CompactString;
    ///
    /// let c = CompactString::new("hello");
    /// let bytes = c.into_bytes();
    ///
    /// assert_eq!(&[104, 101, 108, 108, 111][..], &bytes[..]);
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "smallvec")))]
    pub fn into_bytes(self) -> SmallVec<[u8; MAX_SIZE]> {
        self.0.into_bytes()
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/smallvec/tests.rs"]
#[cfg(test)]
mod tests;
