
    use super::FixedOffset;
    use crate::offset::TimeZone;
    use std::str::FromStr;

    #[test]
    fn test_date_extreme_offset() {
        // starting from 0.3 we don't have an offset exceeding one day.
        // this makes everything easier!
        let offset = FixedOffset::east_opt(86399).unwrap();
        assert_eq!(
            format!(
                "{:?}",
                offset.with_ymd_and_hms(2012, 2, 29, 5, 6, 7).unwrap()
            ),
            "2012-02-29T05:06:07+23:59:59"
        );
        let offset = FixedOffset::east_opt(-86399).unwrap();
        assert_eq!(
            format!(
                "{:?}",
                offset.with_ymd_and_hms(2012, 2, 29, 5, 6, 7).unwrap()
            ),
            "2012-02-29T05:06:07-23:59:59"
        );
        let offset = FixedOffset::west_opt(86399).unwrap();
        assert_eq!(
            format!(
                "{:?}",
                offset.with_ymd_and_hms(2012, 3, 4, 5, 6, 7).unwrap()
            ),
            "2012-03-04T05:06:07-23:59:59"
        );
        let offset = FixedOffset::west_opt(-86399).unwrap();
        assert_eq!(
            format!(
                "{:?}",
                offset.with_ymd_and_hms(2012, 3, 4, 5, 6, 7).unwrap()
            ),
            "2012-03-04T05:06:07+23:59:59"
        );
    }

    #[test]
    fn test_parse_offset() {
        let offset = FixedOffset::from_str("-0500").unwrap();
        assert_eq!(offset.local_minus_utc, -5 * 3600);
        let offset = FixedOffset::from_str("-08:00").unwrap();
        assert_eq!(offset.local_minus_utc, -8 * 3600);
        let offset = FixedOffset::from_str("+06:30").unwrap();
        assert_eq!(offset.local_minus_utc, (6 * 3600) + 1800);
    }
