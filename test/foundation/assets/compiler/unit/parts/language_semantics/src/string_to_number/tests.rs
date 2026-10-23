
    use super::StringToNumber;

    #[test]
    fn trims_ecmascript_whitespace_from_both_ends() {
        const WHITESPACE: &str = "\u{0009}\u{000B}\u{000C}\u{0020}\u{00A0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{202F}\u{205F}\u{3000}\u{FEFF}\u{000A}\u{000D}\u{2028}\u{2029}";

        assert_eq!(format!("{WHITESPACE}1{WHITESPACE}").as_str().string_to_number(), 1.0);
        assert_eq!(format!("{WHITESPACE}0x10{WHITESPACE}").as_str().string_to_number(), 16.0);
        assert_eq!(
            format!("{WHITESPACE}Infinity{WHITESPACE}").as_str().string_to_number(),
            f64::INFINITY
        );
        assert_eq!(format!("{WHITESPACE}{WHITESPACE}").as_str().string_to_number(), 0.0);
    }

    #[test]
    fn does_not_trim_non_ecmascript_whitespace() {
        assert!("1\u{0085}".string_to_number().is_nan());
    }
