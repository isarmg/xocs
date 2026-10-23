
    use crate::syntax_arena::arena::Arena;

    use super::*;

    #[test]
    fn reserve_does_not_overallocate() {
        let arena = Arena::new();
        {
            let mut v: RawVec<u32, _> = RawVec::new_in(&arena);
            // First `reserve` allocates like `reserve_exact`
            v.reserve(0, 9);
            assert_eq!(9, v.capacity_u32());
        }

        {
            let mut v: RawVec<u32, _> = RawVec::new_in(&arena);
            v.reserve(0, 7);
            assert_eq!(7, v.capacity_u32());
            // 97 if more than double of 7, so `reserve` should work
            // like `reserve_exact`.
            v.reserve(7, 90);
            assert_eq!(97, v.capacity_u32());
        }

        {
            let mut v: RawVec<u32, _> = RawVec::new_in(&arena);
            v.reserve(0, 12);
            assert_eq!(12, v.capacity_u32());
            v.reserve(12, 3);
            // 3 is less than half of 12, so `reserve` must grow
            // exponentially. At the time of writing this test grow
            // factor is 2, so new capacity is 24, however, grow factor
            // of 1.5 is OK too. Hence `>= 18` in assert.
            assert!(v.capacity_u32() >= 12 + 12 / 2);
        }
    }
