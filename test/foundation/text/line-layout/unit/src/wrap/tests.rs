
    use super::*;
    use crate::{WordSeparator, WordSplitter, WrapAlgorithm};
    #[cfg(not(feature = "std"))]
    use alloc::{format, vec};

    #[cfg(feature = "hyphenation")]
    use hyphenation::{Language, Load, Standard};

    #[test]
    fn no_wrap() {
        assert_eq!(wrap("foo", 10), vec!["foo"]);
    }

    #[test]
    fn wrap_simple() {
        assert_eq!(wrap("foo bar baz", 5), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn to_be_or_not() {
        assert_eq!(
            wrap(
                "To be, or not to be, that is the question.",
                Options::new(10).wrap_algorithm(WrapAlgorithm::FirstFit)
            ),
            vec!["To be, or", "not to be,", "that is", "the", "question."]
        );
    }

    #[test]
    fn multiple_words_on_first_line() {
        assert_eq!(wrap("foo bar baz", 10), vec!["foo bar", "baz"]);
    }

    #[test]
    fn long_word() {
        assert_eq!(wrap("foo", 0), vec!["f", "o", "o"]);
    }

    #[test]
    fn long_words() {
        assert_eq!(wrap("foo bar", 0), vec!["f", "o", "o", "b", "a", "r"]);
    }

    #[test]
    fn max_width() {
        assert_eq!(wrap("foo bar", usize::MAX), vec!["foo bar"]);

        let text = "Hello there! This is some English text. \
                    It should not be wrapped given the extents below.";
        assert_eq!(wrap(text, usize::MAX), vec![text]);
    }

    #[test]
    fn leading_whitespace() {
        assert_eq!(wrap("  foo bar", 6), vec!["  foo", "bar"]);
    }

    #[test]
    fn leading_whitespace_empty_first_line() {
        // If there is no space for the first word, the first line
        // will be empty. This is because the string is split into
        // words like [" ", "foobar ", "baz"], which puts "foobar " on
        // the second line. We never output trailing whitespace
        assert_eq!(wrap(" foobar baz", 6), vec!["", "foobar", "baz"]);
    }

    #[test]
    fn trailing_whitespace() {
        // Whitespace is only significant inside a line. After a line
        // gets too long and is broken, the first word starts in
        // column zero and is not indented.
        assert_eq!(wrap("foo     bar     baz  ", 5), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn preserve_trailing_whitespace() {
        let options = Options::new(5).preserve_trailing_space(true);
        // Preserve the whitespace at the end of the line, allowing it
        // to overflow if it is just whitespaces.
        assert_eq!(
            wrap("foo     bar     baz  ", &options),
            vec!["foo     ", "bar     ", "baz  "]
        );
    }

    #[test]
    fn issue_99() {
        // We did not reset the in_whitespace flag correctly and did
        // not handle single-character words after a line break.
        assert_eq!(
            wrap("aaabbbccc x yyyzzzwww", 9),
            vec!["aaabbbccc", "x", "yyyzzzwww"]
        );
    }

    #[test]
    fn issue_129() {
        // The dash is an em-dash which takes up four bytes. We used
        // to panic since we tried to index into the character.
        let options = Options::new(1).word_separator(WordSeparator::AsciiSpace);
        assert_eq!(wrap("x – x", options), vec!["x", "–", "x"]);
    }

    #[test]
    fn wide_character_handling() {
        assert_eq!(wrap("Hello, World!", 15), vec!["Hello, World!"]);
        assert_eq!(
            wrap(
                "Ｈｅｌｌｏ, Ｗｏｒｌｄ!",
                Options::new(15).word_separator(WordSeparator::AsciiSpace)
            ),
            vec!["Ｈｅｌｌｏ,", "Ｗｏｒｌｄ!"]
        );

        // Wide characters are allowed to break if the
        // unicode-linebreak feature is enabled.
        #[cfg(feature = "unicode-linebreak")]
        assert_eq!(
            wrap(
                "Ｈｅｌｌｏ, Ｗｏｒｌｄ!",
                Options::new(15).word_separator(WordSeparator::UnicodeBreakProperties),
            ),
            vec!["Ｈｅｌｌｏ, Ｗ", "ｏｒｌｄ!"]
        );
    }

    #[test]
    #[cfg(feature = "unicode-width")]
    fn wrap_wide_characters_unicode_width() {
        // Wide characters occupy 2 columns each. This works in both std
        // and no_std environments with the unicode-width feature.
        let options = Options::new(8);
        assert_eq!(wrap("안녕하세요", &options), vec!["안녕하세", "요"]);
    }

    #[test]
    fn indent_empty_line() {
        // Previously, indentation was not applied to empty lines.
        // However, this is somewhat inconsistent and undesirable if
        // the indentation is something like a border ("| ") which you
        // want to apply to all lines, empty or not.
        let options = Options::new(10).initial_indent("!!!");
        assert_eq!(wrap("", &options), vec!["!!!"]);
    }

    #[test]
    fn indent_single_line() {
        let options = Options::new(10).initial_indent(">>>"); // No trailing space
        assert_eq!(wrap("foo", &options), vec![">>>foo"]);
    }

    #[test]
    fn indent_first_emoji() {
        let options = Options::new(10).initial_indent("👉👉");
        assert_eq!(
            wrap("x x x x x x x x x x x x x", &options),
            vec!["👉👉x x x", "x x x x x", "x x x x x"]
        );
    }

    #[test]
    fn indent_multiple_lines() {
        let options = Options::new(6).initial_indent("* ").subsequent_indent("  ");
        assert_eq!(
            wrap("foo bar baz", &options),
            vec!["* foo", "  bar", "  baz"]
        );
    }

    #[test]
    fn only_initial_indent_multiple_lines() {
        let options = Options::new(10).initial_indent("  ");
        assert_eq!(wrap("foo\nbar\nbaz", &options), vec!["  foo", "bar", "baz"]);
    }

    #[test]
    fn only_subsequent_indent_multiple_lines() {
        let options = Options::new(10).subsequent_indent("  ");
        assert_eq!(
            wrap("foo\nbar\nbaz", &options),
            vec!["foo", "  bar", "  baz"]
        );
    }

    #[test]
    fn indent_break_words() {
        let options = Options::new(5).initial_indent("* ").subsequent_indent("  ");
        assert_eq!(wrap("foobarbaz", &options), vec!["* foo", "  bar", "  baz"]);
    }

    #[test]
    fn initial_indent_break_words() {
        // This is a corner-case showing how the long word is broken
        // according to the width of the subsequent lines. The first
        // fragment of the word no longer fits on the first line,
        // which ends up being pure indentation.
        let options = Options::new(5).initial_indent("-->");
        assert_eq!(wrap("foobarbaz", &options), vec!["-->", "fooba", "rbaz"]);
    }

    #[test]
    fn hyphens() {
        assert_eq!(wrap("foo-bar", 5), vec!["foo-", "bar"]);
    }

    #[test]
    fn trailing_hyphen() {
        let options = Options::new(5).break_words(false);
        assert_eq!(wrap("foobar-", &options), vec!["foobar-"]);
    }

    #[test]
    fn multiple_hyphens() {
        assert_eq!(wrap("foo-bar-baz", 5), vec!["foo-", "bar-", "baz"]);
    }

    #[test]
    fn hyphens_flag() {
        let options = Options::new(5).break_words(false);
        assert_eq!(
            wrap("The --foo-bar flag.", &options),
            vec!["The", "--foo-", "bar", "flag."]
        );
    }

    #[test]
    fn repeated_hyphens() {
        let options = Options::new(4).break_words(false);
        assert_eq!(wrap("foo--bar", &options), vec!["foo--bar"]);
    }

    #[test]
    fn hyphens_alphanumeric() {
        assert_eq!(wrap("Na2-CH4", 5), vec!["Na2-", "CH4"]);
    }

    #[test]
    fn hyphens_non_alphanumeric() {
        let options = Options::new(5).break_words(false);
        assert_eq!(wrap("foo(-)bar", &options), vec!["foo(-)bar"]);
    }

    #[test]
    fn multiple_splits() {
        assert_eq!(wrap("foo-bar-baz", 9), vec!["foo-bar-", "baz"]);
    }

    #[test]
    fn forced_split() {
        let options = Options::new(5).break_words(false);
        assert_eq!(wrap("foobar-baz", &options), vec!["foobar-", "baz"]);
    }

    #[test]
    fn multiple_unbroken_words_issue_193() {
        let options = Options::new(3).break_words(false);
        assert_eq!(
            wrap("small large tiny", &options),
            vec!["small", "large", "tiny"]
        );
        assert_eq!(
            wrap("small  large   tiny", &options),
            vec!["small", "large", "tiny"]
        );
    }

    #[test]
    fn very_narrow_lines_issue_193() {
        let options = Options::new(1).break_words(false);
        assert_eq!(wrap("fooo x y", &options), vec!["fooo", "x", "y"]);
        assert_eq!(wrap("fooo   x     y", &options), vec!["fooo", "x", "y"]);
    }

    #[test]
    fn simple_hyphens() {
        let options = Options::new(8).word_splitter(WordSplitter::HyphenSplitter);
        assert_eq!(wrap("foo bar-baz", &options), vec!["foo bar-", "baz"]);
    }

    #[test]
    fn no_hyphenation() {
        let options = Options::new(8).word_splitter(WordSplitter::NoHyphenation);
        assert_eq!(wrap("foo bar-baz", &options), vec!["foo", "bar-baz"]);
    }

    #[test]
    #[cfg(feature = "hyphenation")]
    fn auto_hyphenation_double_hyphenation() {
        let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
        let options = Options::new(10);
        assert_eq!(
            wrap("Internationalization", &options),
            vec!["Internatio", "nalization"]
        );

        let options = Options::new(10).word_splitter(WordSplitter::Hyphenation(dictionary));
        assert_eq!(
            wrap("Internationalization", &options),
            vec!["Interna-", "tionaliza-", "tion"]
        );
    }

    #[test]
    #[cfg(feature = "hyphenation")]
    fn auto_hyphenation_issue_158() {
        let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
        let options = Options::new(10);
        assert_eq!(
            wrap("participation is the key to success", &options),
            vec!["participat", "ion is", "the key to", "success"]
        );

        let options = Options::new(10).word_splitter(WordSplitter::Hyphenation(dictionary));
        assert_eq!(
            wrap("participation is the key to success", &options),
            vec!["partici-", "pation is", "the key to", "success"]
        );
    }

    #[test]
    #[cfg(feature = "hyphenation")]
    fn split_len_hyphenation() {
        // Test that hyphenation takes the width of the whitespace
        // into account.
        let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
        let options = Options::new(15).word_splitter(WordSplitter::Hyphenation(dictionary));
        assert_eq!(
            wrap("garbage   collection", &options),
            vec!["garbage   col-", "lection"]
        );
    }

    #[test]
    #[cfg(feature = "hyphenation")]
    fn borrowed_lines() {
        // Lines that end with an extra hyphen are owned, the final
        // line is borrowed.
        use alloc::borrow::Cow::{Borrowed, Owned};
        let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
        let options = Options::new(10).word_splitter(WordSplitter::Hyphenation(dictionary));
        let lines = wrap("Internationalization", &options);
        assert_eq!(lines, vec!["Interna-", "tionaliza-", "tion"]);
        if let Borrowed(s) = lines[0] {
            assert!(false, "should not have been borrowed: {:?}", s);
        }
        if let Borrowed(s) = lines[1] {
            assert!(false, "should not have been borrowed: {:?}", s);
        }
        if let Owned(ref s) = lines[2] {
            assert!(false, "should not have been owned: {:?}", s);
        }
    }

    #[test]
    #[cfg(feature = "hyphenation")]
    fn auto_hyphenation_with_hyphen() {
        let dictionary = Standard::from_embedded(Language::EnglishUS).unwrap();
        let options = Options::new(8).break_words(false);
        assert_eq!(
            wrap("over-caffinated", &options),
            vec!["over-", "caffinated"]
        );

        let options = options.word_splitter(WordSplitter::Hyphenation(dictionary));
        assert_eq!(
            wrap("over-caffinated", &options),
            vec!["over-", "caffi-", "nated"]
        );
    }

    #[test]
    fn break_words() {
        assert_eq!(wrap("foobarbaz", 3), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn break_words_wide_characters() {
        // Even the poor man's version of `ch_width` counts these
        // characters as wide.
        let options = Options::new(5).word_separator(WordSeparator::AsciiSpace);
        assert_eq!(wrap("Ｈｅｌｌｏ", options), vec!["Ｈｅ", "ｌｌ", "ｏ"]);
    }

    #[test]
    fn break_words_zero_width() {
        assert_eq!(wrap("foobar", 0), vec!["f", "o", "o", "b", "a", "r"]);
    }

    #[test]
    fn break_long_first_word() {
        assert_eq!(wrap("testx y", 4), vec!["test", "x y"]);
    }

    #[test]
    fn wrap_preserves_line_breaks_trims_whitespace() {
        assert_eq!(wrap("  ", 80), vec![""]);
        assert_eq!(wrap("  \n  ", 80), vec!["", ""]);
        assert_eq!(wrap("  \n \n  \n ", 80), vec!["", "", "", ""]);
    }

    #[test]
    fn wrap_preserves_line_breaks_and_trailing_whitespace() {
        let options = Options::new(80).preserve_trailing_space(true);
        assert_eq!(wrap("  ", &options), vec!["  "]);
        assert_eq!(wrap("  \n  ", &options), vec!["  ", "  "]);
        assert_eq!(wrap("  \n \n  \n ", &options), vec!["  ", " ", "  ", " "]);
    }

    #[test]
    fn preserve_trailing_space_borrows_spaces() {
        use alloc::borrow::Cow::Owned;
        let lines = wrap(
            "foo bar baz",
            Options::new(10).preserve_trailing_space(true),
        );
        assert_eq!(lines, vec!["foo bar ", "baz"]);
        if let Owned(ref s) = lines[0] {
            assert!(false, "should not have been owned: {:?}", s);
        }
    }

    #[test]
    fn wrap_colored_text() {
        // The words are much longer than 6 bytes, but they remain
        // intact after filling the text.
        let green_hello = "\u{1b}[0m\u{1b}[32mHello\u{1b}[0m";
        let blue_world = "\u{1b}[0m\u{1b}[34mWorld!\u{1b}[0m";
        assert_eq!(
            wrap(&format!("{} {}", green_hello, blue_world), 6),
            vec![green_hello, blue_world],
        );
    }
