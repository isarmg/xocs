
    #[test]
    fn test_prototype_last_index_of() {
        use super::StringLastIndexOf;
        assert_eq!("test test test".last_index_of(Some("test"), Some(15.0)), 10);
        assert_eq!("test test test".last_index_of(Some("test"), Some(14.0)), 10);
        assert_eq!("test test test".last_index_of(Some("test"), Some(10.0)), 10);
        assert_eq!("test test test".last_index_of(Some("test"), Some(9.0)), 5);
        assert_eq!("test test test".last_index_of(Some("test"), Some(6.0)), 5);
        assert_eq!("test test test".last_index_of(Some("test"), Some(5.0)), 5);
        assert_eq!("test test test".last_index_of(Some("test"), Some(4.0)), 0);
        assert_eq!("test test test".last_index_of(Some("test"), Some(0.0)), 0);
        assert_eq!("test test test".last_index_of(Some("notpresent"), Some(0.0)), -1);
        assert_eq!("undefined".last_index_of(None, None), 0);
        assert_eq!("test test test".last_index_of(None, None), -1);
        assert_eq!("abcdef".last_index_of(Some("b"), None), 1);
    }
