
    use crate::serde::ts_nanoseconds_option;
    use crate::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

    use bincode::{deserialize, serialize};
    use serde_derive::{Deserialize, Serialize};

    #[test]
    fn test_serde_serialize() {
        assert_eq!(
            serde_json::to_string(
                &NaiveDate::from_ymd_opt(2016, 7, 8)
                    .unwrap()
                    .and_hms_milli_opt(9, 10, 48, 90)
                    .unwrap()
            )
            .ok(),
            Some(r#""2016-07-08T09:10:48.090""#.into())
        );
        assert_eq!(
            serde_json::to_string(
                &NaiveDate::from_ymd_opt(2014, 7, 24)
                    .unwrap()
                    .and_hms_opt(12, 34, 6)
                    .unwrap()
            )
            .ok(),
            Some(r#""2014-07-24T12:34:06""#.into())
        );
        assert_eq!(
            serde_json::to_string(
                &NaiveDate::from_ymd_opt(0, 1, 1)
                    .unwrap()
                    .and_hms_milli_opt(0, 0, 59, 1_000)
                    .unwrap()
            )
            .ok(),
            Some(r#""0000-01-01T00:00:60""#.into())
        );
        assert_eq!(
            serde_json::to_string(
                &NaiveDate::from_ymd_opt(-1, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 7)
                    .unwrap()
            )
            .ok(),
            Some(r#""-0001-12-31T23:59:59.000000007""#.into())
        );
        assert_eq!(
            serde_json::to_string(&NaiveDate::MIN.and_hms_opt(0, 0, 0).unwrap()).ok(),
            Some(r#""-262143-01-01T00:00:00""#.into())
        );
        assert_eq!(
            serde_json::to_string(
                &NaiveDate::MAX
                    .and_hms_nano_opt(23, 59, 59, 1_999_999_999)
                    .unwrap()
            )
            .ok(),
            Some(r#""+262142-12-31T23:59:60.999999999""#.into())
        );
    }

    #[test]
    fn test_serde_deserialize() {
        let from_str = serde_json::from_str::<NaiveDateTime>;

        assert_eq!(
            from_str(r#""2016-07-08T09:10:48.090""#).ok(),
            Some(
                NaiveDate::from_ymd_opt(2016, 7, 8)
                    .unwrap()
                    .and_hms_milli_opt(9, 10, 48, 90)
                    .unwrap()
            )
        );
        assert_eq!(
            from_str(r#""2016-7-8T9:10:48.09""#).ok(),
            Some(
                NaiveDate::from_ymd_opt(2016, 7, 8)
                    .unwrap()
                    .and_hms_milli_opt(9, 10, 48, 90)
                    .unwrap()
            )
        );
        assert_eq!(
            from_str(r#""2014-07-24T12:34:06""#).ok(),
            Some(
                NaiveDate::from_ymd_opt(2014, 7, 24)
                    .unwrap()
                    .and_hms_opt(12, 34, 6)
                    .unwrap()
            )
        );
        assert_eq!(
            from_str(r#""0000-01-01T00:00:60""#).ok(),
            Some(
                NaiveDate::from_ymd_opt(0, 1, 1)
                    .unwrap()
                    .and_hms_milli_opt(0, 0, 59, 1_000)
                    .unwrap()
            )
        );
        assert_eq!(
            from_str(r#""0-1-1T0:0:60""#).ok(),
            Some(
                NaiveDate::from_ymd_opt(0, 1, 1)
                    .unwrap()
                    .and_hms_milli_opt(0, 0, 59, 1_000)
                    .unwrap()
            )
        );
        assert_eq!(
            from_str(r#""-0001-12-31T23:59:59.000000007""#).ok(),
            Some(
                NaiveDate::from_ymd_opt(-1, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 7)
                    .unwrap()
            )
        );
        assert_eq!(
            from_str(r#""-262143-01-01T00:00:00""#).ok(),
            Some(NaiveDate::MIN.and_hms_opt(0, 0, 0).unwrap())
        );
        assert_eq!(
            from_str(r#""+262142-12-31T23:59:60.999999999""#).ok(),
            Some(
                NaiveDate::MAX
                    .and_hms_nano_opt(23, 59, 59, 1_999_999_999)
                    .unwrap()
            )
        );
        assert_eq!(
            from_str(r#""+262142-12-31T23:59:60.9999999999997""#).ok(), // excess digits are ignored
            Some(
                NaiveDate::MAX
                    .and_hms_nano_opt(23, 59, 59, 1_999_999_999)
                    .unwrap()
            )
        );

        // bad formats
        assert!(from_str(r#""""#).is_err());
        assert!(from_str(r#""2016-07-08""#).is_err());
        assert!(from_str(r#""09:10:48.090""#).is_err());
        assert!(from_str(r#""20160708T091048.090""#).is_err());
        assert!(from_str(r#""2000-00-00T00:00:00""#).is_err());
        assert!(from_str(r#""2000-02-30T00:00:00""#).is_err());
        assert!(from_str(r#""2001-02-29T00:00:00""#).is_err());
        assert!(from_str(r#""2002-02-28T24:00:00""#).is_err());
        assert!(from_str(r#""2002-02-28T23:60:00""#).is_err());
        assert!(from_str(r#""2002-02-28T23:59:61""#).is_err());
        assert!(from_str(r#""2016-07-08T09:10:48,090""#).is_err());
        assert!(from_str(r#""2016-07-08 09:10:48.090""#).is_err());
        assert!(from_str(r#""2016-007-08T09:10:48.090""#).is_err());
        assert!(from_str(r#""yyyy-mm-ddThh:mm:ss.fffffffff""#).is_err());
        assert!(from_str(r#"20160708000000"#).is_err());
        assert!(from_str(r#"{}"#).is_err());
        // pre-0.3.0 rustc-serialize format is now invalid
        assert!(from_str(r#"{"date":{"ymdf":20},"time":{"secs":0,"frac":0}}"#).is_err());
        assert!(from_str(r#"null"#).is_err());
    }

    // Bincode is relevant to test separately from JSON because
    // it is not self-describing.
    #[test]
    fn test_serde_bincode() {
        let dt = NaiveDate::from_ymd_opt(2016, 7, 8)
            .unwrap()
            .and_hms_milli_opt(9, 10, 48, 90)
            .unwrap();
        let encoded = serialize(&dt).unwrap();
        let decoded: NaiveDateTime = deserialize(&encoded).unwrap();
        assert_eq!(dt, decoded);
    }

    #[test]
    fn test_serde_bincode_optional() {
        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
        struct Test {
            one: Option<i64>,
            #[serde(with = "ts_nanoseconds_option")]
            two: Option<DateTime<Utc>>,
        }

        let expected = Test {
            one: Some(1),
            two: Some(Utc.with_ymd_and_hms(1970, 1, 1, 0, 1, 1).unwrap()),
        };
        let bytes: Vec<u8> = serialize(&expected).unwrap();
        let actual = deserialize::<Test>(&(bytes)).unwrap();

        assert_eq!(expected, actual);
    }
