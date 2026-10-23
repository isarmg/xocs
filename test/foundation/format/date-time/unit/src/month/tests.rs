
    use super::Month;
    use crate::{Datelike, Months, OutOfRange, TimeZone, Utc};

    #[test]
    fn test_month_enum_try_from() {
        assert_eq!(Month::try_from(1), Ok(Month::January));
        assert_eq!(Month::try_from(2), Ok(Month::February));
        assert_eq!(Month::try_from(12), Ok(Month::December));
        assert_eq!(Month::try_from(13), Err(OutOfRange::new()));

        let date = Utc.with_ymd_and_hms(2019, 10, 28, 9, 10, 11).unwrap();
        assert_eq!(Month::try_from(date.month() as u8), Ok(Month::October));

        let month = Month::January;
        let dt = Utc
            .with_ymd_and_hms(2019, month.number_from_month(), 28, 9, 10, 11)
            .unwrap();
        assert_eq!((dt.year(), dt.month(), dt.day()), (2019, 1, 28));
    }

    #[test]
    fn test_month_enum_primitive_parse() {
        use num_traits::FromPrimitive;

        let jan_opt = Month::from_u32(1);
        let feb_opt = Month::from_u64(2);
        let dec_opt = Month::from_i64(12);
        let no_month = Month::from_u32(13);
        assert_eq!(jan_opt, Some(Month::January));
        assert_eq!(feb_opt, Some(Month::February));
        assert_eq!(dec_opt, Some(Month::December));
        assert_eq!(no_month, None);

        let date = Utc.with_ymd_and_hms(2019, 10, 28, 9, 10, 11).unwrap();
        assert_eq!(Month::from_u32(date.month()), Some(Month::October));

        let month = Month::January;
        let dt = Utc
            .with_ymd_and_hms(2019, month.number_from_month(), 28, 9, 10, 11)
            .unwrap();
        assert_eq!((dt.year(), dt.month(), dt.day()), (2019, 1, 28));
    }

    #[test]
    fn test_month_enum_succ_pred() {
        assert_eq!(Month::January.succ(), Month::February);
        assert_eq!(Month::December.succ(), Month::January);
        assert_eq!(Month::January.pred(), Month::December);
        assert_eq!(Month::February.pred(), Month::January);
    }

    #[test]
    fn test_month_partial_ord() {
        assert!(Month::January <= Month::January);
        assert!(Month::January < Month::February);
        assert!(Month::January < Month::December);
        assert!(Month::July >= Month::May);
        assert!(Month::September > Month::March);
    }

    #[test]
    fn test_months_as_u32() {
        assert_eq!(Months::new(0).as_u32(), 0);
        assert_eq!(Months::new(1).as_u32(), 1);
        assert_eq!(Months::new(u32::MAX).as_u32(), u32::MAX);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_serialize() {
        use serde_json::to_string;
        use Month::*;

        let cases: Vec<(Month, &str)> = vec![
            (January, "\"January\""),
            (February, "\"February\""),
            (March, "\"March\""),
            (April, "\"April\""),
            (May, "\"May\""),
            (June, "\"June\""),
            (July, "\"July\""),
            (August, "\"August\""),
            (September, "\"September\""),
            (October, "\"October\""),
            (November, "\"November\""),
            (December, "\"December\""),
        ];

        for (month, expected_str) in cases {
            let string = to_string(&month).unwrap();
            assert_eq!(string, expected_str);
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_deserialize() {
        use serde_json::from_str;
        use Month::*;

        let cases: Vec<(&str, Month)> = vec![
            ("\"january\"", January),
            ("\"jan\"", January),
            ("\"FeB\"", February),
            ("\"MAR\"", March),
            ("\"mar\"", March),
            ("\"april\"", April),
            ("\"may\"", May),
            ("\"june\"", June),
            ("\"JULY\"", July),
            ("\"august\"", August),
            ("\"september\"", September),
            ("\"October\"", October),
            ("\"November\"", November),
            ("\"DECEmbEr\"", December),
        ];

        for (string, expected_month) in cases {
            let month = from_str::<Month>(string).unwrap();
            assert_eq!(month, expected_month);
        }

        let errors: Vec<&str> = vec![
            "\"not a month\"",
            "\"ja\"",
            "\"Dece\"",
            "Dec",
            "\"Augustin\"",
        ];

        for string in errors {
            from_str::<Month>(string).unwrap_err();
        }
    }

    #[test]
    fn num_days() {
        assert_eq!(Month::January.num_days(2020), Some(31));
        assert_eq!(Month::February.num_days(2020), Some(29));
        assert_eq!(Month::February.num_days(2019), Some(28));
    }
