
    use super::*;
    use crate::Casing;
    use crate::Pattern;

    #[test]
    fn snake_converter_from_case() {
        let conv = Converter::new().to_case(Case::Snake);
        let s = String::from("my var name");
        assert_eq!(s.to_case(Case::Snake), conv.convert(s));
    }

    #[test]
    fn snake_converter_from_scratch() {
        let conv = Converter::new()
            .set_delim("_")
            .set_pattern(Pattern::Lowercase);
        let s = String::from("my var name");
        assert_eq!(s.to_case(Case::Snake), conv.convert(s));
    }

    #[test]
    fn custom_pattern() {
        let conv = Converter::new()
            .to_case(Case::Snake)
            .set_pattern(Pattern::Sentence);
        assert_eq!("Bjarne_case", conv.convert("bjarne case"));
    }

    #[test]
    fn custom_delim() {
        let conv = Converter::new().set_delim("..");
        assert_eq!("oh..My", conv.convert("ohMy"));
    }

    #[test]
    fn no_pattern() {
        let conv = Converter::new()
            .from_case(Case::Title)
            .to_case(Case::Kebab)
            .remove_pattern();
        assert_eq!("wIErd-CASing", conv.convert("wIErd CASing"));
    }

    #[test]
    fn no_delim() {
        let conv = Converter::new()
            .from_case(Case::Title)
            .to_case(Case::Kebab)
            .remove_delim();
        assert_eq!("justflat", conv.convert("Just Flat"));
    }

    #[test]
    fn no_digit_boundaries() {
        let conv = Converter::new()
            .remove_boundaries(&Boundary::digits())
            .to_case(Case::Snake);
        assert_eq!("test_08bound", conv.convert("Test 08Bound"));
        assert_eq!("a8a_a8a", conv.convert("a8aA8A"));
    }

    #[test]
    fn remove_boundary() {
        let conv = Converter::new()
            .remove_boundary(Boundary::DigitUpper)
            .to_case(Case::Snake);
        assert_eq!("test_08bound", conv.convert("Test 08Bound"));
        assert_eq!("a_8_a_a_8a", conv.convert("a8aA8A"));
    }

    #[test]
    fn add_boundary() {
        let conv = Converter::new()
            .from_case(Case::Snake)
            .to_case(Case::Kebab)
            .add_boundary(Boundary::LowerUpper);
        assert_eq!("word-word-word", conv.convert("word_wordWord"));
    }

    #[test]
    fn add_boundaries() {
        let conv = Converter::new()
            .from_case(Case::Snake)
            .to_case(Case::Kebab)
            .add_boundaries(&[Boundary::LowerUpper, Boundary::UpperLower]);
        assert_eq!("word-word-w-ord", conv.convert("word_wordWord"));
    }

    #[test]
    fn reuse_after_change() {
        let conv = Converter::new().from_case(Case::Snake).to_case(Case::Kebab);
        assert_eq!("word-wordword", conv.convert("word_wordWord"));

        let conv = conv.add_boundary(Boundary::LowerUpper);
        assert_eq!("word-word-word", conv.convert("word_wordWord"));
    }

    #[test]
    fn explicit_boundaries() {
        let conv = Converter::new()
            .set_boundaries(&[
                Boundary::DigitLower,
                Boundary::DigitUpper,
                Boundary::Acronym,
            ])
            .to_case(Case::Snake);
        assert_eq!(
            "section8_lesson2_http_requests",
            conv.convert("section8lesson2HTTPRequests")
        );
    }
