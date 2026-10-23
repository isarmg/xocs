
    use super::{
        comment_2822, nanosecond, nanosecond_fixed, short_or_long_month0, short_or_long_weekday,
        timezone_offset_2822,
    };
    use crate::Weekday;
    use crate::format::{INVALID, TOO_SHORT};

    #[test]
    fn test_rfc2822_comments() {
        let testdata = [
            ("", Err(TOO_SHORT)),
            (" ", Err(TOO_SHORT)),
            ("x", Err(INVALID)),
            ("(", Err(TOO_SHORT)),
            ("()", Ok("")),
            (" \r\n\t()", Ok("")),
            ("() ", Ok(" ")),
            ("()z", Ok("z")),
            ("(x)", Ok("")),
            ("(())", Ok("")),
            ("((()))", Ok("")),
            ("(x(x(x)x)x)", Ok("")),
            ("( x ( x ( x ) x ) x )", Ok("")),
            (r"(\)", Err(TOO_SHORT)),
            (r"(\()", Ok("")),
            (r"(\))", Ok("")),
            (r"(\\)", Ok("")),
            ("(()())", Ok("")),
            ("( x ( x ) x ( x ) x )", Ok("")),
        ];

        for (test_in, expected) in testdata.iter() {
            let actual = comment_2822(test_in).map(|(s, _)| s);
            assert_eq!(
                *expected, actual,
                "{test_in:?} expected to produce {expected:?}, but produced {actual:?}."
            );
        }
    }

    #[test]
    fn test_timezone_offset_2822() {
        assert_eq!(timezone_offset_2822("cSt").unwrap(), ("", -21600));
        assert_eq!(timezone_offset_2822("pSt").unwrap(), ("", -28800));
        assert_eq!(timezone_offset_2822("mSt").unwrap(), ("", -25200));
        assert_eq!(timezone_offset_2822("-1551").unwrap(), ("", -57060));
        assert_eq!(timezone_offset_2822("Gp"), Err(INVALID));
    }

    #[test]
    fn test_short_or_long_month0() {
        assert_eq!(short_or_long_month0("JUn").unwrap(), ("", 5));
        assert_eq!(short_or_long_month0("mAy").unwrap(), ("", 4));
        assert_eq!(short_or_long_month0("AuG").unwrap(), ("", 7));
        assert_eq!(short_or_long_month0("Aprâ").unwrap(), ("â", 3));
        assert_eq!(short_or_long_month0("JUl").unwrap(), ("", 6));
        assert_eq!(short_or_long_month0("mAr").unwrap(), ("", 2));
        assert_eq!(short_or_long_month0("Jan").unwrap(), ("", 0));
    }

    #[test]
    fn test_short_or_long_weekday() {
        assert_eq!(short_or_long_weekday("sAtu").unwrap(), ("u", Weekday::Sat));
        assert_eq!(short_or_long_weekday("thu").unwrap(), ("", Weekday::Thu));
    }

    #[test]
    fn test_nanosecond_fixed() {
        assert_eq!(nanosecond_fixed("", 0usize).unwrap(), ("", 0));
        assert!(nanosecond_fixed("", 1usize).is_err());
    }

    #[test]
    fn test_nanosecond() {
        assert_eq!(nanosecond("2Ù").unwrap(), ("Ù", 200000000));
        assert_eq!(nanosecond("8").unwrap(), ("", 800000000));
    }
