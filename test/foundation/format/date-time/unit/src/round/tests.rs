
    use super::{DurationRound, RoundingError, SubsecRound, TimeDelta};
    use crate::offset::{FixedOffset, TimeZone, Utc};
    use crate::Timelike;
    use crate::{DateTime, NaiveDate};

    #[test]
    fn test_round_subsecs() {
        let pst = FixedOffset::east_opt(8 * 60 * 60).unwrap();
        let dt = pst
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2018, 1, 11)
                    .unwrap()
                    .and_hms_nano_opt(10, 5, 13, 84_660_684)
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(dt.round_subsecs(10), dt);
        assert_eq!(dt.round_subsecs(9), dt);
        assert_eq!(dt.round_subsecs(8).nanosecond(), 84_660_680);
        assert_eq!(dt.round_subsecs(7).nanosecond(), 84_660_700);
        assert_eq!(dt.round_subsecs(6).nanosecond(), 84_661_000);
        assert_eq!(dt.round_subsecs(5).nanosecond(), 84_660_000);
        assert_eq!(dt.round_subsecs(4).nanosecond(), 84_700_000);
        assert_eq!(dt.round_subsecs(3).nanosecond(), 85_000_000);
        assert_eq!(dt.round_subsecs(2).nanosecond(), 80_000_000);
        assert_eq!(dt.round_subsecs(1).nanosecond(), 100_000_000);

        assert_eq!(dt.round_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.round_subsecs(0).second(), 13);

        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2018, 1, 11)
                    .unwrap()
                    .and_hms_nano_opt(10, 5, 27, 750_500_000)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(dt.round_subsecs(9), dt);
        assert_eq!(dt.round_subsecs(4), dt);
        assert_eq!(dt.round_subsecs(3).nanosecond(), 751_000_000);
        assert_eq!(dt.round_subsecs(2).nanosecond(), 750_000_000);
        assert_eq!(dt.round_subsecs(1).nanosecond(), 800_000_000);

        assert_eq!(dt.round_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.round_subsecs(0).second(), 28);
    }

    #[test]
    fn test_round_leap_nanos() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 1_750_500_000)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(dt.round_subsecs(9), dt);
        assert_eq!(dt.round_subsecs(4), dt);
        assert_eq!(dt.round_subsecs(2).nanosecond(), 1_750_000_000);
        assert_eq!(dt.round_subsecs(1).nanosecond(), 1_800_000_000);
        assert_eq!(dt.round_subsecs(1).second(), 59);

        assert_eq!(dt.round_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.round_subsecs(0).second(), 0);
    }

    #[test]
    fn test_trunc_subsecs() {
        let pst = FixedOffset::east_opt(8 * 60 * 60).unwrap();
        let dt = pst
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2018, 1, 11)
                    .unwrap()
                    .and_hms_nano_opt(10, 5, 13, 84_660_684)
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(dt.trunc_subsecs(10), dt);
        assert_eq!(dt.trunc_subsecs(9), dt);
        assert_eq!(dt.trunc_subsecs(8).nanosecond(), 84_660_680);
        assert_eq!(dt.trunc_subsecs(7).nanosecond(), 84_660_600);
        assert_eq!(dt.trunc_subsecs(6).nanosecond(), 84_660_000);
        assert_eq!(dt.trunc_subsecs(5).nanosecond(), 84_660_000);
        assert_eq!(dt.trunc_subsecs(4).nanosecond(), 84_600_000);
        assert_eq!(dt.trunc_subsecs(3).nanosecond(), 84_000_000);
        assert_eq!(dt.trunc_subsecs(2).nanosecond(), 80_000_000);
        assert_eq!(dt.trunc_subsecs(1).nanosecond(), 0);

        assert_eq!(dt.trunc_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.trunc_subsecs(0).second(), 13);

        let dt = pst
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2018, 1, 11)
                    .unwrap()
                    .and_hms_nano_opt(10, 5, 27, 750_500_000)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(dt.trunc_subsecs(9), dt);
        assert_eq!(dt.trunc_subsecs(4), dt);
        assert_eq!(dt.trunc_subsecs(3).nanosecond(), 750_000_000);
        assert_eq!(dt.trunc_subsecs(2).nanosecond(), 750_000_000);
        assert_eq!(dt.trunc_subsecs(1).nanosecond(), 700_000_000);

        assert_eq!(dt.trunc_subsecs(0).nanosecond(), 0);
        assert_eq!(dt.trunc_subsecs(0).second(), 27);
    }

    #[test]
    fn test_trunc_leap_nanos() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 1_750_500_000)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(dt.trunc_subsecs(9), dt);
        assert_eq!(dt.trunc_subsecs(4), dt);
        assert_eq!(dt.trunc_subsecs(2).nanosecond(), 1_750_000_000);
        assert_eq!(dt.trunc_subsecs(1).nanosecond(), 1_700_000_000);
        assert_eq!(dt.trunc_subsecs(1).second(), 59);

        assert_eq!(dt.trunc_subsecs(0).nanosecond(), 1_000_000_000);
        assert_eq!(dt.trunc_subsecs(0).second(), 59);
    }

    #[test]
    fn test_duration_round() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 175_500_000)
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(
            dt.duration_round(TimeDelta::new(-1, 0).unwrap()),
            Err(RoundingError::DurationExceedsLimit)
        );
        assert_eq!(
            dt.duration_round(TimeDelta::zero()),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_round(TimeDelta::try_milliseconds(10).unwrap())
                .unwrap()
                .to_string(),
            "2016-12-31 23:59:59.180 UTC"
        );

        // round up
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:25:00 UTC"
        );
        // round down
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 29, 999)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00 UTC"
        );

        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00 UTC"
        );
        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(30).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:30:00 UTC"
        );
        assert_eq!(
            dt.duration_round(TimeDelta::try_hours(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:00:00 UTC"
        );
        assert_eq!(
            dt.duration_round(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-13 00:00:00 UTC"
        );

        // timezone east
        let dt = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2020, 10, 27, 15, 0, 0)
            .unwrap();
        assert_eq!(
            dt.duration_round(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-28 00:00:00 +01:00"
        );
        assert_eq!(
            dt.duration_round(TimeDelta::try_weeks(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-29 00:00:00 +01:00"
        );

        // timezone west
        let dt = FixedOffset::west_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2020, 10, 27, 15, 0, 0)
            .unwrap();
        assert_eq!(
            dt.duration_round(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-28 00:00:00 -01:00"
        );
        assert_eq!(
            dt.duration_round(TimeDelta::try_weeks(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-29 00:00:00 -01:00"
        );
    }

    #[test]
    fn test_duration_round_naive() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 175_500_000)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();

        assert_eq!(
            dt.duration_round(TimeDelta::new(-1, 0).unwrap()),
            Err(RoundingError::DurationExceedsLimit)
        );
        assert_eq!(
            dt.duration_round(TimeDelta::zero()),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_round(TimeDelta::try_milliseconds(10).unwrap())
                .unwrap()
                .to_string(),
            "2016-12-31 23:59:59.180"
        );

        // round up
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:25:00"
        );
        // round down
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 29, 999)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00"
        );

        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00"
        );
        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(30).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:30:00"
        );
        assert_eq!(
            dt.duration_round(TimeDelta::try_hours(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:00:00"
        );
        assert_eq!(
            dt.duration_round(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-13 00:00:00"
        );
    }

    #[test]
    fn test_duration_round_pre_epoch() {
        let dt = Utc.with_ymd_and_hms(1969, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(
            dt.duration_round(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "1969-12-12 12:10:00 UTC"
        );
    }

    #[test]
    fn test_duration_trunc() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 175_500_000)
                    .unwrap(),
            )
            .unwrap();

        assert_eq!(
            dt.duration_trunc(TimeDelta::new(-1, 0).unwrap()),
            Err(RoundingError::DurationExceedsLimit)
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::zero()),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_trunc(TimeDelta::try_milliseconds(10).unwrap())
                .unwrap()
                .to_string(),
            "2016-12-31 23:59:59.170 UTC"
        );

        // would round up
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00 UTC"
        );
        // would round down
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 29, 999)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00 UTC"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00 UTC"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(30).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:00:00 UTC"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_hours(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:00:00 UTC"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 00:00:00 UTC"
        );

        // timezone east
        let dt = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2020, 10, 27, 15, 0, 0)
            .unwrap();
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-27 00:00:00 +01:00"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_weeks(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-22 00:00:00 +01:00"
        );

        // timezone west
        let dt = FixedOffset::west_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2020, 10, 27, 15, 0, 0)
            .unwrap();
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-27 00:00:00 -01:00"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_weeks(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-22 00:00:00 -01:00"
        );
    }

    #[test]
    fn test_duration_trunc_naive() {
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2016, 12, 31)
                    .unwrap()
                    .and_hms_nano_opt(23, 59, 59, 175_500_000)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();

        assert_eq!(
            dt.duration_trunc(TimeDelta::new(-1, 0).unwrap()),
            Err(RoundingError::DurationExceedsLimit)
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::zero()),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_trunc(TimeDelta::try_milliseconds(10).unwrap())
                .unwrap()
                .to_string(),
            "2016-12-31 23:59:59.170"
        );

        // would round up
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00"
        );
        // would round down
        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 29, 999)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:20:00"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(30).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:00:00"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_hours(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:00:00"
        );
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 00:00:00"
        );
    }

    #[test]
    fn test_duration_trunc_pre_epoch() {
        let dt = Utc.with_ymd_and_hms(1969, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(
            dt.duration_trunc(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "1969-12-12 12:10:00 UTC"
        );
    }

    #[test]
    fn issue1010() {
        let dt = DateTime::from_timestamp(-4_227_854_320, 678_774_288).unwrap();
        let span = TimeDelta::microseconds(-7_019_067_213_869_040);
        assert_eq!(
            dt.duration_trunc(span),
            Err(RoundingError::DurationExceedsLimit)
        );

        let dt = DateTime::from_timestamp(320_041_586, 920_103_021).unwrap();
        let span = TimeDelta::nanoseconds(-8_923_838_508_697_114_584);
        assert_eq!(
            dt.duration_round(span),
            Err(RoundingError::DurationExceedsLimit)
        );

        let dt = DateTime::from_timestamp(-2_621_440, 0).unwrap();
        let span = TimeDelta::nanoseconds(-9_223_372_036_854_771_421);
        assert_eq!(
            dt.duration_round(span),
            Err(RoundingError::DurationExceedsLimit)
        );
    }

    #[test]
    fn test_duration_trunc_close_to_epoch() {
        let span = TimeDelta::try_minutes(15).unwrap();

        let dt = NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 15)
            .unwrap();
        assert_eq!(
            dt.duration_trunc(span).unwrap().to_string(),
            "1970-01-01 00:00:00"
        );

        let dt = NaiveDate::from_ymd_opt(1969, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 45)
            .unwrap();
        assert_eq!(
            dt.duration_trunc(span).unwrap().to_string(),
            "1969-12-31 23:45:00"
        );
    }

    #[test]
    fn test_duration_round_close_to_epoch() {
        let span = TimeDelta::try_minutes(15).unwrap();

        let dt = NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 15)
            .unwrap();
        assert_eq!(
            dt.duration_round(span).unwrap().to_string(),
            "1970-01-01 00:00:00"
        );

        let dt = NaiveDate::from_ymd_opt(1969, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 45)
            .unwrap();
        assert_eq!(
            dt.duration_round(span).unwrap().to_string(),
            "1970-01-01 00:00:00"
        );
    }

    #[test]
    fn test_duration_round_close_to_min_max() {
        let span = TimeDelta::nanoseconds(i64::MAX);

        let dt = DateTime::from_timestamp_nanos(i64::MIN / 2 - 1);
        assert_eq!(
            dt.duration_round(span).unwrap().to_string(),
            "1677-09-21 00:12:43.145224193 UTC"
        );

        let dt = DateTime::from_timestamp_nanos(i64::MIN / 2 + 1);
        assert_eq!(
            dt.duration_round(span).unwrap().to_string(),
            "1970-01-01 00:00:00 UTC"
        );

        let dt = DateTime::from_timestamp_nanos(i64::MAX / 2 + 1);
        assert_eq!(
            dt.duration_round(span).unwrap().to_string(),
            "2262-04-11 23:47:16.854775807 UTC"
        );

        let dt = DateTime::from_timestamp_nanos(i64::MAX / 2 - 1);
        assert_eq!(
            dt.duration_round(span).unwrap().to_string(),
            "1970-01-01 00:00:00 UTC"
        );
    }

    #[test]
    fn test_duration_round_up() {
        let dt = NaiveDate::from_ymd_opt(2016, 12, 31)
            .unwrap()
            .and_hms_nano_opt(23, 59, 59, 175_500_000)
            .unwrap()
            .and_utc();

        assert_eq!(
            dt.duration_round_up(TimeDelta::new(-1, 0).unwrap()),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_round_up(TimeDelta::zero()),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_round_up(TimeDelta::MAX),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_round_up(TimeDelta::try_milliseconds(10).unwrap())
                .unwrap()
                .to_string(),
            "2016-12-31 23:59:59.180 UTC"
        );

        // round up
        let dt = NaiveDate::from_ymd_opt(2012, 12, 12)
            .unwrap()
            .and_hms_milli_opt(18, 22, 30, 0)
            .unwrap()
            .and_utc();

        assert_eq!(
            dt.duration_round_up(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:25:00 UTC"
        );

        assert_eq!(
            dt.duration_round_up(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:30:00 UTC"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_minutes(30).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:30:00 UTC"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_hours(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 19:00:00 UTC"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-13 00:00:00 UTC"
        );

        // timezone east
        let dt = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2020, 10, 27, 15, 0, 0)
            .unwrap();
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-28 00:00:00 +01:00"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_weeks(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-29 00:00:00 +01:00"
        );

        // timezone west
        let dt = FixedOffset::west_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2020, 10, 27, 15, 0, 0)
            .unwrap();
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-28 00:00:00 -01:00"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_weeks(1).unwrap())
                .unwrap()
                .to_string(),
            "2020-10-29 00:00:00 -01:00"
        );
    }

    #[test]
    fn test_duration_round_up_naive() {
        let dt = NaiveDate::from_ymd_opt(2016, 12, 31)
            .unwrap()
            .and_hms_nano_opt(23, 59, 59, 175_500_000)
            .unwrap();

        assert_eq!(
            dt.duration_round_up(TimeDelta::new(-1, 0).unwrap()),
            Err(RoundingError::DurationExceedsLimit)
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::zero()),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_round_up(TimeDelta::MAX),
            Err(RoundingError::DurationExceedsLimit)
        );

        assert_eq!(
            dt.duration_round_up(TimeDelta::try_milliseconds(10).unwrap())
                .unwrap()
                .to_string(),
            "2016-12-31 23:59:59.180"
        );

        let dt = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2012, 12, 12)
                    .unwrap()
                    .and_hms_milli_opt(18, 22, 30, 0)
                    .unwrap(),
            )
            .unwrap()
            .naive_utc();
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_minutes(5).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:25:00"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:30:00"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_minutes(30).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 18:30:00"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_hours(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-12 19:00:00"
        );
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_days(1).unwrap())
                .unwrap()
                .to_string(),
            "2012-12-13 00:00:00"
        );
    }

    #[test]
    fn test_duration_round_up_pre_epoch() {
        let dt = Utc.with_ymd_and_hms(1969, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(
            dt.duration_round_up(TimeDelta::try_minutes(10).unwrap())
                .unwrap()
                .to_string(),
            "1969-12-12 12:20:00 UTC"
        );

        let time_delta = TimeDelta::minutes(30);
        assert_eq!(
            DateTime::UNIX_EPOCH
                .duration_round_up(time_delta)
                .unwrap()
                .to_string(),
            "1970-01-01 00:00:00 UTC"
        )
    }

    #[test]
    fn test_duration_round_up_close_to_min_max() {
        let mut dt = NaiveDate::from_ymd_opt(2012, 12, 12)
            .unwrap()
            .and_hms_milli_opt(18, 22, 30, 0)
            .unwrap()
            .and_utc();

        let span = TimeDelta::nanoseconds(i64::MAX);

        assert_eq!(
            dt.duration_round_up(span).unwrap().to_string(),
            DateTime::from_timestamp_nanos(i64::MAX).to_string()
        );

        dt = DateTime::UNIX_EPOCH + TimeDelta::nanoseconds(1);
        assert_eq!(
            dt.duration_round_up(span).unwrap(),
            DateTime::from_timestamp_nanos(i64::MAX)
        );

        let dt = DateTime::from_timestamp_nanos(1);
        assert_eq!(
            dt.duration_round_up(span).unwrap().to_string(),
            "2262-04-11 23:47:16.854775807 UTC"
        );

        let dt = DateTime::from_timestamp_nanos(-1);
        assert_eq!(dt.duration_round_up(span).unwrap(), DateTime::UNIX_EPOCH);

        // Rounds to 1677-09-21 00:12:43.145224193 UTC if at i64::MIN.
        // because i64::MIN is 1677-09-21 00:12:43.145224192 UTC.
        //
        //                                                v
        // We add 2 to get to 1677-09-21 00:12:43.145224194 UTC
        // this issue is because abs(i64::MIN) == i64::MAX + 1
        let dt = DateTime::from_timestamp_nanos(i64::MIN + 2);
        assert_eq!(dt.duration_round_up(span).unwrap(), DateTime::UNIX_EPOCH);
    }
