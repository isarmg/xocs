use dragonbox_ecma::Buffer as DragonboxBuffer;
use itoa::Buffer as ItoaBuffer;

use super::{ESTree, Serializer};

/// [`ESTree`] implementation for `bool`.
impl ESTree for bool {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        serializer.buffer_mut().print_str(if *self { "true" } else { "false" });
    }
}

/// [`ESTree`] implementation for `f64`.
impl ESTree for f64 {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        if self.is_finite() {
            let mut buffer = DragonboxBuffer::new();
            let s = buffer.format_finite(*self);
            serializer.buffer_mut().print_str(s);
        } else if self.is_nan() {
            // Serialize `NAN` as `null`
            // TODO: Throw an error? Use a sentinel value?
            serializer.buffer_mut().print_str("null");
        } else if *self == f64::INFINITY {
            // Serialize `INFINITY` as `1e+400. `JSON.parse` deserializes this as `Infinity`.
            serializer.buffer_mut().print_str("1e+400");
        } else {
            // Serialize `-INFINITY` as `-1e+400`. `JSON.parse` deserializes this as `-Infinity`.
            serializer.buffer_mut().print_str("-1e+400");
        }
    }
}

/// [`ESTree`] implementations for integer types.
macro_rules! impl_integer {
    ($ty:ident) => {
        impl ESTree for $ty {
            fn serialize<S: Serializer>(&self, mut serializer: S) {
                let mut buffer = ItoaBuffer::new();
                let s = buffer.format(*self);
                serializer.buffer_mut().print_str(s);
            }
        }
    };
}

impl_integer!(u8);
impl_integer!(u16);
impl_integer!(u32);
impl_integer!(u64);
impl_integer!(u128);
impl_integer!(usize);
impl_integer!(i8);
impl_integer!(i16);
impl_integer!(i32);
impl_integer!(i64);
impl_integer!(i128);
impl_integer!(isize);

/// [`ESTree`] implementation for `()`.
impl ESTree for () {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        serializer.buffer_mut().print_str("null");
    }
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/syntax_serialization/src/serialize/primitives/tests.rs"]
#[cfg(test)]
mod tests;
