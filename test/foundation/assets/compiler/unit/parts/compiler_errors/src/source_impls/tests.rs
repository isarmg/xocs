
    use super::*;

    #[test]
    fn lf_prefix_fast_path_matches_generic_path() {
        let input = b"zero\none\n\ntwo\nthree\n";
        for cut in 0..=input.len() {
            let fast = PrefixScan::new(input, cut, 1);
            let generic = PrefixScan::new(input, cut, 2);
            assert_eq!(fast.line_count, generic.line_count, "cut={cut}");
            assert_eq!(fast.current_line_start, generic.current_line_start, "cut={cut}");
            assert_eq!(fast.leading.first(), generic.leading.last(), "cut={cut}");
            assert_eq!(fast.leading.start_line, fast.line_count.saturating_sub(1), "cut={cut}");
        }
    }

    #[test]
    fn basic() {
        let src = String::from("foo\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(0, 4), 0, 0).read().unwrap();
        assert_eq!("foo\n", from_utf8(contents.data()).unwrap());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn shifted() {
        let src = String::from("foobar");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(3, 3), 1, 1).read().unwrap();
        assert_eq!("foobar", from_utf8(contents.data()).unwrap());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn middle() {
        let src = String::from("foo\nbar\nbaz\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(4, 4), 0, 0).read().unwrap();
        assert_eq!("bar\n", from_utf8(contents.data()).unwrap());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn middle_of_line() {
        let src = String::from("foo\nbarbar\nbaz\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(7, 4), 0, 0).read().unwrap();
        assert_eq!("bar\n", from_utf8(contents.data()).unwrap());
        assert_eq!(1, contents.line());
        assert_eq!(3, contents.column());
    }

    #[test]
    fn with_crlf() {
        let src = String::from("foo\r\nbar\r\nbaz\r\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(5, 5), 0, 0).read().unwrap();
        assert_eq!("bar\r\n", from_utf8(contents.data()).unwrap());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn with_context() {
        let src = String::from("xxx\nfoo\nbar\nbaz\n\nyyy\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(8, 3), 1, 1).read().unwrap();
        assert_eq!("foo\nbar\nbaz\n", from_utf8(contents.data()).unwrap());
        assert_eq!(1, contents.line());
        assert_eq!(0, contents.column());
    }

    #[test]
    fn multiline_with_context() {
        let src = String::from("aaa\nxxx\n\nfoo\nbar\nbaz\n\nyyy\nbbb\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(9, 11), 1, 1).read().unwrap();
        assert_eq!("\nfoo\nbar\nbaz\n\n", from_utf8(contents.data()).unwrap());
        assert_eq!(2, contents.line());
        assert_eq!(0, contents.column());
        let span = Span::sized(8, 14);
        assert_eq!(&span, contents.span());
    }

    #[test]
    fn zero_length_span_just_past_eof() {
        // Used to panic with a slice out-of-range instead of returning an
        // error (found by differential fuzzing).
        let src = String::from("a");
        assert!(SpanReader::from_span(src.as_bytes(), Span::sized(2, 0), 0, 0).read().is_none());
        let src = String::new();
        assert!(SpanReader::from_span(src.as_bytes(), Span::sized(1, 0), 0, 0).read().is_none());
    }

    #[test]
    fn multiline_with_context_line_start() {
        let src = String::from("one\ntwo\n\nthree\nfour\nfive\n\nsix\nseven\n");
        let contents =
            SpanReader::from_span(src.as_bytes(), Span::sized(2, 0), 2, 2).read().unwrap();
        assert_eq!("one\ntwo\n\n", from_utf8(contents.data()).unwrap());
        assert_eq!(0, contents.line());
        assert_eq!(0, contents.column());
        let span = Span::sized(0, 9);
        assert_eq!(&span, contents.span());
    }
