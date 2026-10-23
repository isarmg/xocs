
    #![expect(
        clippy::cast_possible_truncation,
        reason = "test fixtures are much smaller than u32::MAX"
    )]

    use super::*;
    use crate::compiler_errors::source_impls::SpanScanner;
    use crate::source_location::Span;

    type ExpectedLine<'a> = (usize, usize, usize, &'a str);

    #[test]
    fn visual_widths_preserve_grapheme_geometry() {
        let widths =
            GraphicalReportHandler::line_visual_char_width("e\u{301}火").collect::<Vec<_>>();
        assert_eq!(widths, [1, 0, 2]);
    }

    #[test]
    fn get_lines_preserves_line_geometry() {
        const BASE: usize = 10;
        let cases: &[(&str, &[ExpectedLine<'_>])] = &[
            ("", &[]),
            ("abc", &[(5, BASE, 3, "abc")]),
            ("a\nb", &[(5, BASE, 2, "a"), (6, BASE + 2, 1, "b")]),
            ("a\n", &[(5, BASE, 2, "a")]),
            ("\n", &[(5, BASE, 1, "")]),
            ("a\r\nb", &[(5, BASE, 3, "a"), (6, BASE + 3, 1, "b")]),
            ("a\rb", &[(5, BASE, 3, "a\rb")]),
            ("a\r", &[(4, BASE, 2, "a\r")]),
            ("é\n火", &[(5, BASE, 3, "é"), (6, BASE + 3, 3, "火")]),
        ];
        for &(text, expected) in cases {
            let contents = SpanContents::new(
                text.as_bytes(),
                Span::sized(BASE as u32, text.len() as u32),
                4,
                2,
                expected.len(),
            );
            let actual = GraphicalReportHandler::get_lines(&contents)
                .iter()
                .map(|line| (line.number, line.offset, line.length, line.text))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "text={text:?}");
        }
    }

    #[test]
    fn get_lines_preallocates_the_source_window() {
        let source = "before\ntarget\nafter\nrest";
        let mut scanner = SpanScanner::new(source.as_bytes(), 1, 1);
        let contents = scanner.read_span(Span::sized(7, 6)).unwrap();
        let lines = GraphicalReportHandler::get_lines(&contents);

        assert_eq!(lines.len(), 3);
        assert_eq!(lines.capacity(), 3);
    }
