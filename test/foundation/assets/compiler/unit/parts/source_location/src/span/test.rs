
    use super::Span;

    #[test]
    fn test_size() {
        let s = Span::sized(0, 5);
        assert_eq!(s.size(), 5);
        assert!(!s.is_empty());

        let s = Span::sized(5, 0);
        assert_eq!(s.size(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_hash() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        fn hash<T: Hash>(value: T) -> u64 {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        }

        let first_hash = hash(Span::new(1, 5));
        let second_hash = hash(Span::new(1, 5));
        assert_eq!(first_hash, second_hash);

        // On 64-bit platforms, check hash is equivalent to `u64`
        #[cfg(target_pointer_width = "64")]
        {
            let u64_equivalent: u64 =
                if cfg!(target_endian = "little") { 1 + (5 << 32) } else { (1 << 32) + 5 };
            let u64_hash = hash(u64_equivalent);
            assert_eq!(first_hash, u64_hash);
        }

        // On 32-bit platforms, check `_align` field does not alter hash
        #[cfg(not(target_pointer_width = "64"))]
        {
            #[derive(Hash)]
            #[repr(C)]
            struct PlainSpan {
                start: u32,
                end: u32,
            }

            let plain_hash = hash(PlainSpan { start: 1, end: 5 });
            assert_eq!(first_hash, plain_hash);
        }
    }

    #[test]
    fn test_eq() {
        assert_eq!(Span::new(0, 0), Span::new(0, 0));
        assert_eq!(Span::new(0, 1), Span::new(0, 1));
        assert_eq!(Span::new(1, 5), Span::new(1, 5));

        assert_ne!(Span::new(0, 0), Span::new(0, 1));
        assert_ne!(Span::new(1, 5), Span::new(0, 5));
        assert_ne!(Span::new(1, 5), Span::new(2, 5));
        assert_ne!(Span::new(1, 5), Span::new(1, 4));
        assert_ne!(Span::new(1, 5), Span::new(1, 6));
    }

    #[test]
    fn test_ordering_less() {
        assert!(Span::new(0, 0) < Span::new(0, 1));
        assert!(Span::new(0, 3) < Span::new(2, 5));
    }

    #[test]
    fn test_ordering_greater() {
        assert!(Span::new(0, 1) > Span::new(0, 0));
        assert!(Span::new(2, 5) > Span::new(0, 3));
    }

    #[test]
    fn test_contains() {
        let span = Span::new(5, 10);

        assert!(span.contains_inclusive(span));
        assert!(span.contains_inclusive(Span::new(5, 5)));
        assert!(span.contains_inclusive(Span::new(10, 10)));
        assert!(span.contains_inclusive(Span::new(6, 9)));

        assert!(!span.contains_inclusive(Span::new(0, 0)));
        assert!(!span.contains_inclusive(Span::new(4, 10)));
        assert!(!span.contains_inclusive(Span::new(5, 11)));
        assert!(!span.contains_inclusive(Span::new(4, 11)));
    }

    #[test]
    fn test_expand() {
        let span = Span::new(3, 5);
        assert_eq!(span.expand(0), Span::new(3, 5));
        assert_eq!(span.expand(1), Span::new(2, 6));
        // start and end cannot be expanded past `0` and `u32::MAX`, respectively
        assert_eq!(span.expand(5), Span::new(0, 10));
    }

    #[test]
    fn test_shrink() {
        let span = Span::new(4, 8);
        assert_eq!(span.shrink(0), Span::new(4, 8));
        assert_eq!(span.shrink(1), Span::new(5, 7));
        // can be equal
        assert_eq!(span.shrink(2), Span::new(6, 6));
    }

    #[test]
    #[should_panic(expected = "Cannot shrink span past zero length")]
    fn test_shrink_past_start() {
        let span = Span::new(5, 10);
        let _ = span.shrink(5);
    }

    #[test]
    fn test_move_left() {
        let span = Span::new(5, 10);
        assert_eq!(span.move_left(1), Span::new(4, 9));
        assert_eq!(span.move_left(2), Span::new(3, 8));
        assert_eq!(span.move_left(5), Span::new(0, 5));
    }

    #[test]
    #[should_panic(expected = "Cannot move span past zero length")]
    fn test_move_past_start() {
        let span = Span::new(5, 10);
        let _ = span.move_left(6);
    }

    #[test]
    fn test_move_right() {
        let span: Span = Span::new(5, 10);
        assert_eq!(span.move_right(1), Span::new(6, 11));
        assert_eq!(span.move_right(2), Span::new(7, 12));
        assert_eq!(
            span.move_right(u32::MAX.saturating_sub(10)),
            Span::new(u32::MAX.saturating_sub(5), u32::MAX)
        );
    }

    #[test]
    #[should_panic(expected = "Cannot move span past `u32::MAX` length")]
    fn test_move_past_end() {
        let span = Span::new(u32::MAX.saturating_sub(2), u32::MAX.saturating_sub(1));
        let _ = span.move_right(2);
    }
