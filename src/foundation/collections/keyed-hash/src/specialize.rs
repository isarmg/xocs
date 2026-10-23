use crate::RandomState;
use core::hash::{BuildHasher, Hash, Hasher};

/// Provides a way to get an optimized hasher for a given data type.
/// Rather than using a Hasher generically which can hash any value, this provides a way to get a specialized hash
/// for a specific type. So this may be faster for primitive types.
pub(crate) trait CallHasher {
    fn get_hash<H: Hash + ?Sized>(value: &H, random_state: &RandomState) -> u64;
}

impl<T> CallHasher for T
where
    T: Hash + ?Sized,
{
    #[inline]
    fn get_hash<H: Hash + ?Sized>(value: &H, random_state: &RandomState) -> u64 {
        let mut hasher = random_state.build_hasher();
        value.hash(&mut hasher);
        hasher.finish()
    }
}

#[path = "../../../../../test/foundation/collections/keyed-hash/unit/src/specialize/test.rs"]
#[cfg(test)]
mod test;
