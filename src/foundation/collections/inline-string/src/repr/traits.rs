use core::hint::unreachable_unchecked;

use super::Repr;
use crate::ToCompactStringError;

const FALSE: Repr = Repr::const_new("false");
const TRUE: Repr = Repr::const_new("true");

/// Defines how to _efficiently_ create a [`Repr`] from `self`
pub(crate) trait IntoRepr {
    fn into_repr(self) -> Result<Repr, ToCompactStringError>;
}

impl IntoRepr for f32 {
    #[inline]
    fn into_repr(self) -> Result<Repr, ToCompactStringError> {
        let mut buf = zmij::Buffer::new();
        let s = buf.format(self);
        Ok(Repr::new(s)?)
    }
}

impl IntoRepr for f64 {
    #[inline]
    fn into_repr(self) -> Result<Repr, ToCompactStringError> {
        let mut buf = zmij::Buffer::new();
        let s = buf.format(self);
        Ok(Repr::new(s)?)
    }
}

impl IntoRepr for bool {
    #[inline]
    fn into_repr(self) -> Result<Repr, ToCompactStringError> {
        if self {
            Ok(TRUE)
        } else {
            Ok(FALSE)
        }
    }
}

impl IntoRepr for char {
    #[inline]
    fn into_repr(self) -> Result<Repr, ToCompactStringError> {
        let mut buf = [0_u8; 4];
        let s = self.encode_utf8(&mut buf);

        // This match is just a hint for the compiler.
        match s.len() {
            1..=4 => (),
            // SAFETY: a UTF-8 character is 1 to 4 bytes.
            _ => unsafe { unreachable_unchecked() },
        }

        Ok(Repr::new(s)?)
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/repr/traits/tests.rs"]
#[cfg(test)]
mod tests;
