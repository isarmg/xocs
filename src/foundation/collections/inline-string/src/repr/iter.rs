//! Implementations of the [`FromIterator`] trait to make building [`Repr`]s more ergonomic

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;

use super::{InlineBuffer, Repr, EMPTY, MAX_SIZE};
use crate::{CompactString, UnwrapWithMsg};

impl FromIterator<char> for Repr {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let (lower_bound, _) = iter.size_hint();
        let mut this = match Repr::with_capacity(lower_bound) {
            Ok(this) => this,
            Err(_) => EMPTY, // Ignore the error and hope that the lower_bound is incorrect.
        };

        for c in iter {
            this.push_str(c.encode_utf8(&mut [0; 4]));
        }
        this
    }
}

impl<'a> FromIterator<&'a char> for Repr {
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        iter.into_iter().copied().collect()
    }
}

fn from_as_ref_str_iterator<S, I>(mut iter: I) -> Repr
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
    String: core::iter::Extend<S>,
    String: FromIterator<S>,
{
    // Note: We don't check the lower bound here like we do in the character iterator because it's
    // possible for the iterator to be full of empty strings! In which case checking the lower bound
    // could cause us to heap allocate when there's no need.

    // Continuously pull strings from the iterator
    let mut curr_len = 0;
    let mut inline_buf = InlineBuffer::new_const("");
    while let Some(s) = iter.next() {
        let str_slice = s.as_ref();
        let bytes_len = str_slice.len();

        // this new string is too large to fit into our inline buffer, so heap allocate the rest
        if bytes_len + curr_len > MAX_SIZE {
            let (min_remaining, _) = iter.size_hint();
            let mut string = String::with_capacity(bytes_len + curr_len + min_remaining);

            // push existing strings onto the heap
            // SAFETY: `inline_buf` has been filled with `&str`s which are valid UTF-8
            string.push_str(unsafe { core::str::from_utf8_unchecked(&inline_buf.0[..curr_len]) });
            // push current string onto the heap
            string.push_str(str_slice);
            // extend heap with remaining strings
            string.extend(iter);

            return Repr::from_string(string, true).unwrap_with_msg();
        }

        // write the current string into a slice of the unoccupied space
        inline_buf.0[curr_len..][..bytes_len].copy_from_slice(str_slice.as_bytes());
        curr_len += bytes_len;
    }

    // SAFETY: Everything we just pushed onto the buffer is a `str` which is valid UTF-8
    unsafe { inline_buf.set_len(curr_len) }

    Repr::from_inline(inline_buf)
}

impl<'a> FromIterator<&'a str> for Repr {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        from_as_ref_str_iterator(iter.into_iter())
    }
}

impl FromIterator<Box<str>> for Repr {
    fn from_iter<T: IntoIterator<Item = Box<str>>>(iter: T) -> Self {
        from_as_ref_str_iterator(iter.into_iter())
    }
}

impl FromIterator<String> for Repr {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        from_as_ref_str_iterator(iter.into_iter())
    }
}

impl FromIterator<CompactString> for Repr {
    fn from_iter<T: IntoIterator<Item = CompactString>>(iter: T) -> Self {
        from_as_ref_str_iterator(iter.into_iter())
    }
}

impl<'a> FromIterator<Cow<'a, str>> for Repr {
    fn from_iter<T: IntoIterator<Item = Cow<'a, str>>>(iter: T) -> Self {
        from_as_ref_str_iterator(iter.into_iter())
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/repr/iter/tests.rs"]
#[cfg(test)]
mod tests;
