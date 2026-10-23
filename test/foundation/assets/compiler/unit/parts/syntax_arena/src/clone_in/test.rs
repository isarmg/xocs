
    use super::{Allocator, CloneIn, HashMap, Vec};

    #[test]
    fn clone_in_boxed_slice() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut original = Vec::from_iter_in([1, 2, 3], &allocator).into_boxed_slice();

        let cloned = original.clone_in(allocator);
        let cloned2 = original.clone_in_with_semantic_ids(allocator);
        original[1] = 4;

        assert_eq!(original.as_ref(), &[1, 4, 3]);
        assert_eq!(cloned.as_ref(), &[1, 2, 3]);
        assert_eq!(cloned2.as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn clone_in_empty_boxed_slice() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        // Exercises the zero-sized `alloc_layout` path in `clone_slice_in`
        let original = Vec::<u32>::new_in(&allocator).into_boxed_slice();

        let cloned = original.clone_in(allocator);
        let cloned2 = original.clone_in_with_semantic_ids(allocator);

        assert_eq!(cloned.as_ref(), &[] as &[u32]);
        assert_eq!(cloned2.as_ref(), &[] as &[u32]);
    }

    #[test]
    fn clone_in_vec() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut original = Vec::with_capacity_in(8, &allocator);
        original.extend_from_slice(&[1, 2, 3]);

        let cloned = original.clone_in(allocator);
        let cloned2 = original.clone_in_with_semantic_ids(allocator);
        original[1] = 4;

        assert_eq!(original.as_slice(), &[1, 4, 3]);
        assert_eq!(cloned.as_slice(), &[1, 2, 3]);
        assert_eq!(cloned.capacity(), 3);
        assert_eq!(cloned2.as_slice(), &[1, 2, 3]);
        assert_eq!(cloned2.capacity(), 3);
    }

    #[test]
    fn clone_in_empty_vec() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        // Exercises the `slice.is_empty()` short-circuit to `Vec::new_in`
        let original = Vec::<u32>::new_in(&allocator);

        let cloned = original.clone_in(allocator);
        let cloned2 = original.clone_in_with_semantic_ids(allocator);

        assert_eq!(cloned.as_slice(), &[] as &[u32]);
        assert_eq!(cloned.capacity(), 0);
        assert_eq!(cloned2.as_slice(), &[] as &[u32]);
        assert_eq!(cloned2.capacity(), 0);
    }

    #[test]
    fn clone_in_hash_map() {
        let allocator = Allocator::default();

        let mut original: HashMap<'_, &str, &str> = HashMap::with_capacity_in(8, &allocator);
        original.extend(&[("x", "xx"), ("y", "yy"), ("z", "zz")]);

        let cloned = original.clone_in(&allocator);
        let cloned2 = original.clone_in_with_semantic_ids(&allocator);
        *original.get_mut("y").unwrap() = "changed";

        let mut original_as_vec = original.iter().collect::<std::vec::Vec<_>>();
        original_as_vec.sort_unstable();
        assert_eq!(original_as_vec, &[(&"x", &"xx"), (&"y", &"changed"), (&"z", &"zz")]);

        assert_eq!(cloned.capacity(), 3);
        let mut cloned_as_vec = cloned.iter().collect::<std::vec::Vec<_>>();
        cloned_as_vec.sort_unstable();
        assert_eq!(cloned_as_vec, &[(&"x", &"xx"), (&"y", &"yy"), (&"z", &"zz")]);

        assert_eq!(cloned2.capacity(), 3);
        let mut cloned2_as_vec = cloned2.iter().collect::<std::vec::Vec<_>>();
        cloned2_as_vec.sort_unstable();
        assert_eq!(cloned2_as_vec, &[(&"x", &"xx"), (&"y", &"yy"), (&"z", &"zz")]);
    }
