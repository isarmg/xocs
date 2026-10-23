
    use alloc::string::String;

    use super::Repr;

    #[test]
    fn short_char_iter() {
        let chars = ['a', 'b', 'c'];
        let repr: Repr = chars.iter().collect();

        assert_eq!(repr.as_str(), "abc");
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn short_char_ref_iter() {
        let chars = ['a', 'b', 'c'];
        let repr: Repr = chars.iter().collect();

        assert_eq!(repr.as_str(), "abc");
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    #[cfg_attr(target_pointer_width = "32", ignore)]
    fn packed_char_iter() {
        let chars = [
            '\u{92f01}',
            '\u{81515}',
            '\u{81515}',
            '\u{81515}',
            '\u{81515}',
            '\u{41515}',
        ];

        let repr: Repr = chars.iter().collect();
        let s: String = chars.iter().collect();

        assert_eq!(repr.as_str(), s.as_str());
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn long_char_iter() {
        let long = "This is supposed to be a really long string";
        let repr: Repr = long.chars().collect();

        assert_eq!(repr.as_str(), "This is supposed to be a really long string");
        assert!(repr.is_heap_allocated());
    }

    #[test]
    fn short_string_iter() {
        let strings = vec!["hello", "world"];
        let repr: Repr = strings.into_iter().collect();

        assert_eq!(repr.as_str(), "helloworld");
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn long_short_string_iter() {
        let strings = vec![
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
            "17", "18", "19", "20",
        ];
        let repr: Repr = strings.into_iter().collect();

        assert_eq!(repr.as_str(), "1234567891011121314151617181920");
        assert!(repr.is_heap_allocated());
    }
