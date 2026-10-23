
    #[test]
    fn implemented() {
        assert!(implements!(u32: Copy));
        assert!(implements!(String: Clone));
        assert!(implements!(Vec<u32>: IntoIterator));
        assert!(implements!(u32: PartialEq<u32>));
        assert!(implements!(u16: From<u8>));
        assert!(implements!(&'static str: Copy));
        // Unsized types can be queried
        assert!(implements!(str: Send));
    }

    #[test]
    fn not_implemented() {
        assert!(!implements!(String: Copy));
        assert!(!implements!(u16: From<u32>));
        assert!(!implements!(std::rc::Rc<u32>: Send));
        assert!(!implements!(std::cell::Cell<u32>: Sync));
        assert!(!implements!(str: Sized));
    }

    #[test]
    fn negated() {
        assert!(implements!(String: !Copy));
        assert!(implements!(u16: !From<u32>));
        assert!(implements!(std::rc::Rc<u32>: !Send));
        assert!(implements!(std::cell::Cell<u32>: !Sync));
        assert!(implements!(str: !Sized));

        // Negating a trait which *is* implemented answers `false`
        assert!(!implements!(u32: !Copy));
    }

    #[test]
    fn const_usable() {
        const { assert!(implements!(u32: Copy)) };
        const { assert!(!implements!(String: Copy)) };
        const { assert!(implements!(String: !Copy)) };
    }
