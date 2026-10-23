
    use super::*;

    #[test]
    fn indent_empty() {
        assert_eq!(indent("\n", "  "), "\n");
    }

    #[test]
    #[rustfmt::skip]
    fn indent_nonempty() {
        let text = [
            "  foo\n",
            "bar\n",
            "  baz\n",
        ].join("");
        let expected = [
            "//   foo\n",
            "// bar\n",
            "//   baz\n",
        ].join("");
        assert_eq!(indent(&text, "// "), expected);
    }

    #[test]
    #[rustfmt::skip]
    fn indent_empty_line() {
        let text = [
            "  foo",
            "bar",
            "",
            "  baz",
        ].join("\n");
        let expected = [
            "//   foo",
            "// bar",
            "//",
            "//   baz",
        ].join("\n");
        assert_eq!(indent(&text, "// "), expected);
    }

    #[test]
    fn dedent_empty() {
        assert_eq!(dedent(""), "");
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_multi_line() {
        let x = [
            "    foo",
            "  bar",
            "    baz",
        ].join("\n");
        let y = [
            "  foo",
            "bar",
            "  baz"
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_empty_line() {
        let x = [
            "    foo",
            "  bar",
            "   ",
            "    baz"
        ].join("\n");
        let y = [
            "  foo",
            "bar",
            "",
            "  baz"
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_blank_line() {
        let x = [
            "      foo",
            "",
            "        bar",
            "          foo",
            "          bar",
            "          baz",
        ].join("\n");
        let y = [
            "foo",
            "",
            "  bar",
            "    foo",
            "    bar",
            "    baz",
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_whitespace_line() {
        let x = [
            "      foo",
            " ",
            "        bar",
            "          foo",
            "          bar",
            "          baz",
        ].join("\n");
        let y = [
            "foo",
            "",
            "  bar",
            "    foo",
            "    bar",
            "    baz",
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_mixed_whitespace() {
        let x = [
            "\tfoo",
            "  bar",
        ].join("\n");
        let y = [
            "\tfoo",
            "  bar",
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_tabbed_whitespace() {
        let x = [
            "\t\tfoo",
            "\t\t\tbar",
        ].join("\n");
        let y = [
            "foo",
            "\tbar",
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_mixed_tabbed_whitespace() {
        let x = [
            "\t  \tfoo",
            "\t  \t\tbar",
        ].join("\n");
        let y = [
            "foo",
            "\tbar",
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_mixed_tabbed_whitespace2() {
        let x = [
            "\t  \tfoo",
            "\t    \tbar",
        ].join("\n");
        let y = [
            "\tfoo",
            "  \tbar",
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }

    #[test]
    #[rustfmt::skip]
    fn dedent_preserve_no_terminating_newline() {
        let x = [
            "  foo",
            "    bar",
        ].join("\n");
        let y = [
            "foo",
            "  bar",
        ].join("\n");
        assert_eq!(dedent(&x), y);
    }
