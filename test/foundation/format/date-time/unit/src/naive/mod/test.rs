
    use crate::{NaiveDate, NaiveWeek, Weekday};
    use std::hash::{DefaultHasher, Hash, Hasher};
    #[test]
    fn test_naiveweek() {
        let date = NaiveDate::from_ymd_opt(2022, 5, 18).unwrap();
        let asserts = [
            (Weekday::Mon, "Mon 2022-05-16", "Sun 2022-05-22"),
            (Weekday::Tue, "Tue 2022-05-17", "Mon 2022-05-23"),
            (Weekday::Wed, "Wed 2022-05-18", "Tue 2022-05-24"),
            (Weekday::Thu, "Thu 2022-05-12", "Wed 2022-05-18"),
            (Weekday::Fri, "Fri 2022-05-13", "Thu 2022-05-19"),
            (Weekday::Sat, "Sat 2022-05-14", "Fri 2022-05-20"),
            (Weekday::Sun, "Sun 2022-05-15", "Sat 2022-05-21"),
        ];
        for (start, first_day, last_day) in asserts {
            let week = date.week(start);
            let days = week.days();
            assert_eq!(
                Ok(week.first_day()),
                NaiveDate::parse_from_str(first_day, "%a %Y-%m-%d")
            );
            assert_eq!(
                Ok(week.last_day()),
                NaiveDate::parse_from_str(last_day, "%a %Y-%m-%d")
            );
            assert!(days.contains(&date));
        }
    }

    #[test]
    fn test_naiveweek_min_max() {
        let date_max = NaiveDate::MAX;
        assert!(date_max.week(Weekday::Mon).first_day() <= date_max);
        let date_min = NaiveDate::MIN;
        assert!(date_min.week(Weekday::Mon).last_day() >= date_min);
    }

    #[test]
    fn test_naiveweek_checked_no_panic() {
        let date_max = NaiveDate::MAX;
        if let Some(last) = date_max.week(Weekday::Mon).checked_last_day() {
            assert!(last == date_max);
        }
        let date_min = NaiveDate::MIN;
        if let Some(first) = date_min.week(Weekday::Mon).checked_first_day() {
            assert!(first == date_min);
        }
        let _ = date_min.week(Weekday::Mon).checked_days();
        let _ = date_max.week(Weekday::Mon).checked_days();
    }

    #[test]
    fn test_naiveweek_eq() {
        let a = NaiveWeek {
            date: NaiveDate::from_ymd_opt(2025, 4, 3).unwrap(),
            start: Weekday::Mon,
        };
        let b = NaiveWeek {
            date: NaiveDate::from_ymd_opt(2025, 4, 4).unwrap(),
            start: Weekday::Mon,
        };
        assert_eq!(a, b);

        let c = NaiveWeek {
            date: NaiveDate::from_ymd_opt(2025, 4, 3).unwrap(),
            start: Weekday::Sun,
        };
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    #[test]
    fn test_naiveweek_hash() {
        let a = NaiveWeek {
            date: NaiveDate::from_ymd_opt(2025, 4, 3).unwrap(),
            start: Weekday::Mon,
        };
        let b = NaiveWeek {
            date: NaiveDate::from_ymd_opt(2025, 4, 4).unwrap(),
            start: Weekday::Mon,
        };
        let c = NaiveWeek {
            date: NaiveDate::from_ymd_opt(2025, 4, 3).unwrap(),
            start: Weekday::Sun,
        };

        let mut hasher = DefaultHasher::default();
        a.hash(&mut hasher);
        let a_hash = hasher.finish();

        hasher = DefaultHasher::default();
        b.hash(&mut hasher);
        let b_hash = hasher.finish();

        hasher = DefaultHasher::default();
        c.hash(&mut hasher);
        let c_hash = hasher.finish();

        assert_eq!(a_hash, b_hash);
        assert_ne!(b_hash, c_hash);
        assert_ne!(a_hash, c_hash);
    }
