
    #[cfg(feature = "std")]
    use std::io::Cursor;

    use test_case::test_case;

    use super::Repr;

    #[test_case(""; "empty")]
    #[test_case("hello world"; "short")]
    #[test_case("hello, this is a long string which should be heap allocated"; "long")]
    fn test_from_utf8_buf(word: &'static str) {
        let mut buf = Cursor::new(word.as_bytes());
        let repr = Repr::from_utf8_buf(&mut buf).unwrap();

        assert_eq!(repr.as_str(), word);
        assert_eq!(repr.len(), word.len());
    }

    #[test]
    fn test_from_utf8_packed() {
        cfg_if::cfg_if! {
            if #[cfg(target_pointer_width = "64")] {
                let packed = "this string is 24 chars!";
            } else if #[cfg(target_pointer_width = "32")] {
                let packed = "i am 12 char";
            } else {
                compile_error!("unsupported architecture!")
            }
        }
        let mut buf = Cursor::new(packed.as_bytes());

        let repr = Repr::from_utf8_buf(&mut buf).unwrap();
        assert_eq!(repr.as_str(), packed);

        // This repr should __not__ be heap allocated
        assert!(!repr.is_heap_allocated());
    }

    #[test]
    fn test_fuzz_panic() {
        let bytes = &[
            255, 255, 255, 255, 255, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 1, 12, 0, 0, 96,
        ];
        let mut buf: Cursor<&[u8]> = Cursor::new(bytes);

        assert!(Repr::from_utf8_buf(&mut buf).is_err());
    }

    #[test]
    fn test_valid_repr_but_invalid_utf8() {
        let bytes = &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192,
        ];
        let mut buf: Cursor<&[u8]> = Cursor::new(bytes);

        assert!(Repr::from_utf8_buf(&mut buf).is_err());
    }

    #[test]
    fn test_fake_heap_variant() {
        let bytes = &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
        ];
        let mut buf: Cursor<&[u8]> = Cursor::new(bytes);

        assert!(Repr::from_utf8_buf(&mut buf).is_err());
    }

    #[test]
    fn test_from_non_contiguous() {
        let data = [
            211, 247, 211, 247, 121, 135, 151, 255, 126, 205, 255, 204, 211, 51, 51, 0, 52, 55,
            247, 204, 45, 37, 44, 210, 132, 50, 206, 121, 135, 151, 255, 126, 205, 255, 204, 211,
            51, 51, 0, 52, 55, 247, 204, 45, 44, 210, 132, 50, 206, 51,
        ];
        let (front, back) = data.split_at(data.len() / 2 + 1);
        let mut queue = alloc::collections::VecDeque::with_capacity(data.len());

        // create a non-contiguous slice of memory in queue
        front.iter().copied().for_each(|x| queue.push_back(x));
        back.iter().copied().for_each(|x| queue.push_front(x));

        // make sure it's non-contiguous
        let (a, b) = queue.as_slices();
        assert!(data.is_empty() || !a.is_empty());
        assert!(data.is_empty() || !b.is_empty());

        assert_eq!(data.len(), queue.len());
        assert!(Repr::from_utf8_buf(&mut queue).is_err());
    }

    #[test]
    #[should_panic(expected = "Utf8Error")]
    fn test_invalid_utf8() {
        let invalid = &[0, 159];
        let mut buf: Cursor<&[u8]> = Cursor::new(invalid);

        Repr::from_utf8_buf(&mut buf).unwrap();
    }
