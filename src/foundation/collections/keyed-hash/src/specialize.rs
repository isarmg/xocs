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

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    /// Tests that some non-trivial transformation takes place.
    #[test]
    pub fn test_input_processed() {
        let build_hasher = RandomState::with_seeds(2, 2, 2, 2);
        assert_ne!(0, u64::get_hash(&0, &build_hasher));
        assert_ne!(1, u64::get_hash(&0, &build_hasher));
        assert_ne!(2, u64::get_hash(&0, &build_hasher));
        assert_ne!(3, u64::get_hash(&0, &build_hasher));
        assert_ne!(4, u64::get_hash(&0, &build_hasher));
        assert_ne!(5, u64::get_hash(&0, &build_hasher));

        assert_ne!(0, u64::get_hash(&1, &build_hasher));
        assert_ne!(1, u64::get_hash(&1, &build_hasher));
        assert_ne!(2, u64::get_hash(&1, &build_hasher));
        assert_ne!(3, u64::get_hash(&1, &build_hasher));
        assert_ne!(4, u64::get_hash(&1, &build_hasher));
        assert_ne!(5, u64::get_hash(&1, &build_hasher));

        let xored = u64::get_hash(&0, &build_hasher) ^ u64::get_hash(&1, &build_hasher);
        assert_ne!(0, xored);
        assert_ne!(1, xored);
        assert_ne!(2, xored);
        assert_ne!(3, xored);
        assert_ne!(4, xored);
        assert_ne!(5, xored);
    }

    #[test]
    pub fn test_ref_independent() {
        let build_hasher = RandomState::with_seeds(1, 2, 3, 4);
        assert_eq!(
            u8::get_hash(&&1, &build_hasher),
            u8::get_hash(&1, &build_hasher)
        );
        assert_eq!(
            u16::get_hash(&&2, &build_hasher),
            u16::get_hash(&2, &build_hasher)
        );
        assert_eq!(
            u32::get_hash(&&3, &build_hasher),
            u32::get_hash(&3, &build_hasher)
        );
        assert_eq!(
            u64::get_hash(&&4, &build_hasher),
            u64::get_hash(&4, &build_hasher)
        );
        assert_eq!(
            u128::get_hash(&&5, &build_hasher),
            u128::get_hash(&5, &build_hasher)
        );
        assert_eq!(
            str::get_hash(&"test", &build_hasher),
            str::get_hash("test", &build_hasher)
        );
        assert_eq!(
            str::get_hash(&"test", &build_hasher),
            String::get_hash(&"test".to_string(), &build_hasher)
        );
        let build_hasher = RandomState::with_seeds(10, 20, 30, 40);
        assert_eq!(
            u8::get_hash(&&&1, &build_hasher),
            u8::get_hash(&1, &build_hasher)
        );
        assert_eq!(
            u16::get_hash(&&&2, &build_hasher),
            u16::get_hash(&2, &build_hasher)
        );
        assert_eq!(
            u32::get_hash(&&&3, &build_hasher),
            u32::get_hash(&3, &build_hasher)
        );
        assert_eq!(
            u64::get_hash(&&&4, &build_hasher),
            u64::get_hash(&4, &build_hasher)
        );
        assert_eq!(
            u128::get_hash(&&&5, &build_hasher),
            u128::get_hash(&5, &build_hasher)
        );
        assert_eq!(
            str::get_hash(&&"test", &build_hasher),
            str::get_hash("test", &build_hasher)
        );
        assert_eq!(
            str::get_hash(&&"test", &build_hasher),
            String::get_hash(&"test".to_string(), &build_hasher)
        );
    }
}
