
    use super::*;
    use crate::convert::Convert;
    use crate::operations::aesenc;
    use crate::RandomState;
    use std::hash::{BuildHasher, Hasher};
    #[test]
    fn test_sanity() {
        let mut hasher = RandomState::with_seeds(1, 2, 3, 4).build_hasher();
        hasher.write_u64(0);
        let h1 = hasher.finish();
        hasher.write(&[1, 0, 0, 0, 0, 0, 0, 0]);
        let h2 = hasher.finish();
        assert_ne!(h1, h2);
    }

    #[cfg(feature = "runtime-rng")]
    #[test]
    fn test_builder() {
        use std::collections::HashMap;
        use std::hash::BuildHasherDefault;

        let mut map = HashMap::<u32, u64, BuildHasherDefault<AHasher>>::default();
        map.insert(1, 3);
    }

    #[cfg(feature = "runtime-rng")]
    #[test]
    fn test_default() {
        let hasher_a = AHasher::default();
        let a_enc: [u64; 2] = hasher_a.enc.convert();
        let a_sum: [u64; 2] = hasher_a.sum.convert();
        assert_ne!(0, a_enc[0]);
        assert_ne!(0, a_enc[1]);
        assert_ne!(0, a_sum[0]);
        assert_ne!(0, a_sum[1]);
        assert_ne!(a_enc[0], a_enc[1]);
        assert_ne!(a_sum[0], a_sum[1]);
        assert_ne!(a_enc[0], a_sum[0]);
        assert_ne!(a_enc[1], a_sum[1]);
        let hasher_b = AHasher::default();
        let b_enc: [u64; 2] = hasher_b.enc.convert();
        let b_sum: [u64; 2] = hasher_b.sum.convert();
        assert_eq!(a_enc[0], b_enc[0]);
        assert_eq!(a_enc[1], b_enc[1]);
        assert_eq!(a_sum[0], b_sum[0]);
        assert_eq!(a_sum[1], b_sum[1]);
    }

    #[test]
    fn test_hash() {
        let mut result: [u64; 2] = [0x6c62272e07bb0142, 0x62b821756295c58d];
        let value: [u64; 2] = [1 << 32, 0xFEDCBA9876543210];
        result = aesenc(value.convert(), result.convert()).convert();
        result = aesenc(result.convert(), result.convert()).convert();
        let mut result2: [u64; 2] = [0x6c62272e07bb0142, 0x62b821756295c58d];
        let value2: [u64; 2] = [1, 0xFEDCBA9876543210];
        result2 = aesenc(value2.convert(), result2.convert()).convert();
        result2 = aesenc(result2.convert(), result.convert()).convert();
        let result: [u8; 16] = result.convert();
        let result2: [u8; 16] = result2.convert();
        assert_ne!(hex::encode(result), hex::encode(result2));
    }
