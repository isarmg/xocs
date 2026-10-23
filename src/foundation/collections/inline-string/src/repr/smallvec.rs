use smallvec::SmallVec;

use super::{Repr, MAX_SIZE};

impl Repr {
    /// Consumes the [`Repr`] returning a byte vector in a [`SmallVec`]
    ///
    /// Note: both for the inlined case and the heap case, the buffers are re-used
    #[inline]
    pub(crate) fn into_bytes(self) -> SmallVec<[u8; MAX_SIZE]> {
        if let Some(s) = self.as_static_str() {
            SmallVec::from(s.as_bytes())
        } else if self.is_heap_allocated() {
            let string = self.into_string();
            let bytes = string.into_bytes();
            SmallVec::from_vec(bytes)
        } else {
            // SAFETY: We just checked the discriminant to make sure we're an InlineBuffer
            let inline = unsafe { self.into_inline() };
            let (array, length) = inline.into_array();
            SmallVec::from_buf_and_len(array, length)
        }
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/repr/smallvec/tests.rs"]
#[cfg(test)]
mod tests;
