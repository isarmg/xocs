
    use super::*;

    #[test]
    fn should_pass() {
        for (flags_text, expected) in &[
            ("", (false, false)),
            ("i", (false, false)),
            ("u", (true, false)),
            ("v", (true, true)),
            ("vg", (true, true)),
        ] {
            let reader = Reader::initialize(flags_text, true, false).unwrap();
            let result = FlagsParser::new(reader, 0).parse().unwrap();
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn should_fail() {
        for flags_text in &["uv", "vu", "uu", "vv", "gg", "$"] {
            let reader = Reader::initialize(flags_text, true, false).unwrap();
            let err = FlagsParser::new(reader, 0).parse();
            assert!(err.is_err());
            // println!("{:?}", err.unwrap_err().with_source_code(*flags_text));
        }
        for flags_text in &[r#""uv""#, "'V'", "\"-\"", r#""\162""#] {
            let reader = Reader::initialize(flags_text, true, true).unwrap();
            let err = FlagsParser::new(reader, 0).parse();
            assert!(err.is_err());
            // println!("{:?}", err.unwrap_err().with_source_code(*flags_text));
        }
    }

    #[test]
    fn string_literal() {
        for reader in [
            Reader::initialize("u", true, false).unwrap(),
            Reader::initialize("'u'", true, true).unwrap(),
            Reader::initialize(r#""\165""#, true, true).unwrap(),
            Reader::initialize(r#""\x75""#, true, true).unwrap(),
            Reader::initialize(r#""\u0075""#, true, true).unwrap(),
            Reader::initialize(r#""\u{0075}""#, true, true).unwrap(),
        ] {
            let result = FlagsParser::new(reader, 0).parse().unwrap();
            assert_eq!(result, (true, false));
        }
    }
