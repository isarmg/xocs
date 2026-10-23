//! [`StringExt`] trait - extension methods for [`String`].

use crate::compiler_collections::assert_unchecked;

/// Extension trait for [`String`].
pub trait StringExt {
    /// Append a [`char`] to the end of this `String`, without checking that the `String`
    /// has sufficient spare capacity to hold it.
    ///
    /// This is a faster version of [`String::push`] which skips the check for whether the
    /// `String`'s backing buffer needs to grow (and the cold grow/reallocation branch).
    ///
    /// # SAFETY
    /// `String` must have spare capacity for `ch`
    /// i.e. `self.len() + ch.len_utf8() <= self.capacity()`.
    unsafe fn push_unchecked(&mut self, ch: char);

    /// Append a `&str` to the end of this `String`, without checking that the `String`
    /// has sufficient spare capacity to hold it.
    ///
    /// This is a faster version of [`String::push_str`] which skips the check for whether the
    /// `String`'s backing buffer needs to grow (and the cold grow/reallocation branch).
    ///
    /// # SAFETY
    /// `String` must have spare capacity for `s`
    /// i.e. `self.len() + s.len() <= self.capacity()`.
    unsafe fn push_str_unchecked(&mut self, s: &str);
}

impl StringExt for String {
    #[inline]
    unsafe fn push_unchecked(&mut self, ch: char) {
        // SAFETY: Caller guarantees `self.len() + ch.len_utf8() <= self.capacity()`.
        //
        // The 2nd assertion is the promise to the compiler that no growth is needed. It is phrased as
        // `additional <= capacity - len` (rather than `len + additional <= capacity`) to match
        // the exact shape of the grow check inside `String::push` (`additional > capacity.wrapping_sub(len)`).
        // LLVM does not derive the difference form from the sum form, so only this phrasing removes
        // the capacity check and grow branch.
        // The 1st assertion guarantees `capacity - len` does not wrap.
        unsafe {
            assert_unchecked!(self.len() <= self.capacity());
            assert_unchecked!(ch.len_utf8() <= self.capacity() - self.len());
        }

        self.push(ch);
    }

    #[inline]
    unsafe fn push_str_unchecked(&mut self, s: &str) {
        // SAFETY: Caller guarantees `self.len() + s.len() <= self.capacity()`.
        // See `push_unchecked` above for why the assertions are phrased this way.
        unsafe {
            assert_unchecked!(self.len() <= self.capacity());
            assert_unchecked!(s.len() <= self.capacity() - self.len());
        }

        self.push_str(s);
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_collections/src/string_ext/tests.rs"]
#[cfg(test)]
mod tests;
