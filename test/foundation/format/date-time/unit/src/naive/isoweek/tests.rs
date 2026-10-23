
    use crate::naive::date::{self, NaiveDate};
    use crate::Datelike;

    #[test]
    fn test_iso_week_extremes() {
        let minweek = NaiveDate::MIN.iso_week();
        let maxweek = NaiveDate::MAX.iso_week();

        assert_eq!(minweek.year(), date::MIN_YEAR);
        assert_eq!(minweek.week(), 1);
        assert_eq!(minweek.week0(), 0);
        #[cfg(feature = "alloc")]
        assert_eq!(
            format!("{minweek:?}"),
            NaiveDate::MIN.format("%G-W%V").to_string()
        );

        assert_eq!(maxweek.year(), date::MAX_YEAR + 1);
        assert_eq!(maxweek.week(), 1);
        assert_eq!(maxweek.week0(), 0);
        #[cfg(feature = "alloc")]
        assert_eq!(
            format!("{maxweek:?}"),
            NaiveDate::MAX.format("%G-W%V").to_string()
        );
    }

    #[test]
    fn test_iso_week_equivalence_for_first_week() {
        let monday = NaiveDate::from_ymd_opt(2024, 12, 30).unwrap();
        let friday = NaiveDate::from_ymd_opt(2025, 1, 3).unwrap();

        assert_eq!(monday.iso_week(), friday.iso_week());
    }

    #[test]
    fn test_iso_week_equivalence_for_last_week() {
        let monday = NaiveDate::from_ymd_opt(2026, 12, 28).unwrap();
        let friday = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();

        assert_eq!(monday.iso_week(), friday.iso_week());
    }

    #[test]
    fn test_iso_week_ordering_for_first_week() {
        let monday = NaiveDate::from_ymd_opt(2024, 12, 30).unwrap();
        let friday = NaiveDate::from_ymd_opt(2025, 1, 3).unwrap();

        assert!(monday.iso_week() >= friday.iso_week());
        assert!(monday.iso_week() <= friday.iso_week());
    }

    #[test]
    fn test_iso_week_ordering_for_last_week() {
        let monday = NaiveDate::from_ymd_opt(2026, 12, 28).unwrap();
        let friday = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();

        assert!(monday.iso_week() >= friday.iso_week());
        assert!(monday.iso_week() <= friday.iso_week());
    }
