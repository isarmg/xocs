
    #[test]
    fn test_macros() {
        assert_eq!(format_compact!("2"), "2");
        assert_eq!(format_compact!("{}", 2), "2");

        assert!(!format_compact!("2").is_heap_allocated());
        assert!(!format_compact!("{}", 2).is_heap_allocated());
    }
