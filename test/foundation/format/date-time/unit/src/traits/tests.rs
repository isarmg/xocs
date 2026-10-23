
    use super::Datelike;
    use crate::{Days, NaiveDate};

    /// Tests `Datelike::num_days_from_ce` against an alternative implementation.
    ///
    /// The alternative implementation is not as short as the current one but it is simpler to
    /// understand, with less unexplained magic constants.
    #[test]
    fn test_num_days_from_ce_against_alternative_impl() {
        /// Returns the number of multiples of `div` in the range `start..end`.
        ///
        /// If the range `start..end` is back-to-front, i.e. `start` is greater than `end`, the
        /// behaviour is defined by the following equation:
        /// `in_between(start, end, div) == - in_between(end, start, div)`.
        ///
        /// When `div` is 1, this is equivalent to `end - start`, i.e. the length of `start..end`.
        ///
        /// # Panics
        ///
        /// Panics if `div` is not positive.
        fn in_between(start: i32, end: i32, div: i32) -> i32 {
            assert!(div > 0, "in_between: nonpositive div = {div}");
            let start = (start.div_euclid(div), start.rem_euclid(div));
            let end = (end.div_euclid(div), end.rem_euclid(div));
            // The lowest multiple of `div` greater than or equal to `start`, divided.
            let start = start.0 + (start.1 != 0) as i32;
            // The lowest multiple of `div` greater than or equal to   `end`, divided.
            let end = end.0 + (end.1 != 0) as i32;
            end - start
        }

        /// Alternative implementation to `Datelike::num_days_from_ce`
        fn num_days_from_ce<Date: Datelike>(date: &Date) -> i32 {
            let year = date.year();
            let diff = move |div| in_between(1, year, div);
            // 365 days a year, one more in leap years. In the gregorian calendar, leap years are all
            // the multiples of 4 except multiples of 100 but including multiples of 400.
            date.ordinal() as i32 + 365 * diff(1) + diff(4) - diff(100) + diff(400)
        }

        for year in NaiveDate::MIN.year()..=NaiveDate::MAX.year() {
            let jan1_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
            assert_eq!(
                jan1_year.num_days_from_ce(),
                num_days_from_ce(&jan1_year),
                "on {jan1_year:?}"
            );
            let mid_year = jan1_year + Days::new(133);
            assert_eq!(
                mid_year.num_days_from_ce(),
                num_days_from_ce(&mid_year),
                "on {mid_year:?}"
            );
        }
    }

    #[test]
    fn test_num_days_in_month() {
        let feb_leap_year = NaiveDate::from_ymd_opt(2004, 2, 1).unwrap();
        assert_eq!(feb_leap_year.num_days_in_month(), 29);
        let feb = feb_leap_year.with_year(2005).unwrap();
        assert_eq!(feb.num_days_in_month(), 28);
        let march = feb.with_month(3).unwrap();
        assert_eq!(march.num_days_in_month(), 31);
    }
