
        use crate::NaiveDate;

        #[test]
        fn test_serde_serialize() {
            assert_eq!(
                serde_json::to_string(&NaiveDate::from_ymd_opt(2014, 7, 24).unwrap()).ok(),
                Some(r#""2014-07-24""#.into())
            );
            assert_eq!(
                serde_json::to_string(&NaiveDate::from_ymd_opt(0, 1, 1).unwrap()).ok(),
                Some(r#""0000-01-01""#.into())
            );
            assert_eq!(
                serde_json::to_string(&NaiveDate::from_ymd_opt(-1, 12, 31).unwrap()).ok(),
                Some(r#""-0001-12-31""#.into())
            );
            assert_eq!(
                serde_json::to_string(&NaiveDate::MIN).ok(),
                Some(r#""-262143-01-01""#.into())
            );
            assert_eq!(
                serde_json::to_string(&NaiveDate::MAX).ok(),
                Some(r#""+262142-12-31""#.into())
            );
        }

        #[test]
        fn test_serde_deserialize() {
            let from_str = serde_json::from_str::<NaiveDate>;

            assert_eq!(
                from_str(r#""2016-07-08""#).ok(),
                Some(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap())
            );
            assert_eq!(
                from_str(r#""2016-7-8""#).ok(),
                Some(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap())
            );
            assert_eq!(
                from_str(r#""+002016-07-08""#).ok(),
                NaiveDate::from_ymd_opt(2016, 7, 8)
            );
            assert_eq!(
                from_str(r#""0000-01-01""#).ok(),
                Some(NaiveDate::from_ymd_opt(0, 1, 1).unwrap())
            );
            assert_eq!(
                from_str(r#""0-1-1""#).ok(),
                Some(NaiveDate::from_ymd_opt(0, 1, 1).unwrap())
            );
            assert_eq!(
                from_str(r#""-0001-12-31""#).ok(),
                Some(NaiveDate::from_ymd_opt(-1, 12, 31).unwrap())
            );
            assert_eq!(from_str(r#""-262143-01-01""#).ok(), Some(NaiveDate::MIN));
            assert_eq!(from_str(r#""+262142-12-31""#).ok(), Some(NaiveDate::MAX));

            // bad formats
            assert!(from_str(r#""""#).is_err());
            assert!(from_str(r#""20001231""#).is_err());
            assert!(from_str(r#""2000-00-00""#).is_err());
            assert!(from_str(r#""2000-02-30""#).is_err());
            assert!(from_str(r#""2001-02-29""#).is_err());
            assert!(from_str(r#""2002-002-28""#).is_err());
            assert!(from_str(r#""yyyy-mm-dd""#).is_err());
            assert!(from_str(r#"0"#).is_err());
            assert!(from_str(r#"20.01"#).is_err());
            let min = i32::MIN.to_string();
            assert!(from_str(&min).is_err());
            let max = i32::MAX.to_string();
            assert!(from_str(&max).is_err());
            let min = i64::MIN.to_string();
            assert!(from_str(&min).is_err());
            let max = i64::MAX.to_string();
            assert!(from_str(&max).is_err());
            assert!(from_str(r#"{}"#).is_err());
        }

        #[test]
        fn test_serde_bincode() {
            // Bincode is relevant to test separately from JSON because
            // it is not self-describing.
            use bincode::{deserialize, serialize};

            let d = NaiveDate::from_ymd_opt(2014, 7, 24).unwrap();
            let encoded = serialize(&d).unwrap();
            let decoded: NaiveDate = deserialize(&encoded).unwrap();
            assert_eq!(d, decoded);
        }
    
