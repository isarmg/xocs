
    use super::*;

    #[test]
    fn options_agree_with_usize() {
        let opt_usize = Options::from(42_usize);
        let opt_options = Options::new(42);

        assert_eq!(opt_usize.width, opt_options.width);
        assert_eq!(opt_usize.initial_indent, opt_options.initial_indent);
        assert_eq!(opt_usize.subsequent_indent, opt_options.subsequent_indent);
        assert_eq!(opt_usize.break_words, opt_options.break_words);
        assert_eq!(
            opt_usize.word_splitter.split_points("hello-world"),
            opt_options.word_splitter.split_points("hello-world")
        );
    }
