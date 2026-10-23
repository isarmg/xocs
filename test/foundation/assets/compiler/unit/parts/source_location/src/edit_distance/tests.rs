
    use super::*;

    #[test]
    fn test_min_edit_distance() {
        assert_eq!(min_edit_distance("", ""), 0);
        assert_eq!(min_edit_distance("a", "a"), 0);
        assert_eq!(min_edit_distance("abc", "abc"), 0);
        assert_eq!(min_edit_distance("", "abc"), 3);
        assert_eq!(min_edit_distance("abc", ""), 3);
        assert_eq!(min_edit_distance("abc", "def"), 3);
        assert_eq!(min_edit_distance("sitting", "kitten"), 3);
    }

    #[test]
    fn test_best_match() {
        let candidates = vec!["apple", "banana", "cherry"];

        // Exact match returns None
        assert_eq!(best_match("apple", candidates.clone(), 2), None);

        // Close match within threshold
        assert_eq!(best_match("aple", candidates.clone(), 2), Some("apple"));
        assert_eq!(best_match("banan", candidates.clone(), 2), Some("banana"));

        // No match within threshold
        assert_eq!(best_match("xyz", candidates.clone(), 2), None);

        // Empty candidates
        let empty: Vec<&str> = vec![];
        assert_eq!(best_match("test", empty, 2), None);
    }
