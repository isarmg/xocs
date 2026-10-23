
    use super::*;

    #[test]
    fn test_fixed_offset_min_max_dates() {
        for offset_hour in -23..=23 {
            dbg!(offset_hour);
            let offset = FixedOffset::east_opt(offset_hour * 60 * 60).unwrap();

            let local_max = offset.from_utc_datetime(&NaiveDateTime::MAX);
            assert_eq!(local_max.naive_utc(), NaiveDateTime::MAX);
            let local_min = offset.from_utc_datetime(&NaiveDateTime::MIN);
            assert_eq!(local_min.naive_utc(), NaiveDateTime::MIN);

            let local_max = offset.from_local_datetime(&NaiveDateTime::MAX);
            if offset_hour >= 0 {
                assert_eq!(local_max.unwrap().naive_local(), NaiveDateTime::MAX);
            } else {
                assert_eq!(local_max, MappedLocalTime::None);
            }
            let local_min = offset.from_local_datetime(&NaiveDateTime::MIN);
            if offset_hour <= 0 {
                assert_eq!(local_min.unwrap().naive_local(), NaiveDateTime::MIN);
            } else {
                assert_eq!(local_min, MappedLocalTime::None);
            }
        }
    }

    #[test]
    fn test_negative_millis() {
        let dt = Utc.timestamp_millis_opt(-1000).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59 UTC");
        let dt = Utc.timestamp_millis_opt(-7000).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:53 UTC");
        let dt = Utc.timestamp_millis_opt(-7001).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:52.999 UTC");
        let dt = Utc.timestamp_millis_opt(-7003).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:52.997 UTC");
        let dt = Utc.timestamp_millis_opt(-999).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59.001 UTC");
        let dt = Utc.timestamp_millis_opt(-1).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59.999 UTC");
        let dt = Utc.timestamp_millis_opt(-60000).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:00 UTC");
        let dt = Utc.timestamp_millis_opt(-3600000).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:00:00 UTC");

        for (millis, expected) in &[
            (-7000, "1969-12-31 23:59:53 UTC"),
            (-7001, "1969-12-31 23:59:52.999 UTC"),
            (-7003, "1969-12-31 23:59:52.997 UTC"),
        ] {
            match Utc.timestamp_millis_opt(*millis) {
                MappedLocalTime::Single(dt) => {
                    assert_eq!(dt.to_string(), *expected);
                }
                e => panic!("Got {e:?} instead of an okay answer"),
            }
        }
    }

    #[test]
    fn test_negative_nanos() {
        let dt = Utc.timestamp_nanos(-1_000_000_000);
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59 UTC");
        let dt = Utc.timestamp_nanos(-999_999_999);
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59.000000001 UTC");
        let dt = Utc.timestamp_nanos(-1);
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59.999999999 UTC");
        let dt = Utc.timestamp_nanos(-60_000_000_000);
        assert_eq!(dt.to_string(), "1969-12-31 23:59:00 UTC");
        let dt = Utc.timestamp_nanos(-3_600_000_000_000);
        assert_eq!(dt.to_string(), "1969-12-31 23:00:00 UTC");
    }

    #[test]
    fn test_nanos_never_panics() {
        Utc.timestamp_nanos(i64::MAX);
        Utc.timestamp_nanos(i64::default());
        Utc.timestamp_nanos(i64::MIN);
    }

    #[test]
    fn test_negative_micros() {
        let dt = Utc.timestamp_micros(-1_000_000).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59 UTC");
        let dt = Utc.timestamp_micros(-999_999).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59.000001 UTC");
        let dt = Utc.timestamp_micros(-1).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:59.999999 UTC");
        let dt = Utc.timestamp_micros(-60_000_000).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:59:00 UTC");
        let dt = Utc.timestamp_micros(-3_600_000_000).unwrap();
        assert_eq!(dt.to_string(), "1969-12-31 23:00:00 UTC");
    }
