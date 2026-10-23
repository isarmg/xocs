
    use compact_str::CompactString;

    use super::CompactStr;

    #[test]
    fn test_compactstr_eq() {
        let foo = CompactStr::new("foo");
        assert_eq!(foo, "foo");
        assert_eq!(&foo, "foo");
        assert_eq!("foo", foo);
        assert_eq!("foo", &foo);
        assert_eq!(foo.into_compact_string(), CompactString::new("foo"));
    }

    #[test]
    fn test_format_compact_str() {
        assert_eq!(format_compact_str!("foo{}bar", 123), "foo123bar");
    }
