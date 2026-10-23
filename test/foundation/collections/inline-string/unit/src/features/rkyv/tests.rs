
    use alloc::string::String;

    use rkyv::string::ArchivedString;
    use rkyv::{rancor, Archive};
    use test_strategy::proptest;

    use crate::CompactString;

    #[cfg_attr(miri, ignore)] // https://github.com/rust-lang/unsafe-code-guidelines/issues/134
    #[test]
    fn test_roundtrip() {
        const VALUE: &str = "Hello, 🌍!";

        let bytes_compact = rkyv::to_bytes::<rancor::Error>(&CompactString::from(VALUE)).unwrap();
        let bytes_control = rkyv::to_bytes::<rancor::Error>(&String::from(VALUE)).unwrap();
        assert_eq!(&*bytes_compact, &*bytes_control);

        let archived = rkyv::access::<ArchivedString, rancor::Error>(&bytes_compact).unwrap();
        let compact = rkyv::deserialize::<CompactString, rancor::Error>(archived).unwrap();
        let control = rkyv::deserialize::<String, rancor::Error>(archived).unwrap();
        assert_eq!(archived, VALUE);
        assert_eq!(compact, VALUE);
        assert_eq!(control, VALUE);

        let archived = rkyv::access::<ArchivedString, rancor::Error>(&bytes_compact).unwrap();
        let compact = rkyv::deserialize::<CompactString, rancor::Error>(archived).unwrap();
        let control = rkyv::deserialize::<String, rancor::Error>(archived).unwrap();
        assert_eq!(archived, VALUE);
        assert_eq!(compact, VALUE);
        assert_eq!(control, VALUE);
    }

    #[cfg_attr(miri, ignore)]
    #[proptest]
    fn proptest_roundtrip(s: String) {
        let bytes_compact = rkyv::to_bytes::<rancor::Error>(&CompactString::from(&s)).unwrap();
        let bytes_control = rkyv::to_bytes::<rancor::Error>(&s).unwrap();
        assert_eq!(&*bytes_compact, &*bytes_control);

        let archived =
            rkyv::access::<<CompactString as Archive>::Archived, rancor::Error>(&bytes_compact)
                .unwrap();
        let compact = rkyv::deserialize::<CompactString, rancor::Error>(archived).unwrap();
        let control = rkyv::deserialize::<String, rancor::Error>(archived).unwrap();
        assert_eq!(archived, &s);
        assert_eq!(compact, s);
        assert_eq!(control, s);

        let archived =
            rkyv::access::<<String as Archive>::Archived, rancor::Error>(&bytes_compact).unwrap();
        let compact = rkyv::deserialize::<CompactString, rancor::Error>(archived).unwrap();
        let control = rkyv::deserialize::<String, rancor::Error>(archived).unwrap();
        assert_eq!(archived, &s);
        assert_eq!(compact, s);
        assert_eq!(control, s);
    }
