
    use super::*;
    use alloc::vec;

    #[derive(Debug, PartialEq)]
    struct Word(f64);

    #[rustfmt::skip]
    impl Fragment for Word {
        fn width(&self) -> f64 { self.0 }
        fn whitespace_width(&self) -> f64 { 1.0 }
        fn penalty_width(&self) -> f64 { 0.0 }
    }

    #[test]
    fn wrap_fragments_with_infinite_widths() {
        let words = vec![Word(f64::INFINITY)];
        assert_eq!(
            wrap_optimal_fit(&words, &[0.0], &Penalties::default()),
            Err(OverflowError)
        );
    }

    #[test]
    fn wrap_fragments_with_huge_widths() {
        let words = vec![Word(1e200), Word(1e250), Word(1e300)];
        assert_eq!(
            wrap_optimal_fit(&words, &[1e300], &Penalties::default()),
            Err(OverflowError)
        );
    }

    #[test]
    fn wrap_fragments_with_large_widths() {
        // The gaps will be of the sizes between 1e25 and 1e75. This
        // makes the `gap * gap` cost fit comfortably in a f64.
        let words = vec![Word(1e25), Word(1e50), Word(1e75)];
        assert_eq!(
            wrap_optimal_fit(&words, &[1e100], &Penalties::default()),
            Ok(vec![&vec![Word(1e25), Word(1e50), Word(1e75)][..]])
        );
    }
