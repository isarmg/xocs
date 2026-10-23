
    #[cfg(feature = "clock")]
    use crate::Local;
    use crate::{DateTime, FixedOffset, TimeZone, Utc};
    use core::fmt;

    #[test]
    fn test_serde_serialize() {
        assert_eq!(
            serde_json::to_string(&Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()).ok(),
            Some(r#""2014-07-24T12:34:06Z""#.to_owned())
        );
        assert_eq!(
            serde_json::to_string(
                &FixedOffset::east_opt(3660)
                    .unwrap()
                    .with_ymd_and_hms(2014, 7, 24, 12, 34, 6)
                    .unwrap()
            )
            .ok(),
            Some(r#""2014-07-24T12:34:06+01:01""#.to_owned())
        );
        assert_eq!(
            serde_json::to_string(
                &FixedOffset::east_opt(3650)
                    .unwrap()
                    .with_ymd_and_hms(2014, 7, 24, 12, 34, 6)
                    .unwrap()
            )
            .ok(),
            // An offset with seconds is not allowed by RFC 3339, so we round it to the nearest minute.
            // In this case `+01:00:50` becomes `+01:01`
            Some(r#""2014-07-24T12:34:06+01:01""#.to_owned())
        );
    }

    #[test]
    fn test_serde_deserialize() {
        // should check against the offset as well (the normal DateTime comparison will ignore them)
        fn norm<Tz: TimeZone>(dt: &Option<DateTime<Tz>>) -> Option<(&DateTime<Tz>, &Tz::Offset)> {
            dt.as_ref().map(|dt| (dt, dt.offset()))
        }

        let dt: Option<DateTime<Utc>> = serde_json::from_str(r#""2014-07-24T12:34:06Z""#).ok();
        assert_eq!(
            norm(&dt),
            norm(&Some(Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()))
        );
        let dt: Option<DateTime<Utc>> = serde_json::from_str(r#""2014-07-24T13:57:06+01:23""#).ok();
        assert_eq!(
            norm(&dt),
            norm(&Some(Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap()))
        );

        let dt: Option<DateTime<FixedOffset>> =
            serde_json::from_str(r#""2014-07-24T12:34:06Z""#).ok();
        assert_eq!(
            norm(&dt),
            norm(&Some(
                FixedOffset::east_opt(0)
                    .unwrap()
                    .with_ymd_and_hms(2014, 7, 24, 12, 34, 6)
                    .unwrap()
            ))
        );
        let dt: Option<DateTime<FixedOffset>> =
            serde_json::from_str(r#""2014-07-24T13:57:06+01:23""#).ok();
        assert_eq!(
            norm(&dt),
            norm(&Some(
                FixedOffset::east_opt(60 * 60 + 23 * 60)
                    .unwrap()
                    .with_ymd_and_hms(2014, 7, 24, 13, 57, 6)
                    .unwrap()
            ))
        );

        // we don't know the exact local offset but we can check that
        // the conversion didn't change the instant itself
        #[cfg(feature = "clock")]
        {
            let dt: DateTime<Local> =
                serde_json::from_str(r#""2014-07-24T12:34:06Z""#).expect("local should parse");
            assert_eq!(dt, Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap());

            let dt: DateTime<Local> = serde_json::from_str(r#""2014-07-24T13:57:06+01:23""#)
                .expect("local should parse with offset");
            assert_eq!(dt, Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap());
        }

        assert!(serde_json::from_str::<DateTime<Utc>>(r#""2014-07-32T12:34:06Z""#).is_err());
        assert!(
            serde_json::from_str::<DateTime<FixedOffset>>(r#""2014-07-32T12:34:06Z""#).is_err()
        );
    }

    #[test]
    fn test_serde_bincode() {
        // Bincode is relevant to test separately from JSON because
        // it is not self-describing.
        use bincode::{deserialize, serialize};

        let dt = Utc.with_ymd_and_hms(2014, 7, 24, 12, 34, 6).unwrap();
        let encoded = serialize(&dt).unwrap();
        let decoded: DateTime<Utc> = deserialize(&encoded).unwrap();
        assert_eq!(dt, decoded);
        assert_eq!(dt.offset(), decoded.offset());
    }

    #[test]
    fn test_serde_no_offset_debug() {
        use crate::{MappedLocalTime, NaiveDate, NaiveDateTime, Offset};
        use core::fmt::Debug;

        #[derive(Clone)]
        struct TestTimeZone;
        impl Debug for TestTimeZone {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "TEST")
            }
        }
        impl TimeZone for TestTimeZone {
            type Offset = TestTimeZone;
            fn from_offset(_state: &TestTimeZone) -> TestTimeZone {
                TestTimeZone
            }
            fn offset_from_local_date(&self, _local: &NaiveDate) -> MappedLocalTime<TestTimeZone> {
                MappedLocalTime::Single(TestTimeZone)
            }
            fn offset_from_local_datetime(
                &self,
                _local: &NaiveDateTime,
            ) -> MappedLocalTime<TestTimeZone> {
                MappedLocalTime::Single(TestTimeZone)
            }
            fn offset_from_utc_date(&self, _utc: &NaiveDate) -> TestTimeZone {
                TestTimeZone
            }
            fn offset_from_utc_datetime(&self, _utc: &NaiveDateTime) -> TestTimeZone {
                TestTimeZone
            }
        }
        impl Offset for TestTimeZone {
            fn fix(&self) -> FixedOffset {
                FixedOffset::east_opt(15 * 60 * 60).unwrap()
            }
        }

        let tz = TestTimeZone;
        assert_eq!(format!("{:?}", &tz), "TEST");

        let dt = tz.with_ymd_and_hms(2023, 4, 24, 21, 10, 33).unwrap();
        let encoded = serde_json::to_string(&dt).unwrap();
        dbg!(&encoded);
        let decoded: DateTime<FixedOffset> = serde_json::from_str(&encoded).unwrap();
        assert_eq!(dt, decoded);
        assert_eq!(dt.offset().fix(), *decoded.offset());
    }
