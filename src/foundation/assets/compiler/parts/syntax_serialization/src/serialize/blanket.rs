use std::borrow::Cow;

use super::{ESTree, Serializer};

/// [`ESTree`] implementation for a `&` reference to any type that implements `ESTree`.
impl<T> ESTree for &T
where
    T: ESTree + ?Sized,
{
    #[inline(always)]
    fn serialize<S: Serializer>(&self, serializer: S) {
        (**self).serialize(serializer);
    }
}

/// [`ESTree`] implementation for a `&mut` reference to any type that implements `ESTree`.
impl<T> ESTree for &mut T
where
    T: ESTree + ?Sized,
{
    #[inline(always)]
    fn serialize<S: Serializer>(&self, serializer: S) {
        (**self).serialize(serializer);
    }
}

/// [`ESTree`] implementation for `Option<T>`.
impl<T: ESTree> ESTree for Option<T> {
    fn serialize<S: Serializer>(&self, mut serializer: S) {
        if let Some(value) = self {
            value.serialize(serializer);
        } else {
            serializer.buffer_mut().print_str("null");
        }
    }
}

/// [`ESTree`] implementation for a `Cow` wrapping any type that implements `ESTree`.
impl<T> ESTree for Cow<'_, T>
where
    T: ESTree + ToOwned + ?Sized,
{
    #[inline(always)]
    fn serialize<S: Serializer>(&self, serializer: S) {
        (**self).serialize(serializer);
    }
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/syntax_serialization/src/serialize/blanket/tests.rs"]
#[cfg(test)]
mod tests;
