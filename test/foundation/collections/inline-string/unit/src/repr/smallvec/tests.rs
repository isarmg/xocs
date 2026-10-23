
    use test_case::test_case;

    use crate::CompactString;

    #[test_case("" ; "empty")]
    #[test_case("abc" ; "short")]
    #[test_case("I am a long string 😊😊😊😊😊" ; "long")]
    fn proptest_roundtrip(s: &'static str) {
        let og_compact = CompactString::from(s);
        assert_eq!(og_compact, s);

        let bytes = og_compact.into_bytes();

        let ex_compact = CompactString::from_utf8(bytes).unwrap();
        assert_eq!(ex_compact, s);

        // test `StaticStr` variant
        let og_compact = CompactString::const_new(s);
        assert_eq!(og_compact, s);

        let bytes = og_compact.into_bytes();

        let ex_compact = CompactString::from_utf8(bytes).unwrap();
        assert_eq!(ex_compact, s);
    }
