
    use super::*;

    #[derive(PartialEq, Debug)]
    enum HashCall {
        Bytes(Vec<u8>),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
        U128(u128),
        Usize(usize),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        I128(i128),
        Isize(isize),
        // Ideally we'd handle `write_length_prefix` and `write_str` as well, but they are unstable.
    }

    #[derive(PartialEq, Debug)]
    struct TestHasher {
        calls: Vec<HashCall>,
    }

    impl Hasher for TestHasher {
        fn finish(&self) -> u64 {
            panic!("only used for tests");
        }
        fn write(&mut self, bytes: &[u8]) {
            self.calls.push(HashCall::Bytes(bytes.to_vec()));
        }
        fn write_u8(&mut self, i: u8) {
            self.calls.push(HashCall::U8(i));
        }
        fn write_u16(&mut self, i: u16) {
            self.calls.push(HashCall::U16(i));
        }
        fn write_u32(&mut self, i: u32) {
            self.calls.push(HashCall::U32(i));
        }
        fn write_u64(&mut self, i: u64) {
            self.calls.push(HashCall::U64(i));
        }
        fn write_u128(&mut self, i: u128) {
            self.calls.push(HashCall::U128(i));
        }
        fn write_usize(&mut self, i: usize) {
            self.calls.push(HashCall::Usize(i));
        }
        fn write_i8(&mut self, i: i8) {
            self.calls.push(HashCall::I8(i));
        }
        fn write_i16(&mut self, i: i16) {
            self.calls.push(HashCall::I16(i));
        }
        fn write_i32(&mut self, i: i32) {
            self.calls.push(HashCall::I32(i));
        }
        fn write_i64(&mut self, i: i64) {
            self.calls.push(HashCall::I64(i));
        }
        fn write_i128(&mut self, i: i128) {
            self.calls.push(HashCall::I128(i));
        }
        fn write_isize(&mut self, i: isize) {
            self.calls.push(HashCall::Isize(i));
        }
    }

    fn test_hash<T: PhfHash>(x: T) -> Vec<HashCall> {
        let mut state = TestHasher { calls: Vec::new() };
        x.phf_hash(&mut state);
        state.calls
    }

    #[test]
    fn byte_slices_are_hashed_efficiently() {
        assert_eq!(
            test_hash(&[1u8, 2, 3]),
            [HashCall::U64(3), HashCall::Bytes([1, 2, 3].to_vec())]
        );
        assert_eq!(
            test_hash(&[1i8, 2, 3]),
            [HashCall::U64(3), HashCall::Bytes([1, 2, 3].to_vec())]
        );
        assert_eq!(
            test_hash(&[false, true]),
            [HashCall::U64(2), HashCall::Bytes([0, 1].to_vec())]
        );
    }

    #[test]
    fn slices_and_arrays_are_hashed_consistently() {
        assert_eq!(test_hash(&[1u8, 2, 3]), test_hash(&[1u8, 2, 3][..]));
        assert_eq!(test_hash(&[1u16, 2, 3]), test_hash(&[1u16, 2, 3][..]));
    }

    #[test]
    fn array_reference_borrow_is_generic() {
        fn assert_borrow<K, B: ?Sized>()
        where
            K: PhfBorrow<B>,
        {
        }

        assert_borrow::<&[u32; 2], [u32; 2]>();
        assert_borrow::<&[bool; 2], [bool; 2]>();
        assert_borrow::<&[char; 2], [char; 2]>();
    }

    #[test]
    fn tuple_eq_allows_shorter_reference_lifetimes() {
        fn assert_ref_tuple<'a>(key: &(&'a str, &'a str)) -> bool
        where
            (&'static str, &'static str): PhfEq<(&'a str, &'a str)>,
        {
            ("a", "b").phf_eq(key)
        }

        fn assert_mixed_tuple<'a>(key: &(u32, &'a str)) -> bool
        where
            (u32, &'static str): PhfEq<(u32, &'a str)>,
        {
            (1, "a").phf_eq(key)
        }

        let a = String::from("a");
        let b = String::from("b");
        assert!(assert_ref_tuple(&(a.as_str(), b.as_str())));
        assert!(assert_mixed_tuple(&(1, a.as_str())));
    }

    #[test]
    fn variable_width_slice_elements_are_delimited() {
        assert_ne!(test_hash(&["ab", "c"]), test_hash(&["a", "bc"]));

        let key = 0;
        let left = hash(&["ab", "c"], &key);
        let right = hash(&["a", "bc"], &key);
        assert!(
            (left.g, left.f1, left.f2) != (right.g, right.f1, right.f2),
            "different string arrays must not produce identical PHF hashes"
        );
    }
