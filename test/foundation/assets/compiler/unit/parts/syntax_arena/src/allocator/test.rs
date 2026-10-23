
    use super::Allocator;

    #[test]
    fn test_api() {
        let mut allocator = Allocator::default();
        {
            let array = allocator.alloc([123; 10]);
            assert_eq!(array, &[123; 10]);
            let str = allocator.alloc_str("hello");
            assert_eq!(str, "hello");
        }
        allocator.reset();
    }

    #[test]
    fn string_from_array_len_1() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array(["hello"]);
        assert_eq!(s, "hello");
    }

    #[test]
    fn string_from_array_len_2() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array(["hello", "world!"]);
        assert_eq!(s, "helloworld!");
    }

    #[test]
    fn string_from_array_len_3() {
        let hello = "hello";
        let world = std::string::String::from("world");
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array([hello, &world, "!"]);
        assert_eq!(s, "helloworld!");
    }

    #[test]
    fn string_from_empty_array() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array([]);
        assert_eq!(s, "");
    }

    #[test]
    fn string_from_array_of_empty_strs() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array(["", "", ""]);
        assert_eq!(s, "");
    }

    #[test]
    fn string_from_array_containing_some_empty_strs() {
        let allocator = Allocator::default();
        let s = allocator.alloc_concat_strs_array(["", "hello", ""]);
        assert_eq!(s, "hello");
    }
