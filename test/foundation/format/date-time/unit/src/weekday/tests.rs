
    use super::Weekday;

    #[test]
    fn test_days_since() {
        for i in 0..7 {
            let base_day = Weekday::try_from(i).unwrap();

            assert_eq!(
                base_day.num_days_from_monday(),
                base_day.days_since(Weekday::Mon)
            );
            assert_eq!(
                base_day.num_days_from_sunday(),
                base_day.days_since(Weekday::Sun)
            );

            assert_eq!(base_day.days_since(base_day), 0);

            assert_eq!(base_day.days_since(base_day.pred()), 1);
            assert_eq!(base_day.days_since(base_day.pred().pred()), 2);
            assert_eq!(base_day.days_since(base_day.pred().pred().pred()), 3);
            assert_eq!(base_day.days_since(base_day.pred().pred().pred().pred()), 4);
            assert_eq!(
                base_day.days_since(base_day.pred().pred().pred().pred().pred()),
                5
            );
            assert_eq!(
                base_day.days_since(base_day.pred().pred().pred().pred().pred().pred()),
                6
            );

            assert_eq!(base_day.days_since(base_day.succ()), 6);
            assert_eq!(base_day.days_since(base_day.succ().succ()), 5);
            assert_eq!(base_day.days_since(base_day.succ().succ().succ()), 4);
            assert_eq!(base_day.days_since(base_day.succ().succ().succ().succ()), 3);
            assert_eq!(
                base_day.days_since(base_day.succ().succ().succ().succ().succ()),
                2
            );
            assert_eq!(
                base_day.days_since(base_day.succ().succ().succ().succ().succ().succ()),
                1
            );
        }
    }

    #[test]
    fn test_formatting_alignment() {
        // No exhaustive testing here as we just delegate the
        // implementation to Formatter::pad. Just some basic smoke
        // testing to ensure that it's in fact being done.
        assert_eq!(format!("{:x>7}", Weekday::Mon), "xxxxMon");
        assert_eq!(format!("{:^7}", Weekday::Mon), "  Mon  ");
        assert_eq!(format!("{:Z<7}", Weekday::Mon), "MonZZZZ");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_serialize() {
        use serde_json::to_string;
        use Weekday::*;

        let cases: Vec<(Weekday, &str)> = vec![
            (Mon, "\"Mon\""),
            (Tue, "\"Tue\""),
            (Wed, "\"Wed\""),
            (Thu, "\"Thu\""),
            (Fri, "\"Fri\""),
            (Sat, "\"Sat\""),
            (Sun, "\"Sun\""),
        ];

        for (weekday, expected_str) in cases {
            let string = to_string(&weekday).unwrap();
            assert_eq!(string, expected_str);
        }
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_deserialize() {
        use serde_json::from_str;
        use Weekday::*;

        let cases: Vec<(&str, Weekday)> = vec![
            ("\"mon\"", Mon),
            ("\"MONDAY\"", Mon),
            ("\"MonDay\"", Mon),
            ("\"mOn\"", Mon),
            ("\"tue\"", Tue),
            ("\"tuesday\"", Tue),
            ("\"wed\"", Wed),
            ("\"wednesday\"", Wed),
            ("\"thu\"", Thu),
            ("\"thursday\"", Thu),
            ("\"fri\"", Fri),
            ("\"friday\"", Fri),
            ("\"sat\"", Sat),
            ("\"saturday\"", Sat),
            ("\"sun\"", Sun),
            ("\"sunday\"", Sun),
        ];

        for (str, expected_weekday) in cases {
            let weekday = from_str::<Weekday>(str).unwrap();
            assert_eq!(weekday, expected_weekday);
        }

        let errors: Vec<&str> = vec![
            "\"not a weekday\"",
            "\"monDAYs\"",
            "\"mond\"",
            "mon",
            "\"thur\"",
            "\"thurs\"",
        ];

        for str in errors {
            from_str::<Weekday>(str).unwrap_err();
        }
    }
