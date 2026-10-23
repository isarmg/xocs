
    use super::*;

    #[test]
    fn unfill_simple() {
        let (text, options) = unfill("foo\nbar");
        assert_eq!(text, "foo bar");
        assert_eq!(options.width, 3);
        assert_eq!(options.line_ending, LineEnding::LF);
    }

    #[test]
    fn unfill_no_new_line() {
        let (text, options) = unfill("foo bar");
        assert_eq!(text, "foo bar");
        assert_eq!(options.width, 7);
        assert_eq!(options.line_ending, LineEnding::LF);
    }

    #[test]
    fn unfill_simple_crlf() {
        let (text, options) = unfill("foo\r\nbar");
        assert_eq!(text, "foo bar");
        assert_eq!(options.width, 3);
        assert_eq!(options.line_ending, LineEnding::CRLF);
    }

    #[test]
    fn unfill_mixed_new_lines() {
        let (text, options) = unfill("foo\r\nbar\nbaz");
        assert_eq!(text, "foo bar baz");
        assert_eq!(options.width, 3);
        assert_eq!(options.line_ending, LineEnding::LF);
    }

    #[test]
    fn test_unfill_consecutive_different_prefix() {
        let (text, options) = unfill("foo\n*\n/");
        assert_eq!(text, "foo * /");
        assert_eq!(options.width, 3);
        assert_eq!(options.line_ending, LineEnding::LF);
    }

    #[test]
    fn unfill_trailing_newlines() {
        let (text, options) = unfill("foo\nbar\n\n\n");
        assert_eq!(text, "foo bar\n");
        assert_eq!(options.width, 3);
    }

    #[test]
    fn unfill_mixed_trailing_newlines() {
        let (text, options) = unfill("foo\r\nbar\n\r\n\n");
        assert_eq!(text, "foo bar\n");
        assert_eq!(options.width, 3);
        assert_eq!(options.line_ending, LineEnding::LF);
    }

    #[test]
    fn unfill_trailing_crlf() {
        let (text, options) = unfill("foo bar\r\n");
        assert_eq!(text, "foo bar\r\n");
        assert_eq!(options.width, 7);
        assert_eq!(options.line_ending, LineEnding::CRLF);
    }

    #[test]
    fn unfill_initial_indent() {
        let (text, options) = unfill("  foo\nbar\nbaz");
        assert_eq!(text, "foo bar baz");
        assert_eq!(options.width, 5);
        assert_eq!(options.initial_indent, "  ");
    }

    #[test]
    fn unfill_differing_indents() {
        let (text, options) = unfill("  foo\n    bar\n  baz");
        assert_eq!(text, "foo   bar baz");
        assert_eq!(options.width, 7);
        assert_eq!(options.initial_indent, "  ");
        assert_eq!(options.subsequent_indent, "  ");
    }

    #[test]
    fn unfill_list_item() {
        let (text, options) = unfill("* foo\n  bar\n  baz");
        assert_eq!(text, "foo bar baz");
        assert_eq!(options.width, 5);
        assert_eq!(options.initial_indent, "* ");
        assert_eq!(options.subsequent_indent, "  ");
    }

    #[test]
    fn unfill_multiple_char_prefix() {
        let (text, options) = unfill("    // foo bar\n    // baz\n    // quux");
        assert_eq!(text, "foo bar baz quux");
        assert_eq!(options.width, 14);
        assert_eq!(options.initial_indent, "    // ");
        assert_eq!(options.subsequent_indent, "    // ");
    }

    #[test]
    fn unfill_block_quote() {
        let (text, options) = unfill("> foo\n> bar\n> baz");
        assert_eq!(text, "foo bar baz");
        assert_eq!(options.width, 5);
        assert_eq!(options.initial_indent, "> ");
        assert_eq!(options.subsequent_indent, "> ");
    }

    #[test]
    fn unfill_only_prefixes_issue_466() {
        // Test that we don't crash if the first line has only prefix
        // chars *and* the second line is shorter than the first line.
        let (text, options) = unfill("######\nfoo");
        assert_eq!(text, " foo");
        assert_eq!(options.width, 6);
        assert_eq!(options.initial_indent, "######");
        assert_eq!(options.subsequent_indent, "");
    }

    #[test]
    fn unfill_trailing_newlines_issue_466() {
        // Test that we don't crash on a '\r' following a string of
        // '\n'. The problem was that we removed both kinds of
        // characters in one code path, but not in the other.
        let (text, options) = unfill("foo\n##\n\n\r");
        // The \n\n changes subsequent_indent to "".
        assert_eq!(text, "foo ## \r");
        assert_eq!(options.width, 3);
        assert_eq!(options.initial_indent, "");
        assert_eq!(options.subsequent_indent, "");
    }

    #[test]
    fn unfill_whitespace() {
        assert_eq!(unfill("foo   bar").0, "foo   bar");
    }

    #[test]
    fn refill_convert_lf_to_crlf() {
        let options = Options::new(5).line_ending(LineEnding::CRLF);
        assert_eq!(refill("foo\nbar\n", options), "foo\r\nbar\r\n",);
    }

    #[test]
    fn refill_convert_crlf_to_lf() {
        let options = Options::new(5).line_ending(LineEnding::LF);
        assert_eq!(refill("foo\r\nbar\r\n", options), "foo\nbar\n",);
    }

    #[test]
    fn refill_convert_mixed_newlines() {
        let options = Options::new(5).line_ending(LineEnding::CRLF);
        assert_eq!(refill("foo\r\nbar\n", options), "foo\r\nbar\r\n",);
    }

    #[test]
    fn refill_defaults_to_lf() {
        assert_eq!(refill("foo bar baz", 5), "foo\nbar\nbaz");
    }
