
    use super::*;
    use crate::WrapAlgorithm;

    #[test]
    fn fill_simple() {
        assert_eq!(fill("foo bar baz", 10), "foo bar\nbaz");
    }

    #[test]
    fn fill_unicode_boundary() {
        // https://github.com/mgeisler/textwrap/issues/390
        fill("\u{1b}!Ͽ", 10);
    }

    #[test]
    fn non_breaking_space() {
        let options = Options::new(5).break_words(false);
        assert_eq!(fill("foo bar baz", &options), "foo bar baz");
    }

    #[test]
    fn non_breaking_hyphen() {
        let options = Options::new(5).break_words(false);
        assert_eq!(fill("foo‑bar‑baz", &options), "foo‑bar‑baz");
    }

    #[test]
    fn fill_preserves_line_breaks_trims_whitespace() {
        assert_eq!(fill("  ", 80), "");
        assert_eq!(fill("  \n  ", 80), "\n");
        assert_eq!(fill("  \n \n  \n ", 80), "\n\n\n");
    }

    #[test]
    fn preserve_line_breaks() {
        assert_eq!(fill("", 80), "");
        assert_eq!(fill("\n", 80), "\n");
        assert_eq!(fill("\n\n\n", 80), "\n\n\n");
        assert_eq!(fill("test\n", 80), "test\n");
        assert_eq!(fill("test\n\na\n\n", 80), "test\n\na\n\n");
        assert_eq!(
            fill(
                "1 3 5 7\n1 3 5 7",
                Options::new(7).wrap_algorithm(WrapAlgorithm::FirstFit)
            ),
            "1 3 5 7\n1 3 5 7"
        );
        assert_eq!(
            fill(
                "1 3 5 7\n1 3 5 7",
                Options::new(5).wrap_algorithm(WrapAlgorithm::FirstFit)
            ),
            "1 3 5\n7\n1 3 5\n7"
        );
    }

    #[test]
    fn break_words_line_breaks() {
        assert_eq!(fill("ab\ncdefghijkl", 5), "ab\ncdefg\nhijkl");
        assert_eq!(fill("abcdefgh\nijkl", 5), "abcde\nfgh\nijkl");
    }

    #[test]
    fn break_words_empty_lines() {
        assert_eq!(
            fill("foo\nbar", &Options::new(2).break_words(false)),
            "foo\nbar"
        );
    }

    #[test]
    fn fill_inplace_empty() {
        let mut text = String::from("");
        fill_inplace(&mut text, 80);
        assert_eq!(text, "");
    }

    #[test]
    fn fill_inplace_simple() {
        let mut text = String::from("foo bar baz");
        fill_inplace(&mut text, 10);
        assert_eq!(text, "foo bar\nbaz");
    }

    #[test]
    fn fill_inplace_multiple_lines() {
        let mut text = String::from("Some text to wrap over multiple lines");
        fill_inplace(&mut text, 12);
        assert_eq!(text, "Some text to\nwrap over\nmultiple\nlines");
    }

    #[test]
    fn fill_inplace_long_word() {
        let mut text = String::from("Internationalization is hard");
        fill_inplace(&mut text, 10);
        assert_eq!(text, "Internationalization\nis hard");
    }

    #[test]
    fn fill_inplace_no_hyphen_splitting() {
        let mut text = String::from("A well-chosen example");
        fill_inplace(&mut text, 10);
        assert_eq!(text, "A\nwell-chosen\nexample");
    }

    #[test]
    fn fill_inplace_newlines() {
        let mut text = String::from("foo bar\n\nbaz\n\n\n");
        fill_inplace(&mut text, 10);
        assert_eq!(text, "foo bar\n\nbaz\n\n\n");
    }

    #[test]
    fn fill_inplace_newlines_reset_line_width() {
        let mut text = String::from("1 3 5\n1 3 5 7 9\n1 3 5 7 9 1 3");
        fill_inplace(&mut text, 10);
        assert_eq!(text, "1 3 5\n1 3 5 7 9\n1 3 5 7 9\n1 3");
    }

    #[test]
    fn fill_inplace_leading_whitespace() {
        let mut text = String::from("  foo bar baz");
        fill_inplace(&mut text, 10);
        assert_eq!(text, "  foo bar\nbaz");
    }

    #[test]
    fn fill_inplace_trailing_whitespace() {
        let mut text = String::from("foo bar baz  ");
        fill_inplace(&mut text, 10);
        assert_eq!(text, "foo bar\nbaz  ");
    }

    #[test]
    fn fill_inplace_interior_whitespace() {
        // To avoid an unwanted indentation of "baz", it is important
        // to replace the final ' ' with '\n'.
        let mut text = String::from("foo  bar    baz");
        fill_inplace(&mut text, 10);
        assert_eq!(text, "foo  bar   \nbaz");
    }
