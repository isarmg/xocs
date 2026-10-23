
    use alloc::string::String;

    use test_strategy::proptest;

    use crate::repr::{HEAP_MASK, MAX_SIZE};
    use crate::CompactString;

    fn assert_roundtrip(s: &str) {
        let bytes_compact = borsh::to_vec(&CompactString::from(s)).unwrap();
        let bytes_control = borsh::to_vec(&String::from(s)).unwrap();
        assert_eq!(&*bytes_compact, &*bytes_control);

        let compact: CompactString = borsh::from_slice(&bytes_compact).unwrap();
        let control: String = borsh::from_slice(&bytes_control).unwrap();
        assert_eq!(compact, s);
        assert_eq!(control, s);
    }

    #[test]
    fn test_deserialize_invalid_utf8() {
        let bytes = borsh::to_vec(&[HEAP_MASK; MAX_SIZE] as &[u8]).unwrap();
        borsh::from_slice::<CompactString>(&bytes).unwrap_err();
    }

    #[test]
    fn test_deserialize_unexpected_eof() {
        let s = core::str::from_utf8(&[b'a'; 55]).unwrap();
        let mut bytes = borsh::to_vec(s).unwrap();
        bytes.pop();
        borsh::from_slice::<CompactString>(&bytes).unwrap_err();
    }

    #[test]
    fn test_roundtrip() {
        assert_roundtrip("Hello, 🌍!");
    }

    #[cfg_attr(miri, ignore)]
    #[proptest]
    fn proptest_roundtrip(s: String) {
        assert_roundtrip(&s);
    }
