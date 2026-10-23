
    use super::const_str_eq;

    #[test]
    fn equal_and_unequal() {
        // Equal
        assert!(const_str_eq("", ""));
        assert!(const_str_eq("foo", "foo"));
        assert!(const_str_eq("foobar", "foobar"));

        // Not equal to empty string
        assert!(!const_str_eq("", "foo"));
        assert!(!const_str_eq("foo", ""));

        // Differing lengths
        assert!(!const_str_eq("f", "foo"));
        assert!(!const_str_eq("foo", "f"));
        assert!(!const_str_eq("foo", "foobar"));

        // Same length, differing bytes
        assert!(!const_str_eq("foo", "for"));
        assert!(!const_str_eq("foo", "bar"));
    }

    #[test]
    fn usable_in_const_context() {
        const EQ: bool = const_str_eq("foo", "foo");
        const NE: bool = const_str_eq("foo", "bar");
        assert_eq!((EQ, NE), (true, false));
    }
