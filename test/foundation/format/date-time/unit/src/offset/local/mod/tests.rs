
    use super::Local;
    use crate::offset::TimeZone;
    use crate::{Datelike, Days, Utc};
    #[test]
    fn verify_correct_offsets() {
        let now = Local::now();
        let from_local = Local.from_local_datetime(&now.naive_local()).unwrap();
        let from_utc = Local.from_utc_datetime(&now.naive_utc());

        assert_eq!(
            now.offset().local_minus_utc(),
            from_local.offset().local_minus_utc()
        );
        assert_eq!(
            now.offset().local_minus_utc(),
            from_utc.offset().local_minus_utc()
        );

        assert_eq!(now, from_local);
        assert_eq!(now, from_utc);
    }

    #[test]
    fn verify_correct_offsets_distant_past() {
        let distant_past = Local::now() - Days::new(365 * 500);
        let from_local = Local
            .from_local_datetime(&distant_past.naive_local())
            .unwrap();
        let from_utc = Local.from_utc_datetime(&distant_past.naive_utc());

        assert_eq!(
            distant_past.offset().local_minus_utc(),
            from_local.offset().local_minus_utc()
        );
        assert_eq!(
            distant_past.offset().local_minus_utc(),
            from_utc.offset().local_minus_utc()
        );

        assert_eq!(distant_past, from_local);
        assert_eq!(distant_past, from_utc);
    }

    #[test]
    fn verify_correct_offsets_distant_future() {
        let distant_future = Local::now() + Days::new(365 * 35000);
        let from_local = Local
            .from_local_datetime(&distant_future.naive_local())
            .unwrap();
        let from_utc = Local.from_utc_datetime(&distant_future.naive_utc());

        assert_eq!(
            distant_future.offset().local_minus_utc(),
            from_local.offset().local_minus_utc()
        );
        assert_eq!(
            distant_future.offset().local_minus_utc(),
            from_utc.offset().local_minus_utc()
        );

        assert_eq!(distant_future, from_local);
        assert_eq!(distant_future, from_utc);
    }

    #[test]
    fn test_local_date_sanity_check() {
        // issue #27
        assert_eq!(
            Local.with_ymd_and_hms(2999, 12, 28, 0, 0, 0).unwrap().day(),
            28
        );
    }

    #[test]
    fn test_leap_second() {
        // issue #123
        let today = Utc::now().date_naive();

        if let Some(dt) = today.and_hms_milli_opt(15, 2, 59, 1000) {
            let timestr = dt.time().to_string();
            // the OS API may or may not support the leap second,
            // but there are only two sensible options.
            assert!(
                timestr == "15:02:60" || timestr == "15:03:00",
                "unexpected timestr {timestr:?}"
            );
        }

        if let Some(dt) = today.and_hms_milli_opt(15, 2, 3, 1234) {
            let timestr = dt.time().to_string();
            assert!(
                timestr == "15:02:03.234" || timestr == "15:02:04.234",
                "unexpected timestr {timestr:?}"
            );
        }
    }
