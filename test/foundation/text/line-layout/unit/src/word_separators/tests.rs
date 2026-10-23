
    use super::WordSeparator::*;
    use super::*;
    use alloc::vec;

    // Like assert_eq!, but the left expression is an iterator.
    macro_rules! assert_iter_eq {
        ($left:expr, $right:expr) => {
            assert_eq!($left.collect::<Vec<_>>(), $right);
        };
    }

    fn to_words(words: Vec<&str>) -> Vec<Word<'_>> {
        words.into_iter().map(Word::from).collect()
    }

    macro_rules! test_find_words {
        ($ascii_name:ident,
         $unicode_name:ident,
         $([ $line:expr, $ascii_words:expr, $unicode_words:expr ]),+) => {
            #[test]
            fn $ascii_name() {
                $(
                    let expected_words = to_words($ascii_words.to_vec());
                    let actual_words = WordSeparator::AsciiSpace
                        .find_words($line)
                        .collect::<Vec<_>>();
                    assert_eq!(actual_words, expected_words, "Line: {:?}", $line);
                )+
            }

            #[test]
            #[cfg(feature = "unicode-linebreak")]
            fn $unicode_name() {
                $(
                    let expected_words = to_words($unicode_words.to_vec());
                    let actual_words = WordSeparator::UnicodeBreakProperties
                        .find_words($line)
                        .collect::<Vec<_>>();
                    assert_eq!(actual_words, expected_words, "Line: {:?}", $line);
                )+
            }
        };
    }

    test_find_words!(ascii_space_empty, unicode_empty, ["", [], []]);

    test_find_words!(
        ascii_single_word,
        unicode_single_word,
        ["foo", ["foo"], ["foo"]]
    );

    test_find_words!(
        ascii_two_words,
        unicode_two_words,
        ["foo bar", ["foo ", "bar"], ["foo ", "bar"]]
    );

    test_find_words!(
        ascii_multiple_words,
        unicode_multiple_words,
        ["foo bar", ["foo ", "bar"], ["foo ", "bar"]],
        ["x y z", ["x ", "y ", "z"], ["x ", "y ", "z"]]
    );

    test_find_words!(
        ascii_only_whitespace,
        unicode_only_whitespace,
        [" ", [" "], [" "]],
        ["    ", ["    "], ["    "]]
    );

    test_find_words!(
        ascii_inter_word_whitespace,
        unicode_inter_word_whitespace,
        ["foo   bar", ["foo   ", "bar"], ["foo   ", "bar"]]
    );

    test_find_words!(
        ascii_trailing_whitespace,
        unicode_trailing_whitespace,
        ["foo   ", ["foo   "], ["foo   "]]
    );

    test_find_words!(
        ascii_leading_whitespace,
        unicode_leading_whitespace,
        ["   foo", ["   ", "foo"], ["   ", "foo"]]
    );

    test_find_words!(
        ascii_multi_column_char,
        unicode_multi_column_char,
        ["\u{1f920}", ["\u{1f920}"], ["\u{1f920}"]] // cowboy emoji 🤠
    );

    test_find_words!(
        ascii_hyphens,
        unicode_hyphens,
        ["foo-bar", ["foo-bar"], ["foo-bar"]],
        ["foo- bar", ["foo- ", "bar"], ["foo- ", "bar"]],
        ["foo - bar", ["foo ", "- ", "bar"], ["foo ", "- ", "bar"]],
        ["foo -bar", ["foo ", "-bar"], ["foo ", "-bar"]]
    );

    test_find_words!(
        ascii_newline,
        unicode_newline,
        ["foo\nbar", ["foo\nbar"], ["foo\n", "bar"]]
    );

    test_find_words!(
        ascii_tab,
        unicode_tab,
        ["foo\tbar", ["foo\tbar"], ["foo\t", "bar"]]
    );

    test_find_words!(
        ascii_non_breaking_space,
        unicode_non_breaking_space,
        ["foo\u{00A0}bar", ["foo\u{00A0}bar"], ["foo\u{00A0}bar"]]
    );

    #[test]
    #[cfg(all(unix, feature = "std"))]
    fn find_words_colored_text() {
        use termion::color::{Blue, Fg, Green, Reset};

        let green_hello = format!("{}Hello{} ", Fg(Green), Fg(Reset));
        let blue_world = format!("{}World!{}", Fg(Blue), Fg(Reset));
        assert_iter_eq!(
            AsciiSpace.find_words(&format!("{}{}", green_hello, blue_world)),
            vec![Word::from(&green_hello), Word::from(&blue_world)]
        );

        #[cfg(feature = "unicode-linebreak")]
        assert_iter_eq!(
            UnicodeBreakProperties.find_words(&format!("{}{}", green_hello, blue_world)),
            vec![Word::from(&green_hello), Word::from(&blue_world)]
        );
    }

    #[test]
    fn find_words_color_inside_word() {
        let text = "foo\u{1b}[0m\u{1b}[32mbar\u{1b}[0mbaz";
        assert_iter_eq!(AsciiSpace.find_words(text), vec![Word::from(text)]);

        #[cfg(feature = "unicode-linebreak")]
        assert_iter_eq!(
            UnicodeBreakProperties.find_words(text),
            vec![Word::from(text)]
        );
    }

    #[test]
    #[cfg(feature = "unicode-linebreak")]
    fn find_words_complex_scripts() {
        let text = "ภาษาไทย"; // English: "Thai language"

        #[cfg(feature = "unicode-linebreak-complex")]
        assert_iter_eq!(
            UnicodeBreakProperties.find_words(text),
            vec![Word::from("ภาษา"), Word::from("ไทย")]
        );

        #[cfg(not(feature = "unicode-linebreak-complex"))]
        assert_iter_eq!(
            UnicodeBreakProperties.find_words(text),
            vec![Word::from(text)]
        );
    }

    #[test]
    fn word_separator_new() {
        #[cfg(feature = "unicode-linebreak")]
        assert!(matches!(WordSeparator::new(), UnicodeBreakProperties));

        #[cfg(not(feature = "unicode-linebreak"))]
        assert!(matches!(WordSeparator::new(), AsciiSpace));
    }
