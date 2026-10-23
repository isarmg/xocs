use core::str::Utf8Error;

use bytes::Buf;

use super::{Repr, MAX_SIZE};
use crate::UnwrapWithMsg;

impl Repr {
    /// Converts a [`Buf`] of bytes to a [`Repr`], checking that the provided bytes are valid UTF-8
    pub(crate) fn from_utf8_buf<B: Buf>(buf: &mut B) -> Result<Self, Utf8Error> {
        // SAFETY: We check below to make sure the provided buffer is valid UTF-8
        let (repr, bytes_written) = unsafe { Self::collect_buf(buf) };

        // Check to make sure the provided bytes are valid UTF-8, return the Repr if they are!
        match core::str::from_utf8(&repr.as_slice()[..bytes_written]) {
            Ok(_) => Ok(repr),
            Err(e) => Err(e),
        }
    }

    /// Converts a [`Buf`] of bytes to a [`Repr`], without checking for valid UTF-8
    ///
    /// # Safety
    /// * The provided buffer must be valid UTF-8
    pub(crate) unsafe fn from_utf8_buf_unchecked<B: Buf>(buf: &mut B) -> Self {
        let (repr, _bytes_written) = Self::collect_buf(buf);
        repr
    }

    /// Collects the bytes from a [`Buf`] into a [`Repr`]
    ///
    /// # Safety
    /// * The caller must guarantee that `buf` is valid UTF-8
    unsafe fn collect_buf<B: Buf>(buf: &mut B) -> (Self, usize) {
        // Get an empty Repr we can write into
        let mut repr = super::EMPTY;
        let mut bytes_written = 0;

        debug_assert_eq!(repr.len(), bytes_written);

        while buf.has_remaining() {
            let chunk = buf.chunk();
            let chunk_len = chunk.len();

            // There's an edge case where the final byte of this buffer == `HEAP_MASK`, which is
            // invalid UTF-8, but would result in us creating an inline variant, that identifies as
            // a heap variant. If a user ever tried to reference the data at all, we'd incorrectly
            // try and read data from an invalid memory address, causing undefined behavior.
            if bytes_written < MAX_SIZE && bytes_written + chunk_len == MAX_SIZE {
                let last_byte = chunk[chunk_len - 1];
                // If we hit the edge case, reserve additional space to make the repr becomes heap
                // allocated, which prevents us from writing this last byte inline
                if last_byte >= 0b11000000 {
                    repr.reserve(MAX_SIZE + 1).unwrap_with_msg();
                }
            }

            // reserve at least enough space to fit this chunk
            repr.reserve(chunk_len).unwrap_with_msg();

            // Write the chunk into the spare capacity without first creating a reference to the
            // uninitialized memory.
            //
            // SAFETY:
            // * `reserve` guarantees at least `bytes_written + chunk_len` bytes of capacity.
            // * `chunk` is valid for reads of `chunk_len` bytes.
            // * `chunk` belongs to `buf`, so it cannot overlap `repr`.
            repr.as_mut_ptr()
                .add(bytes_written)
                .copy_from_nonoverlapping(chunk.as_ptr(), chunk_len);

            // Set the length of the Repr
            // SAFETY: We just wrote an additional `chunk_len` bytes into the Repr
            bytes_written += chunk_len;
            repr.set_len(bytes_written);

            // advance the pointer of the buffer
            buf.advance(chunk_len);
        }

        (repr, bytes_written)
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/repr/bytes/test.rs"]
#[cfg(test)]
mod test;
