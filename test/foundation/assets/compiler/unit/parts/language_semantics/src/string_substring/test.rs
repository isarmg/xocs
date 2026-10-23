
    #[test]
    fn test_prototype_last_index_of() {
        use super::StringSubstring;
        assert_eq!("foo".substring(Some(1.0), None), "oo");
        assert_eq!("foo".substring(Some(1.0), Some(2.0)), "o");
        assert_eq!("foo".substring(Some(1.0), Some(1.0)), "");
        assert_eq!("foo".substring(Some(1.0), Some(0.0)), "");
        assert_eq!("foo".substring(Some(0.0), Some(0.0)), "");
        assert_eq!("foo".substring(Some(0.0), Some(1.0)), "f");
        assert_eq!("abc".substring(Some(0.0), Some(2.0)), "ab");
        assert_eq!("abcde".substring(Some(0.0), Some(2.0)), "ab");
    }
