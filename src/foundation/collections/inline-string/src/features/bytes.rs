use core::str::Utf8Error;

use bytes::Buf;

use crate::{CompactString, Repr};

impl CompactString {
    /// Converts a buffer of bytes to a [`CompactString`]
    ///
    /// # Examples
    /// ### Basic usage
    /// ```
    /// # use compact_str::CompactString;
    /// # use std::collections::VecDeque;
    ///
    /// // `bytes::Buf` is implemented for `VecDeque<u8>`
    /// let mut sparkle_heart = VecDeque::from(vec![240, 159, 146, 150]);
    /// // We know these bytes are valid, so we can `.unwrap()` or `.expect(...)` here
    /// let compact_str = CompactString::from_utf8_buf(&mut sparkle_heart).expect("valid utf-8");
    ///
    /// assert_eq!(compact_str, "💖");
    /// ```
    ///
    /// ### With invalid/non-UTF8 bytes
    /// ```
    /// # use compact_str::CompactString;
    /// # use std::io;
    ///
    /// // `bytes::Buf` is implemented for `std::io::Cursor<&[u8]>`
    /// let mut invalid = io::Cursor::new(&[0, 159]);
    ///
    /// // The provided buffer is invalid, so trying to create a `CompactString` will fail
    /// assert!(CompactString::from_utf8_buf(&mut invalid).is_err());
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "bytes")))]
    pub fn from_utf8_buf<B: Buf>(buf: &mut B) -> Result<Self, Utf8Error> {
        Repr::from_utf8_buf(buf).map(CompactString)
    }

    /// Converts a buffer of bytes to a [`CompactString`], without checking that the provided buffer
    /// is valid UTF-8.
    ///
    /// # Safety
    /// This function is unsafe because it does not check that the provided bytes are valid UTF-8.
    /// If this constraint is violated, it may cause memory safety issues with futures uses of the
    /// `CompactString`, as the rest of the library assumes that `CompactString`s are valid UTF-8
    ///
    /// # Examples
    /// ```
    /// # use compact_str::CompactString;
    /// # use std::io;
    ///
    /// let word = "hello world";
    /// // `bytes::Buf` is implemented for `std::io::Cursor<&[u8]>`
    /// let mut buffer = io::Cursor::new(word.as_bytes());
    /// let compact_str = unsafe { CompactString::from_utf8_buf_unchecked(&mut buffer) };
    ///
    /// assert_eq!(compact_str, word);
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "bytes")))]
    pub unsafe fn from_utf8_buf_unchecked<B: Buf>(buf: &mut B) -> Self {
        let repr = Repr::from_utf8_buf_unchecked(buf);
        CompactString(repr)
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/bytes/test.rs"]
#[cfg(test)]
mod test;
