
    use super::*;

    #[test]
    fn test_date_to_julian_day() {
        // Test known values
        assert_eq!(date_to_julian_day(2000, 1, 1), Some(2451545)); // J2000.0
        assert_eq!(date_to_julian_day(1970, 1, 1), Some(2440588)); // Unix epoch
        // Test overflow returns None
        assert_eq!(date_to_julian_day(i32::MAX, 1, 1), None);
    }

    #[test]
    fn test_date_to_unix_timestamp() {
        assert_eq!(date_to_unix_timestamp(1970, 1, 1), Some(0));
        assert_eq!(date_to_unix_timestamp(2000, 1, 1), Some(946684800));
        // Invalid dates
        assert_eq!(date_to_unix_timestamp(2000, 0, 1), None);
        assert_eq!(date_to_unix_timestamp(2000, 13, 1), None);
        assert_eq!(date_to_unix_timestamp(2000, 1, 0), None);
        assert_eq!(date_to_unix_timestamp(2000, 1, 32), None);
    }

    #[test]
    fn baseline_date_helpers() {
        assert_eq!(unix_timestamp_to_date(0), (1970, 1, 1));
        assert_eq!(unix_timestamp_to_date(946684800), (2000, 1, 1));
        assert_eq!(unix_timestamp_to_date(-1), (1969, 12, 31));
        assert_eq!(add_months((2024, 5, 31), -30), (2021, 12, 1));
        assert_eq!(add_months(add_months((2024, 3, 31), 30), -30), (2024, 4, 1));
        let timestamp = date_to_unix_timestamp(2040, 1, 1).unwrap();
        assert_eq!(
            add_months_utc(timestamp, -30),
            date_to_unix_timestamp(2037, 7, 1)
        );

        #[cfg(not(miri))]
        {
            let timestamp = date_to_unix_timestamp(2035, 8, 1).unwrap();
            assert_eq!(
                mktime_or_fallback(timestamp, 30, -1),
                date_to_unix_timestamp(2038, 2, 1)
            );
        }
    }

    #[cfg(not(miri))]
    #[test]
    fn local_date_uses_timezone() {
        const CHILD: &str = "OXC_BROWSERSLIST_LOCAL_DATE_TEST";
        if std::env::var_os(CHILD).is_some() {
            let timestamp = date_to_unix_timestamp(2018, 1, 29).unwrap();
            let timestamp = add_months_local(timestamp, -30).unwrap();
            assert_eq!(unix_timestamp_to_date(timestamp), (2015, 7, 28));
            return;
        }

        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                &format!(
                    "{}::local_date_uses_timezone",
                    module_path!().split_once("::").unwrap().1
                ),
            ])
            .env(CHILD, "1")
            .env("TZ", "PST8PDT")
            .status()
            .unwrap();
        assert!(status.success());
    }
