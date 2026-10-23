
    use super::{NonNullConst, NonNullMut};

    // ---------------------------------------------------------------------------------
    // Basic construction and conversion
    // ---------------------------------------------------------------------------------

    #[test]
    fn new_returns_none_for_null() {
        use std::ptr;

        assert!(NonNullConst::<u32>::new(ptr::null()).is_none());

        // SAFETY: No provenance requirements for null pointers
        assert!(unsafe { NonNullMut::<u32>::new(ptr::null_mut()) }.is_none());
    }

    #[test]
    fn new_returns_some_for_non_null() {
        let n = 0u32;
        let p = &raw const n;
        assert!(NonNullConst::new(p).is_some());

        let mut n = 0u32;
        let p = &raw mut n;
        // SAFETY: `p` has write provenance
        assert!(unsafe { NonNullMut::new(p) }.is_some());
    }

    #[test]
    fn cast_const_and_cast_mut_round_trip() {
        let mut n = 0u32;
        let mut_ptr = NonNullMut::from_mut(&mut n);
        let const_ptr = mut_ptr.cast_const();
        assert_eq!(mut_ptr.as_ptr(), const_ptr.as_ptr());

        // SAFETY: `const_ptr` was derived from `mut_ptr`, so it retains write provenance
        // and can be upgraded back to `NonNullMut`
        let round_tripped_mut_ptr = unsafe { const_ptr.cast_mut() };
        assert_eq!(round_tripped_mut_ptr, mut_ptr);
    }

    #[test]
    fn addr_usize_matches_addr() {
        let n = 0u32;
        let ptr = NonNullConst::from_ref(&n);
        assert_eq!(ptr.addr_usize(), ptr.addr().get());
    }

    #[test]
    fn map_addr_usize() {
        let arr = [0u8; 4];
        let first = NonNullConst::from_ref(&arr[0]);
        let second = NonNullConst::from_ref(&arr[1]);
        // SAFETY: Adding 1 byte stays within the `arr` allocation
        let mapped = unsafe { first.map_addr_usize(|a| a + 1) };
        assert_eq!(mapped.as_ptr(), second.as_ptr());

        let mut arr = [0u8; 4];
        let first = NonNullMut::from_slice_data_mut(&mut arr);
        // SAFETY: Adding 1 byte stays within the `arr` allocation
        let mapped = unsafe { first.map_addr_usize(|a| a + 1) };
        assert_eq!(mapped.addr_usize(), first.addr_usize() + 1);
    }

    // ---------------------------------------------------------------------------------
    // Slice/array methods
    // ---------------------------------------------------------------------------------

    #[test]
    fn from_slice_data_points_to_first_element() {
        let arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullConst::from_slice_data(&arr);
        assert_eq!(ptr.as_ptr(), arr.as_ptr());
        // SAFETY: `ptr` points to a valid, initialized `u32`
        assert_eq!(unsafe { ptr.read() }, 1);
    }

    #[test]
    fn from_slice_data_mut_points_to_first_element() {
        let mut arr: [u32; 3] = [1, 2, 3];
        let expected = arr.as_mut_ptr();
        let ptr = NonNullMut::from_slice_data_mut(&mut arr);
        assert_eq!(ptr.as_mut_ptr(), expected);
        // SAFETY: `ptr` points to a valid, initialized `u32`
        assert_eq!(unsafe { ptr.read() }, 1);
    }

    #[test]
    fn as_data_ptr_on_slice_points_to_first_element() {
        let arr: [u32; 3] = [1, 2, 3];
        let slice_ptr = NonNullConst::from_ref(&arr[..]);
        let data_ptr = slice_ptr.as_data_ptr();
        assert_eq!(data_ptr.as_ptr(), arr.as_ptr());

        let mut arr: [u32; 3] = [1, 2, 3];
        let slice_ptr = NonNullMut::from_mut(&mut arr[..]);
        let data_ptr = slice_ptr.as_data_ptr();
        assert_eq!(data_ptr.as_mut_ptr(), arr.as_mut_ptr());
    }

    #[test]
    fn as_data_ptr_on_array_points_to_first_element() {
        let arr: [u32; 3] = [1, 2, 3];
        let arr_ptr = NonNullConst::from_ref(&arr);
        let data_ptr = arr_ptr.as_data_ptr();
        assert_eq!(data_ptr.as_ptr(), arr.as_ptr());

        let mut arr: [u32; 3] = [1, 2, 3];
        let arr_ptr = NonNullMut::from_mut(&mut arr);
        let data_ptr = arr_ptr.as_data_ptr();
        assert_eq!(data_ptr.as_mut_ptr(), arr.as_mut_ptr());
    }

    #[test]
    fn slice_len_and_is_empty() {
        let arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullConst::from_ref(&arr[..]);
        assert_eq!(ptr.len(), 3);
        assert!(!ptr.is_empty());

        let empty: [u32; 0] = [];
        let empty_ptr = NonNullConst::from_ref(&empty[..]);
        assert_eq!(empty_ptr.len(), 0);
        assert!(empty_ptr.is_empty());

        let mut arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullMut::from_mut(&mut arr[..]);
        assert_eq!(ptr.len(), 3);
        assert!(!ptr.is_empty());

        let mut empty: [u32; 0] = [];
        let empty_ptr = NonNullMut::from_mut(&mut empty[..]);
        assert_eq!(empty_ptr.len(), 0);
        assert!(empty_ptr.is_empty());
    }

    #[test]
    fn array_len_and_is_empty() {
        let arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullConst::from_ref(&arr);
        assert_eq!(ptr.len(), 3);
        assert!(!ptr.is_empty());

        let empty: [u32; 0] = [];
        let empty_ptr = NonNullConst::from_ref(&empty);
        assert_eq!(empty_ptr.len(), 0);
        assert!(empty_ptr.is_empty());

        let mut arr: [u32; 3] = [1, 2, 3];
        let ptr = NonNullMut::from_mut(&mut arr);
        assert_eq!(ptr.len(), 3);
        assert!(!ptr.is_empty());

        let mut empty: [u32; 0] = [];
        let empty_ptr = NonNullMut::from_mut(&mut empty);
        assert_eq!(empty_ptr.len(), 0);
        assert!(empty_ptr.is_empty());
    }

    // ---------------------------------------------------------------------------------
    // IntoNonNull: Methods that accept any pointer type
    // ---------------------------------------------------------------------------------

    #[test]
    fn offset_from_accepts_all_pointer_types() {
        let mut arr = [0u32; 4];

        let base_const = NonNullConst::from_ref(&arr[0]);
        let base_mut = NonNullMut::from_mut(&mut arr[0]);
        let base_non_null = base_const.as_non_null();

        let tip_const = NonNullConst::from_ref(&arr[3]);
        let tip_mut = NonNullMut::from_mut(&mut arr[3]);

        // SAFETY: All pointers are in the same `arr` allocation
        unsafe {
            // `NonNullConst::offset_from` accepts all pointer types as origin
            assert_eq!(tip_const.offset_from(base_const), 3);
            assert_eq!(tip_const.offset_from(base_mut), 3);
            assert_eq!(tip_const.offset_from(base_non_null), 3);

            // `NonNullMut::offset_from` accepts all pointer types as origin
            assert_eq!(tip_mut.offset_from(base_const), 3);
            assert_eq!(tip_mut.offset_from(base_mut), 3);
            assert_eq!(tip_mut.offset_from(base_non_null), 3);
        }
    }

    #[test]
    fn offset_from_unsigned_accepts_all_pointer_types() {
        let mut arr = [0u32; 4];

        let base_const = NonNullConst::from_ref(&arr[0]);
        let base_mut = NonNullMut::from_mut(&mut arr[0]);
        let base_non_null = base_const.as_non_null();

        let tip_const = NonNullConst::from_ref(&arr[3]);
        let tip_mut = NonNullMut::from_mut(&mut arr[3]);

        // SAFETY:
        // * All pointers are in the same `arr` allocation.
        // * `tip` is at a higher address than all `base_*` pointers.
        unsafe {
            // `NonNullConst::offset_from_unsigned` accepts all pointer types as origin
            assert_eq!(tip_const.offset_from_unsigned(base_const), 3);
            assert_eq!(tip_const.offset_from_unsigned(base_mut), 3);
            assert_eq!(tip_const.offset_from_unsigned(base_non_null), 3);

            // `NonNullMut::offset_from_unsigned` accepts all pointer types as origin
            assert_eq!(tip_mut.offset_from_unsigned(base_const), 3);
            assert_eq!(tip_mut.offset_from_unsigned(base_mut), 3);
            assert_eq!(tip_mut.offset_from_unsigned(base_non_null), 3);
        }
    }

    #[test]
    fn copy_nonoverlapping_from_accepts_all_pointer_types() {
        // `NonNullConst` as source
        let src: [u32; 3] = [1, 2, 3];
        let src_ptr = NonNullConst::from_slice_data(&src);
        let mut dst = [0u32; 3];
        let dst_ptr = NonNullMut::from_slice_data_mut(&mut dst);
        // SAFETY: `src_ptr` and `dst_ptr` are valid for 3 elements and don't overlap
        unsafe { dst_ptr.copy_nonoverlapping_from(src_ptr, 3) };
        assert_eq!(dst, [1, 2, 3]);

        // `NonNullMut` as source
        let mut src: [u32; 3] = [1, 2, 3];
        let src_ptr = NonNullMut::from_slice_data_mut(&mut src);
        let mut dst = [0u32; 3];
        let dst_ptr = NonNullMut::from_slice_data_mut(&mut dst);
        // SAFETY: `src_ptr` and `dst_ptr` are valid for 3 elements and don't overlap
        unsafe { dst_ptr.copy_nonoverlapping_from(src_ptr, 3) };
        assert_eq!(dst, [1, 2, 3]);

        // `NonNull` as source
        let src: [u32; 3] = [1, 2, 3];
        let src_ptr = NonNullConst::from_slice_data(&src).as_non_null();
        let mut dst = [0u32; 3];
        let dst_ptr = NonNullMut::from_slice_data_mut(&mut dst);
        // SAFETY: `src_ptr` and `dst_ptr` are valid for 3 elements and don't overlap
        unsafe { dst_ptr.copy_nonoverlapping_from(src_ptr, 3) };
        assert_eq!(dst, [1, 2, 3]);
    }

    #[test]
    fn copy_from_allows_overlapping() {
        // `NonNullConst` as source
        let mut arr: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        // Derive both pointers from the same mutable raw pointer to avoid Stacked Borrows violation
        let base = NonNullMut::from_slice_data_mut(&mut arr);
        let src = base.cast_const();
        // SAFETY: `base + 2` is within the `arr` allocation
        let dst = unsafe { base.add(2) };
        // SAFETY: Overlapping copy of 3 elements within the same allocation
        unsafe { dst.copy_from(src, 3) };
        assert_eq!(arr, [1, 2, 1, 2, 3, 6, 7]);

        // `NonNullMut` as source
        let mut arr: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let base = NonNullMut::from_slice_data_mut(&mut arr);
        let src = base;
        // SAFETY: `base + 2` is within the `arr` allocation
        let dst = unsafe { base.add(2) };
        // SAFETY: Overlapping copy of 3 elements within the same allocation
        unsafe { dst.copy_from(src, 3) };
        assert_eq!(arr, [1, 2, 1, 2, 3, 6, 7]);

        // `NonNull` as source
        let mut arr: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let base = NonNullMut::from_slice_data_mut(&mut arr);
        let src = base.as_non_null();
        // SAFETY: `base + 2` is within the `arr` allocation
        let dst = unsafe { base.add(2) };
        // SAFETY: Overlapping copy of 3 elements within the same allocation
        unsafe { dst.copy_from(src, 3) };
        assert_eq!(arr, [1, 2, 1, 2, 3, 6, 7]);
    }

    // ---------------------------------------------------------------------------------
    // PartialEq / Eq / PartialOrd / Ord / Hash
    // ---------------------------------------------------------------------------------

    #[test]
    fn eq() {
        let arr = [0u32; 2];
        let a = NonNullConst::from_ref(&arr[0]);
        let b = NonNullConst::from_ref(&arr[0]);
        let c = NonNullConst::from_ref(&arr[1]);

        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut arr = [0u32; 2];
        let a = NonNullMut::from_slice_data_mut(&mut arr);
        let b = a;
        // SAFETY: `a + 1` is within the allocation
        let c = unsafe { a.add(1) };

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn ord() {
        use std::cmp::Ordering;

        let arr = [0u32; 3];
        let a = NonNullConst::from_ref(&arr[0]);
        let b = NonNullConst::from_ref(&arr[1]);
        let c = NonNullConst::from_ref(&arr[2]);

        assert!(a < b);
        assert!(b > a);
        assert!(a < c);
        assert!(c > a);
        assert!(b < c);
        assert!(c > b);
        assert!(a <= a);
        assert!(a >= a);
        assert_eq!(a.cmp(&a), Ordering::Equal);
        assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));

        let mut arr = [0u32; 3];
        let base = NonNullMut::from_slice_data_mut(&mut arr);
        let a = base;
        // SAFETY: Offsets are within the allocation
        let (b, c) = unsafe { (base.add(1), base.add(2)) };

        assert!(a < b);
        assert!(b > a);
        assert!(a < c);
        assert!(c > a);
        assert!(b < c);
        assert!(c > b);
        assert!(a <= a);
        assert!(a >= a);
        assert_eq!(a.cmp(&a), Ordering::Equal);
        assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));
    }

    #[test]
    fn hash() {
        use std::hash::{DefaultHasher, Hash, Hasher};

        fn hash_of<T: Hash>(val: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            val.hash(&mut hasher);
            hasher.finish()
        }

        let arr = [0u32; 2];
        let a = NonNullConst::from_ref(&arr[0]);
        let b = NonNullConst::from_ref(&arr[0]);
        let c = NonNullConst::from_ref(&arr[1]);

        // Equal pointers must have equal hashes
        assert_eq!(hash_of(&a), hash_of(&b));
        // Different pointers should (almost certainly) have different hashes
        assert_ne!(hash_of(&a), hash_of(&c));

        let mut arr = [0u32; 2];
        let a = NonNullMut::from_slice_data_mut(&mut arr);
        // SAFETY: Offsets are within the allocation
        let (b, c) = unsafe { (a.add(0), a.add(1)) };

        // Equal pointers must have equal hashes
        assert_eq!(hash_of(&a), hash_of(&b));
        // Different pointers should (almost certainly) have different hashes
        assert_ne!(hash_of(&a), hash_of(&c));

        // `NonNullConst` and `NonNullMut` pointing to same address have same hash
        let mut x = 0u32;
        let const_ptr = NonNullConst::from_ref(&x);
        let mut_ptr = NonNullMut::from_mut(&mut x);
        assert_eq!(hash_of(&const_ptr), hash_of(&mut_ptr));
    }

    // ---------------------------------------------------------------------------------
    // Send / Sync
    // ---------------------------------------------------------------------------------

    /// Both types wrap `NonNull`, so inherit its `!Send` / `!Sync`.
    /// Types containing them have to opt in to `Send` / `Sync` explicitly.
    #[test]
    fn not_send_or_sync() {
        use crate::compiler_collections::types::implements;

        assert!(implements!(NonNullConst<u32>: !Send));
        assert!(implements!(NonNullConst<u32>: !Sync));
        assert!(implements!(NonNullMut<u32>: !Send));
        assert!(implements!(NonNullMut<u32>: !Sync));
    }
