
    use super::Span;

    /// Tests from [`Span`] docs, since rustdoc test runner is disabled
    #[test]
    fn doctest() {
        // 1
        let text = "foo bar baz";
        let span = Span::new(4, 7);
        assert_eq!(&text[span], "bar");

        // 2
        let a = Span::new(5, 10); // Start and end offsets
        let b = Span::sized(5, 5); // Start offset and size
        assert_eq!(a, b);

        // 3
        let s = Span::new(5, 10);
        assert_eq!(s.shrink(2), Span::new(7, 8));
        assert_eq!(s.shrink(2), s.shrink_left(2).shrink_right(2));

        assert_eq!(s.expand(5), Span::new(0, 15));
        assert_eq!(s.expand(5), s.expand_left(5).expand_right(5));
    }
