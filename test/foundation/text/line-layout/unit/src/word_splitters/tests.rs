
    use super::*;
    use alloc::vec;

    // Like assert_eq!, but the left expression is an iterator.
    macro_rules! assert_iter_eq {
        ($left:expr, $right:expr) => {
            assert_eq!($left.collect::<Vec<_>>(), $right);
        };
    }

    #[test]
    fn split_words_no_words() {
        assert_iter_eq!(split_words(vec![], &WordSplitter::HyphenSplitter), vec![]);
    }

    #[test]
    fn split_words_empty_word() {
        assert_iter_eq!(
            split_words(vec![Word::from("   ")], &WordSplitter::HyphenSplitter),
            vec![Word::from("   ")]
        );
    }

    #[test]
    fn split_words_single_word() {
        assert_iter_eq!(
            split_words(vec![Word::from("foobar")], &WordSplitter::HyphenSplitter),
            vec![Word::from("foobar")]
        );
    }

    #[test]
    fn split_words_hyphen_splitter() {
        assert_iter_eq!(
            split_words(vec![Word::from("foo-bar")], &WordSplitter::HyphenSplitter),
            vec![Word::from("foo-"), Word::from("bar")]
        );
    }

    #[test]
    fn split_words_no_hyphenation() {
        assert_iter_eq!(
            split_words(vec![Word::from("foo-bar")], &WordSplitter::NoHyphenation),
            vec![Word::from("foo-bar")]
        );
    }

    #[test]
    fn split_words_adds_penalty() {
        let fixed_split_point = |_: &str| vec![3];

        assert_iter_eq!(
            split_words(
                vec![Word::from("foobar")].into_iter(),
                &WordSplitter::Custom(fixed_split_point)
            ),
            vec![
                Word {
                    word: "foo",
                    width: 3,
                    whitespace: "",
                    penalty: "-"
                },
                Word {
                    word: "bar",
                    width: 3,
                    whitespace: "",
                    penalty: ""
                }
            ]
        );

        assert_iter_eq!(
            split_words(
                vec![Word::from("fo-bar")].into_iter(),
                &WordSplitter::Custom(fixed_split_point)
            ),
            vec![
                Word {
                    word: "fo-",
                    width: 3,
                    whitespace: "",
                    penalty: ""
                },
                Word {
                    word: "bar",
                    width: 3,
                    whitespace: "",
                    penalty: ""
                }
            ]
        );
    }
