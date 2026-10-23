
    #[test]
    fn test_string_index_of() {
        use super::StringIndexOf;

        assert_eq!("test test test".index_of(Some("t"), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(1.0)), 3);
        assert_eq!("test test test".index_of(Some("t"), Some(4.0)), 5);
        assert_eq!("test test test".index_of(Some("t"), Some(4.1)), 5);
        assert_eq!("test test test".index_of(Some("t"), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(-1.1)), 0);
        assert_eq!("test test test".index_of(Some("t"), Some(-1_073_741_825.0)), 0);
        assert_eq!("test test test".index_of(Some("e"), Some(0.0)), 1);
        assert_eq!("test test test".index_of(Some("s"), Some(0.0)), 2);
        assert_eq!("test test test".index_of(Some("test"), Some(4.0)), 5);
        assert_eq!("test test test".index_of(Some("test"), Some(5.0)), 5);
        assert_eq!("test test test".index_of(Some("test"), Some(6.0)), 10);
        assert_eq!("test test test".index_of(Some("test"), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some("test"), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some("not found"), Some(-1.0)), -1);
        assert_eq!("test test test".index_of(Some("test"), Some(-1.0)), 0);
        assert_eq!("test test test".index_of(Some("test"), Some(-1_073_741_825.0)), 0);
        assert_eq!("test test test".index_of(Some("test"), Some(0.0)), 0);
        assert_eq!("test test test".index_of(Some("notpresent"), Some(0.0)), -1);
        assert_eq!("undefined".index_of(None, Some(0.0)), 0);
        assert_eq!("test test test".index_of(None, Some(0.0)), -1);
    }
