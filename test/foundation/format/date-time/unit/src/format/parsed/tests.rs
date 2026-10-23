
    use super::super::{IMPOSSIBLE, NOT_ENOUGH, OUT_OF_RANGE};
    use super::Parsed;
    use crate::naive::{NaiveDate, NaiveTime};
    use crate::offset::{FixedOffset, TimeZone, Utc};
    use crate::Datelike;
    use crate::Weekday::*;

    #[test]
    fn test_parsed_set_fields() {
        // year*, isoyear*
        let mut p = Parsed::new();
        assert_eq!(p.set_year(1987), Ok(()));
        assert_eq!(p.set_year(1986), Err(IMPOSSIBLE));
        assert_eq!(p.set_year(1988), Err(IMPOSSIBLE));
        assert_eq!(p.set_year(1987), Ok(()));
        assert_eq!(p.set_year_div_100(20), Ok(())); // independent to `year`
        assert_eq!(p.set_year_div_100(21), Err(IMPOSSIBLE));
        assert_eq!(p.set_year_div_100(19), Err(IMPOSSIBLE));
        assert_eq!(p.set_year_mod_100(37), Ok(())); // ditto
        assert_eq!(p.set_year_mod_100(38), Err(IMPOSSIBLE));
        assert_eq!(p.set_year_mod_100(36), Err(IMPOSSIBLE));

        let mut p = Parsed::new();
        assert_eq!(p.set_year(0), Ok(()));
        assert_eq!(p.set_year_div_100(0), Ok(()));
        assert_eq!(p.set_year_mod_100(0), Ok(()));

        let mut p = Parsed::new();
        assert_eq!(p.set_year_div_100(-1), Err(OUT_OF_RANGE));
        assert_eq!(p.set_year_mod_100(-1), Err(OUT_OF_RANGE));
        assert_eq!(p.set_year(-1), Ok(()));
        assert_eq!(p.set_year(-2), Err(IMPOSSIBLE));
        assert_eq!(p.set_year(0), Err(IMPOSSIBLE));

        let mut p = Parsed::new();
        assert_eq!(p.set_year_div_100(0x1_0000_0008), Err(OUT_OF_RANGE));
        assert_eq!(p.set_year_div_100(8), Ok(()));
        assert_eq!(p.set_year_div_100(0x1_0000_0008), Err(OUT_OF_RANGE));

        // month, week*, isoweek, ordinal, day, minute, second, nanosecond, offset
        let mut p = Parsed::new();
        assert_eq!(p.set_month(7), Ok(()));
        assert_eq!(p.set_month(1), Err(IMPOSSIBLE));
        assert_eq!(p.set_month(6), Err(IMPOSSIBLE));
        assert_eq!(p.set_month(8), Err(IMPOSSIBLE));
        assert_eq!(p.set_month(12), Err(IMPOSSIBLE));

        let mut p = Parsed::new();
        assert_eq!(p.set_month(8), Ok(()));
        assert_eq!(p.set_month(0x1_0000_0008), Err(OUT_OF_RANGE));

        // hour
        let mut p = Parsed::new();
        assert_eq!(p.set_hour(12), Ok(()));
        assert_eq!(p.set_hour(11), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour(13), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour(12), Ok(()));
        assert_eq!(p.set_ampm(false), Err(IMPOSSIBLE));
        assert_eq!(p.set_ampm(true), Ok(()));
        assert_eq!(p.set_hour12(12), Ok(()));
        assert_eq!(p.set_hour12(0), Err(OUT_OF_RANGE)); // requires canonical representation
        assert_eq!(p.set_hour12(1), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour12(11), Err(IMPOSSIBLE));

        let mut p = Parsed::new();
        assert_eq!(p.set_ampm(true), Ok(()));
        assert_eq!(p.set_hour12(7), Ok(()));
        assert_eq!(p.set_hour(7), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour(18), Err(IMPOSSIBLE));
        assert_eq!(p.set_hour(19), Ok(()));

        // timestamp
        let mut p = Parsed::new();
        assert_eq!(p.set_timestamp(1_234_567_890), Ok(()));
        assert_eq!(p.set_timestamp(1_234_567_889), Err(IMPOSSIBLE));
        assert_eq!(p.set_timestamp(1_234_567_891), Err(IMPOSSIBLE));
    }

    #[test]
    fn test_parsed_set_range() {
        assert_eq!(
            Parsed::new().set_year(i32::MIN as i64 - 1),
            Err(OUT_OF_RANGE)
        );
        assert!(Parsed::new().set_year(i32::MIN as i64).is_ok());
        assert!(Parsed::new().set_year(i32::MAX as i64).is_ok());
        assert_eq!(
            Parsed::new().set_year(i32::MAX as i64 + 1),
            Err(OUT_OF_RANGE)
        );

        assert_eq!(Parsed::new().set_year_div_100(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_year_div_100(0).is_ok());
        assert!(Parsed::new().set_year_div_100(i32::MAX as i64).is_ok());
        assert_eq!(
            Parsed::new().set_year_div_100(i32::MAX as i64 + 1),
            Err(OUT_OF_RANGE)
        );

        assert_eq!(Parsed::new().set_year_mod_100(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_year_mod_100(0).is_ok());
        assert!(Parsed::new().set_year_mod_100(99).is_ok());
        assert_eq!(Parsed::new().set_year_mod_100(100), Err(OUT_OF_RANGE));

        assert_eq!(
            Parsed::new().set_isoyear(i32::MIN as i64 - 1),
            Err(OUT_OF_RANGE)
        );
        assert!(Parsed::new().set_isoyear(i32::MIN as i64).is_ok());
        assert!(Parsed::new().set_isoyear(i32::MAX as i64).is_ok());
        assert_eq!(
            Parsed::new().set_isoyear(i32::MAX as i64 + 1),
            Err(OUT_OF_RANGE)
        );

        assert_eq!(Parsed::new().set_isoyear_div_100(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_isoyear_div_100(0).is_ok());
        assert!(Parsed::new().set_isoyear_div_100(99).is_ok());
        assert_eq!(
            Parsed::new().set_isoyear_div_100(i32::MAX as i64 + 1),
            Err(OUT_OF_RANGE)
        );

        assert_eq!(Parsed::new().set_isoyear_mod_100(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_isoyear_mod_100(0).is_ok());
        assert!(Parsed::new().set_isoyear_mod_100(99).is_ok());
        assert_eq!(Parsed::new().set_isoyear_mod_100(100), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_quarter(0), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_quarter(1).is_ok());
        assert!(Parsed::new().set_quarter(4).is_ok());
        assert_eq!(Parsed::new().set_quarter(5), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_month(0), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_month(1).is_ok());
        assert!(Parsed::new().set_month(12).is_ok());
        assert_eq!(Parsed::new().set_month(13), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_week_from_sun(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_week_from_sun(0).is_ok());
        assert!(Parsed::new().set_week_from_sun(53).is_ok());
        assert_eq!(Parsed::new().set_week_from_sun(54), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_week_from_mon(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_week_from_mon(0).is_ok());
        assert!(Parsed::new().set_week_from_mon(53).is_ok());
        assert_eq!(Parsed::new().set_week_from_mon(54), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_isoweek(0), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_isoweek(1).is_ok());
        assert!(Parsed::new().set_isoweek(53).is_ok());
        assert_eq!(Parsed::new().set_isoweek(54), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_ordinal(0), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_ordinal(1).is_ok());
        assert!(Parsed::new().set_ordinal(366).is_ok());
        assert_eq!(Parsed::new().set_ordinal(367), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_day(0), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_day(1).is_ok());
        assert!(Parsed::new().set_day(31).is_ok());
        assert_eq!(Parsed::new().set_day(32), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_hour12(0), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_hour12(1).is_ok());
        assert!(Parsed::new().set_hour12(12).is_ok());
        assert_eq!(Parsed::new().set_hour12(13), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_hour(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_hour(0).is_ok());
        assert!(Parsed::new().set_hour(23).is_ok());
        assert_eq!(Parsed::new().set_hour(24), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_minute(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_minute(0).is_ok());
        assert!(Parsed::new().set_minute(59).is_ok());
        assert_eq!(Parsed::new().set_minute(60), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_second(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_second(0).is_ok());
        assert!(Parsed::new().set_second(60).is_ok());
        assert_eq!(Parsed::new().set_second(61), Err(OUT_OF_RANGE));

        assert_eq!(Parsed::new().set_nanosecond(-1), Err(OUT_OF_RANGE));
        assert!(Parsed::new().set_nanosecond(0).is_ok());
        assert!(Parsed::new().set_nanosecond(999_999_999).is_ok());
        assert_eq!(
            Parsed::new().set_nanosecond(1_000_000_000),
            Err(OUT_OF_RANGE)
        );

        assert!(Parsed::new().set_timestamp(i64::MIN).is_ok());
        assert!(Parsed::new().set_timestamp(i64::MAX).is_ok());

        assert_eq!(
            Parsed::new().set_offset(i32::MIN as i64 - 1),
            Err(OUT_OF_RANGE)
        );
        assert!(Parsed::new().set_offset(i32::MIN as i64).is_ok());
        assert!(Parsed::new().set_offset(i32::MAX as i64).is_ok());
        assert_eq!(
            Parsed::new().set_offset(i32::MAX as i64 + 1),
            Err(OUT_OF_RANGE)
        );
    }

    #[test]
    fn test_parsed_to_naive_date() {
        macro_rules! parse {
            ($($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_naive_date()
            )
        }

        let ymd = |y, m, d| Ok(NaiveDate::from_ymd_opt(y, m, d).unwrap());

        // ymd: omission of fields
        assert_eq!(parse!(), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 1984), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 1984, month: 1), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 1984, month: 1, day: 2), ymd(1984, 1, 2));
        assert_eq!(parse!(year: 1984, day: 2), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_div_100: 19), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_div_100: 19, year_mod_100: 84), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 84, month: 1),
            Err(NOT_ENOUGH)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 84, month: 1, day: 2),
            ymd(1984, 1, 2)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 84, day: 2),
            Err(NOT_ENOUGH)
        );
        assert_eq!(parse!(year_div_100: 19, month: 1, day: 2), Err(NOT_ENOUGH));
        assert_eq!(parse!(year_mod_100: 70, month: 1, day: 2), ymd(1970, 1, 2));
        assert_eq!(parse!(year_mod_100: 69, month: 1, day: 2), ymd(2069, 1, 2));

        // ymd: out-of-range conditions
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 84, month: 2, day: 29),
            ymd(1984, 2, 29)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 2, day: 29),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 13, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 12, day: 31),
            ymd(1983, 12, 31)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 12, day: 32),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 83, month: 12, day: 0),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: 100, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 19, year_mod_100: -1, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year_div_100: 0, year_mod_100: 0, month: 1, day: 1),
            ymd(0, 1, 1)
        );
        assert_eq!(
            parse!(year_div_100: -1, year_mod_100: 42, month: 1, day: 1),
            Err(IMPOSSIBLE)
        );
        let max_year = NaiveDate::MAX.year();
        assert_eq!(
            parse!(year_div_100: max_year / 100,
                          year_mod_100: max_year % 100, month: 1, day: 1),
            ymd(max_year, 1, 1)
        );
        assert_eq!(
            parse!(year_div_100: (max_year + 1) / 100,
                          year_mod_100: (max_year + 1) % 100, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );

        // ymd: conflicting inputs
        assert_eq!(
            parse!(year: 1984, year_div_100: 19, month: 1, day: 1),
            ymd(1984, 1, 1)
        );
        assert_eq!(
            parse!(year: 1984, year_div_100: 20, month: 1, day: 1),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 1984, year_mod_100: 84, month: 1, day: 1),
            ymd(1984, 1, 1)
        );
        assert_eq!(
            parse!(year: 1984, year_mod_100: 83, month: 1, day: 1),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 1984, year_div_100: 19, year_mod_100: 84, month: 1, day: 1),
            ymd(1984, 1, 1)
        );
        assert_eq!(
            parse!(year: 1984, year_div_100: 18, year_mod_100: 94, month: 1, day: 1),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 1984, year_div_100: 18, year_mod_100: 184, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year: -1, year_div_100: 0, year_mod_100: -1, month: 1, day: 1),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year: -1, year_div_100: -1, year_mod_100: 99, month: 1, day: 1),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: -1, year_div_100: 0, month: 1, day: 1),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: -1, year_mod_100: 99, month: 1, day: 1),
            Err(IMPOSSIBLE)
        );

        // quarters
        assert_eq!(parse!(year: 2000, quarter: 1), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(year: 2000, quarter: 1, month: 1, day: 1),
            ymd(2000, 1, 1)
        );
        assert_eq!(
            parse!(year: 2000, quarter: 2, month: 4, day: 1),
            ymd(2000, 4, 1)
        );
        assert_eq!(
            parse!(year: 2000, quarter: 3, month: 7, day: 1),
            ymd(2000, 7, 1)
        );
        assert_eq!(
            parse!(year: 2000, quarter: 4, month: 10, day: 1),
            ymd(2000, 10, 1)
        );

        // quarter: conflicting inputs
        assert_eq!(
            parse!(year: 2000, quarter: 2, month: 3, day: 31),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2000, quarter: 4, month: 3, day: 31),
            Err(IMPOSSIBLE)
        );

        // weekdates
        assert_eq!(parse!(year: 2000, week_from_mon: 0), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 2000, week_from_sun: 0), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 2000, weekday: Sun), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(year: 2000, week_from_mon: 0, weekday: Fri),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 0, weekday: Fri),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 0, weekday: Sat),
            ymd(2000, 1, 1)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 0, weekday: Sat),
            ymd(2000, 1, 1)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 0, weekday: Sun),
            ymd(2000, 1, 2)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 1, weekday: Sun),
            ymd(2000, 1, 2)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, weekday: Mon),
            ymd(2000, 1, 3)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 1, weekday: Mon),
            ymd(2000, 1, 3)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, weekday: Sat),
            ymd(2000, 1, 8)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 1, weekday: Sat),
            ymd(2000, 1, 8)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, weekday: Sun),
            ymd(2000, 1, 9)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 2, weekday: Sun),
            ymd(2000, 1, 9)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 2, weekday: Mon),
            ymd(2000, 1, 10)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 52, weekday: Sat),
            ymd(2000, 12, 30)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 53, weekday: Sun),
            ymd(2000, 12, 31)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 53, weekday: Mon),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2000, week_from_sun: 0xffffffff, weekday: Mon),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(year: 2006, week_from_sun: 0, weekday: Sat),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2006, week_from_sun: 1, weekday: Sun),
            ymd(2006, 1, 1)
        );

        // weekdates: conflicting inputs
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, week_from_sun: 1, weekday: Sat),
            ymd(2000, 1, 8)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, week_from_sun: 2, weekday: Sun),
            ymd(2000, 1, 9)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 1, week_from_sun: 1, weekday: Sun),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2000, week_from_mon: 2, week_from_sun: 2, weekday: Sun),
            Err(IMPOSSIBLE)
        );

        // ISO weekdates
        assert_eq!(parse!(isoyear: 2004, isoweek: 53), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(isoyear: 2004, isoweek: 53, weekday: Fri),
            ymd(2004, 12, 31)
        );
        assert_eq!(
            parse!(isoyear: 2004, isoweek: 53, weekday: Sat),
            ymd(2005, 1, 1)
        );
        assert_eq!(
            parse!(isoyear: 2004, isoweek: 0xffffffff, weekday: Sat),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(isoyear: 2005, isoweek: 0, weekday: Thu),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(isoyear: 2005, isoweek: 5, weekday: Thu),
            ymd(2005, 2, 3)
        );
        assert_eq!(parse!(isoyear: 2005, weekday: Thu), Err(NOT_ENOUGH));

        // year and ordinal
        assert_eq!(parse!(ordinal: 123), Err(NOT_ENOUGH));
        assert_eq!(parse!(year: 2000, ordinal: 0), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2000, ordinal: 1), ymd(2000, 1, 1));
        assert_eq!(parse!(year: 2000, ordinal: 60), ymd(2000, 2, 29));
        assert_eq!(parse!(year: 2000, ordinal: 61), ymd(2000, 3, 1));
        assert_eq!(parse!(year: 2000, ordinal: 366), ymd(2000, 12, 31));
        assert_eq!(parse!(year: 2000, ordinal: 367), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2000, ordinal: 0xffffffff), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2100, ordinal: 0), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2100, ordinal: 1), ymd(2100, 1, 1));
        assert_eq!(parse!(year: 2100, ordinal: 59), ymd(2100, 2, 28));
        assert_eq!(parse!(year: 2100, ordinal: 60), ymd(2100, 3, 1));
        assert_eq!(parse!(year: 2100, ordinal: 365), ymd(2100, 12, 31));
        assert_eq!(parse!(year: 2100, ordinal: 366), Err(OUT_OF_RANGE));
        assert_eq!(parse!(year: 2100, ordinal: 0xffffffff), Err(OUT_OF_RANGE));

        // more complex cases
        assert_eq!(
            parse!(year: 2014, month: 12, day: 31, ordinal: 365, isoyear: 2015, isoweek: 1,
                          week_from_sun: 52, week_from_mon: 52, weekday: Wed),
            ymd(2014, 12, 31)
        );
        assert_eq!(
            parse!(year: 2014, month: 12, ordinal: 365, isoyear: 2015, isoweek: 1,
                          week_from_sun: 52, week_from_mon: 52),
            ymd(2014, 12, 31)
        );
        assert_eq!(
            parse!(year: 2014, month: 12, day: 31, ordinal: 365, isoyear: 2014, isoweek: 53,
                          week_from_sun: 52, week_from_mon: 52, weekday: Wed),
            Err(IMPOSSIBLE)
        ); // no ISO week date 2014-W53-3
        assert_eq!(
            parse!(year: 2012, isoyear: 2015, isoweek: 1,
                          week_from_sun: 52, week_from_mon: 52),
            Err(NOT_ENOUGH)
        ); // ambiguous (2014-12-29, 2014-12-30, 2014-12-31)
        assert_eq!(
            parse!(year_div_100: 20, isoyear_mod_100: 15, ordinal: 366),
            Err(NOT_ENOUGH)
        );
        // technically unique (2014-12-31) but Chrono gives up
    }

    #[test]
    fn test_parsed_to_naive_time() {
        macro_rules! parse {
            ($($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_naive_time()
            )
        }

        let hms = |h, m, s| Ok(NaiveTime::from_hms_opt(h, m, s).unwrap());
        let hmsn = |h, m, s, n| Ok(NaiveTime::from_hms_nano_opt(h, m, s, n).unwrap());

        // omission of fields
        assert_eq!(parse!(), Err(NOT_ENOUGH));
        assert_eq!(parse!(hour_div_12: 0), Err(NOT_ENOUGH));
        assert_eq!(parse!(hour_div_12: 0, hour_mod_12: 1), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23),
            hms(1, 23, 0)
        );
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 45),
            hms(1, 23, 45)
        );
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 45,
                          nanosecond: 678_901_234),
            hmsn(1, 23, 45, 678_901_234)
        );
        assert_eq!(
            parse!(hour_div_12: 1, hour_mod_12: 11, minute: 45, second: 6),
            hms(23, 45, 6)
        );
        assert_eq!(parse!(hour_mod_12: 1, minute: 23), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, nanosecond: 456_789_012),
            Err(NOT_ENOUGH)
        );

        // out-of-range conditions
        assert_eq!(
            parse!(hour_div_12: 2, hour_mod_12: 0, minute: 0),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(hour_div_12: 1, hour_mod_12: 12, minute: 0),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 60),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 61),
            Err(OUT_OF_RANGE)
        );
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 34,
                          nanosecond: 1_000_000_000),
            Err(OUT_OF_RANGE)
        );

        // leap seconds
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 60),
            hmsn(1, 23, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(hour_div_12: 0, hour_mod_12: 1, minute: 23, second: 60,
                          nanosecond: 999_999_999),
            hmsn(1, 23, 59, 1_999_999_999)
        );
    }

    #[test]
    fn test_parsed_to_naive_datetime_with_offset() {
        macro_rules! parse {
            (offset = $offset:expr; $($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_naive_datetime_with_offset($offset)
            );
            ($($k:ident: $v:expr),*) => (parse!(offset = 0; $($k: $v),*))
        }

        let ymdhms = |y, m, d, h, n, s| {
            Ok(NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_opt(h, n, s)
                .unwrap())
        };
        let ymdhmsn = |y, m, d, h, n, s, nano| {
            Ok(NaiveDate::from_ymd_opt(y, m, d)
                .unwrap()
                .and_hms_nano_opt(h, n, s, nano)
                .unwrap())
        };

        // omission of fields
        assert_eq!(parse!(), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(year: 2015, month: 1, day: 30,
                          hour_div_12: 1, hour_mod_12: 2, minute: 38),
            ymdhms(2015, 1, 30, 14, 38, 0)
        );
        assert_eq!(
            parse!(year: 1997, month: 1, day: 30,
                          hour_div_12: 1, hour_mod_12: 2, minute: 38, second: 5),
            ymdhms(1997, 1, 30, 14, 38, 5)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 34, hour_div_12: 0, hour_mod_12: 5,
                          minute: 6, second: 7, nanosecond: 890_123_456),
            ymdhmsn(2012, 2, 3, 5, 6, 7, 890_123_456)
        );
        assert_eq!(parse!(timestamp: 0), ymdhms(1970, 1, 1, 0, 0, 0));
        assert_eq!(
            parse!(timestamp: 1, nanosecond: 0),
            ymdhms(1970, 1, 1, 0, 0, 1)
        );
        assert_eq!(
            parse!(timestamp: 1, nanosecond: 1),
            ymdhmsn(1970, 1, 1, 0, 0, 1, 1)
        );
        assert_eq!(
            parse!(timestamp: 1_420_000_000),
            ymdhms(2014, 12, 31, 4, 26, 40)
        );
        assert_eq!(
            parse!(timestamp: -0x1_0000_0000),
            ymdhms(1833, 11, 24, 17, 31, 44)
        );

        // full fields
        assert_eq!(
            parse!(year: 2014, year_div_100: 20, year_mod_100: 14, month: 12, day: 31,
                          ordinal: 365, isoyear: 2015, isoyear_div_100: 20, isoyear_mod_100: 15,
                          isoweek: 1, week_from_sun: 52, week_from_mon: 52, weekday: Wed,
                          hour_div_12: 0, hour_mod_12: 4, minute: 26, second: 40,
                          nanosecond: 12_345_678, timestamp: 1_420_000_000),
            ymdhmsn(2014, 12, 31, 4, 26, 40, 12_345_678)
        );
        assert_eq!(
            parse!(year: 2014, year_div_100: 20, year_mod_100: 14, month: 12, day: 31,
                          ordinal: 365, isoyear: 2015, isoyear_div_100: 20, isoyear_mod_100: 15,
                          isoweek: 1, week_from_sun: 52, week_from_mon: 52, weekday: Wed,
                          hour_div_12: 0, hour_mod_12: 4, minute: 26, second: 40,
                          nanosecond: 12_345_678, timestamp: 1_419_999_999),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(offset = 32400;
                          year: 2014, year_div_100: 20, year_mod_100: 14, month: 12, day: 31,
                          ordinal: 365, isoyear: 2015, isoyear_div_100: 20, isoyear_mod_100: 15,
                          isoweek: 1, week_from_sun: 52, week_from_mon: 52, weekday: Wed,
                          hour_div_12: 0, hour_mod_12: 4, minute: 26, second: 40,
                          nanosecond: 12_345_678, timestamp: 1_419_967_600),
            ymdhmsn(2014, 12, 31, 4, 26, 40, 12_345_678)
        );

        // more timestamps
        let max_days_from_year_1970 =
            NaiveDate::MAX.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let year_0_from_year_1970 = NaiveDate::from_ymd_opt(0, 1, 1)
            .unwrap()
            .signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let min_days_from_year_1970 =
            NaiveDate::MIN.signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        assert_eq!(
            parse!(timestamp: min_days_from_year_1970.num_seconds()),
            ymdhms(NaiveDate::MIN.year(), 1, 1, 0, 0, 0)
        );
        assert_eq!(
            parse!(timestamp: year_0_from_year_1970.num_seconds()),
            ymdhms(0, 1, 1, 0, 0, 0)
        );
        assert_eq!(
            parse!(timestamp: max_days_from_year_1970.num_seconds() + 86399),
            ymdhms(NaiveDate::MAX.year(), 12, 31, 23, 59, 59)
        );

        // leap seconds #1: partial fields
        assert_eq!(
            parse!(second: 59, timestamp: 1_341_100_798),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(second: 59, timestamp: 1_341_100_799),
            ymdhms(2012, 6, 30, 23, 59, 59)
        );
        assert_eq!(
            parse!(second: 59, timestamp: 1_341_100_800),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(second: 60, timestamp: 1_341_100_799),
            ymdhmsn(2012, 6, 30, 23, 59, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(second: 60, timestamp: 1_341_100_800),
            ymdhmsn(2012, 6, 30, 23, 59, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(second: 0, timestamp: 1_341_100_800),
            ymdhms(2012, 7, 1, 0, 0, 0)
        );
        assert_eq!(parse!(second: 1, timestamp: 1_341_100_800), Err(IMPOSSIBLE));
        assert_eq!(
            parse!(second: 60, timestamp: 1_341_100_801),
            Err(IMPOSSIBLE)
        );

        // leap seconds #2: full fields
        // we need to have separate tests for them since it uses another control flow.
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 59, timestamp: 1_341_100_798),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 59, timestamp: 1_341_100_799),
            ymdhms(2012, 6, 30, 23, 59, 59)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 59, timestamp: 1_341_100_800),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 60, timestamp: 1_341_100_799),
            ymdhmsn(2012, 6, 30, 23, 59, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 60, timestamp: 1_341_100_800),
            ymdhmsn(2012, 6, 30, 23, 59, 59, 1_000_000_000)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 183, hour_div_12: 0, hour_mod_12: 0,
                          minute: 0, second: 0, timestamp: 1_341_100_800),
            ymdhms(2012, 7, 1, 0, 0, 0)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 183, hour_div_12: 0, hour_mod_12: 0,
                          minute: 0, second: 1, timestamp: 1_341_100_800),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(year: 2012, ordinal: 182, hour_div_12: 1, hour_mod_12: 11,
                          minute: 59, second: 60, timestamp: 1_341_100_801),
            Err(IMPOSSIBLE)
        );

        // error codes
        assert_eq!(
            parse!(year: 2015, month: 1, day: 20, weekday: Tue,
                          hour_div_12: 2, hour_mod_12: 1, minute: 35, second: 20),
            Err(OUT_OF_RANGE)
        ); // `hour_div_12` is out of range
    }

    #[test]
    fn test_parsed_to_datetime() {
        macro_rules! parse {
            ($($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_datetime()
            )
        }

        let ymdhmsn = |y, m, d, h, n, s, nano, off| {
            Ok(FixedOffset::east_opt(off)
                .unwrap()
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(y, m, d)
                        .unwrap()
                        .and_hms_nano_opt(h, n, s, nano)
                        .unwrap(),
                )
                .unwrap())
        };

        assert_eq!(parse!(offset: 0), Err(NOT_ENOUGH));
        assert_eq!(
            parse!(year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678),
            Err(NOT_ENOUGH)
        );
        assert_eq!(
            parse!(year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 0),
            ymdhmsn(2014, 12, 31, 4, 26, 40, 12_345_678, 0)
        );
        assert_eq!(
            parse!(year: 2014, ordinal: 365, hour_div_12: 1, hour_mod_12: 1,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 32400),
            ymdhmsn(2014, 12, 31, 13, 26, 40, 12_345_678, 32400)
        );
        assert_eq!(
            parse!(year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 1,
                          minute: 42, second: 4, nanosecond: 12_345_678, offset: -9876),
            ymdhmsn(2014, 12, 31, 1, 42, 4, 12_345_678, -9876)
        );
        assert_eq!(
            parse!(year: 2015, ordinal: 1, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 86_400),
            Err(OUT_OF_RANGE)
        ); // `FixedOffset` does not support such huge offset
    }

    #[test]
    fn test_parsed_to_datetime_with_timezone() {
        macro_rules! parse {
            ($tz:expr; $($k:ident: $v:expr),*) => (
                Parsed { $($k: Some($v),)* ..Parsed::new() }.to_datetime_with_timezone(&$tz)
            )
        }

        // single result from ymdhms
        assert_eq!(
            parse!(Utc;
                          year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 0),
            Ok(Utc
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(2014, 12, 31)
                        .unwrap()
                        .and_hms_nano_opt(4, 26, 40, 12_345_678)
                        .unwrap()
                )
                .unwrap())
        );
        assert_eq!(
            parse!(Utc;
                          year: 2014, ordinal: 365, hour_div_12: 1, hour_mod_12: 1,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 32400),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(FixedOffset::east_opt(32400).unwrap();
                          year: 2014, ordinal: 365, hour_div_12: 0, hour_mod_12: 4,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 0),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(FixedOffset::east_opt(32400).unwrap();
                          year: 2014, ordinal: 365, hour_div_12: 1, hour_mod_12: 1,
                          minute: 26, second: 40, nanosecond: 12_345_678, offset: 32400),
            Ok(FixedOffset::east_opt(32400)
                .unwrap()
                .from_local_datetime(
                    &NaiveDate::from_ymd_opt(2014, 12, 31)
                        .unwrap()
                        .and_hms_nano_opt(13, 26, 40, 12_345_678)
                        .unwrap()
                )
                .unwrap())
        );

        // single result from timestamp
        assert_eq!(
            parse!(Utc; timestamp: 1_420_000_000, offset: 0),
            Ok(Utc.with_ymd_and_hms(2014, 12, 31, 4, 26, 40).unwrap())
        );
        assert_eq!(
            parse!(Utc; timestamp: 1_420_000_000, offset: 32400),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(FixedOffset::east_opt(32400).unwrap(); timestamp: 1_420_000_000, offset: 0),
            Err(IMPOSSIBLE)
        );
        assert_eq!(
            parse!(FixedOffset::east_opt(32400).unwrap(); timestamp: 1_420_000_000, offset: 32400),
            Ok(FixedOffset::east_opt(32400)
                .unwrap()
                .with_ymd_and_hms(2014, 12, 31, 13, 26, 40)
                .unwrap())
        );

        // TODO test with a variable time zone (for None and Ambiguous cases)
    }

    #[test]
    fn issue_551() {
        use crate::Weekday;
        let mut parsed = Parsed::new();

        parsed.year = Some(2002);
        parsed.week_from_mon = Some(22);
        parsed.weekday = Some(Weekday::Mon);
        assert_eq!(
            NaiveDate::from_ymd_opt(2002, 6, 3).unwrap(),
            parsed.to_naive_date().unwrap()
        );

        parsed.year = Some(2001);
        assert_eq!(
            NaiveDate::from_ymd_opt(2001, 5, 28).unwrap(),
            parsed.to_naive_date().unwrap()
        );
    }
