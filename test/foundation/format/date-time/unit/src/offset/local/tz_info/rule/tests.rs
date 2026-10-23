
    use super::super::timezone::Transition;
    use super::super::{Error, TimeZone};
    use super::{AlternateTime, LocalTimeType, RuleDay, TransitionRule};

    #[test]
    fn test_quoted() -> Result<(), Error> {
        let transition_rule = TransitionRule::from_tz_string(b"<-03>+3<+03>-3,J1,J365", false)?;
        assert_eq!(
            transition_rule,
            AlternateTime::new(
                LocalTimeType::new(-10800, false, Some(b"-03"))?,
                LocalTimeType::new(10800, true, Some(b"+03"))?,
                RuleDay::julian_1(1)?,
                7200,
                RuleDay::julian_1(365)?,
                7200,
            )?
            .into()
        );
        Ok(())
    }

    #[test]
    fn test_full() -> Result<(), Error> {
        let tz_string = b"NZST-12:00:00NZDT-13:00:00,M10.1.0/02:00:00,M3.3.0/02:00:00";
        let transition_rule = TransitionRule::from_tz_string(tz_string, false)?;
        assert_eq!(
            transition_rule,
            AlternateTime::new(
                LocalTimeType::new(43200, false, Some(b"NZST"))?,
                LocalTimeType::new(46800, true, Some(b"NZDT"))?,
                RuleDay::month_weekday(10, 1, 0)?,
                7200,
                RuleDay::month_weekday(3, 3, 0)?,
                7200,
            )?
            .into()
        );
        Ok(())
    }

    #[test]
    fn test_negative_dst() -> Result<(), Error> {
        let tz_string = b"IST-1GMT0,M10.5.0,M3.5.0/1";
        let transition_rule = TransitionRule::from_tz_string(tz_string, false)?;
        assert_eq!(
            transition_rule,
            AlternateTime::new(
                LocalTimeType::new(3600, false, Some(b"IST"))?,
                LocalTimeType::new(0, true, Some(b"GMT"))?,
                RuleDay::month_weekday(10, 5, 0)?,
                7200,
                RuleDay::month_weekday(3, 5, 0)?,
                3600,
            )?
            .into()
        );
        Ok(())
    }

    #[test]
    fn test_negative_hour() -> Result<(), Error> {
        let tz_string = b"<-03>3<-02>,M3.5.0/-2,M10.5.0/-1";
        assert!(TransitionRule::from_tz_string(tz_string, false).is_err());

        assert_eq!(
            TransitionRule::from_tz_string(tz_string, true)?,
            AlternateTime::new(
                LocalTimeType::new(-10800, false, Some(b"-03"))?,
                LocalTimeType::new(-7200, true, Some(b"-02"))?,
                RuleDay::month_weekday(3, 5, 0)?,
                -7200,
                RuleDay::month_weekday(10, 5, 0)?,
                -3600,
            )?
            .into()
        );
        Ok(())
    }

    #[test]
    fn test_invalid_offset_hour() {
        // An offset of 24 hours does not fit in a `FixedOffset` and must be rejected.
        assert!(TransitionRule::from_tz_string(b"FOO24", false).is_err());
        assert!(TransitionRule::from_tz_string(b"FOO+24", false).is_err());
        assert!(TransitionRule::from_tz_string(b"FOO-24", false).is_err());
        // The maximum valid offset is 23:59:59.
        assert!(TransitionRule::from_tz_string(b"FOO23:59:59", false).is_ok());
    }

    #[test]
    fn test_all_year_dst() -> Result<(), Error> {
        let tz_string = b"EST5EDT,0/0,J365/25";
        assert!(TransitionRule::from_tz_string(tz_string, false).is_err());

        assert_eq!(
            TransitionRule::from_tz_string(tz_string, true)?,
            AlternateTime::new(
                LocalTimeType::new(-18000, false, Some(b"EST"))?,
                LocalTimeType::new(-14400, true, Some(b"EDT"))?,
                RuleDay::julian_0(0)?,
                0,
                RuleDay::julian_1(365)?,
                90000,
            )?
            .into()
        );
        Ok(())
    }

    #[test]
    fn test_v3_file() -> Result<(), Error> {
        let bytes = b"TZif3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\0\0\0\x04\0\0\x1c\x20\0\0IST\0TZif3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\0\0\0\x01\0\0\0\0\0\0\0\x01\0\0\0\x01\0\0\0\x04\0\0\0\0\x7f\xe8\x17\x80\0\0\0\x1c\x20\0\0IST\0\x01\x01\x0aIST-2IDT,M3.4.4/26,M10.5.0\x0a";

        let time_zone = TimeZone::from_tz_data(bytes)?;

        let time_zone_result = TimeZone::new(
            vec![Transition::new(2145916800, 0)],
            vec![LocalTimeType::new(7200, false, Some(b"IST"))?],
            Vec::new(),
            Some(TransitionRule::from(AlternateTime::new(
                LocalTimeType::new(7200, false, Some(b"IST"))?,
                LocalTimeType::new(10800, true, Some(b"IDT"))?,
                RuleDay::month_weekday(3, 4, 4)?,
                93600,
                RuleDay::month_weekday(10, 5, 0)?,
                7200,
            )?)),
        )?;

        assert_eq!(time_zone, time_zone_result);

        Ok(())
    }

    #[test]
    fn test_rule_day() -> Result<(), Error> {
        let rule_day_j1 = RuleDay::julian_1(60)?;
        assert_eq!(rule_day_j1.transition_date(2000), (3, 1));
        assert_eq!(rule_day_j1.transition_date(2001), (3, 1));
        assert_eq!(rule_day_j1.unix_time(2000, 43200), 951912000);

        let rule_day_j0 = RuleDay::julian_0(59)?;
        assert_eq!(rule_day_j0.transition_date(2000), (2, 29));
        assert_eq!(rule_day_j0.transition_date(2001), (3, 1));
        assert_eq!(rule_day_j0.unix_time(2000, 43200), 951825600);

        let rule_day_mwd = RuleDay::month_weekday(2, 5, 2)?;
        assert_eq!(rule_day_mwd.transition_date(2000), (2, 29));
        assert_eq!(rule_day_mwd.transition_date(2001), (2, 27));
        assert_eq!(rule_day_mwd.unix_time(2000, 43200), 951825600);
        assert_eq!(rule_day_mwd.unix_time(2001, 43200), 983275200);

        Ok(())
    }

    #[test]
    fn test_transition_rule() -> Result<(), Error> {
        let transition_rule_fixed = TransitionRule::from(LocalTimeType::new(-36000, false, None)?);
        assert_eq!(
            transition_rule_fixed.find_local_time_type(0)?.offset(),
            -36000
        );

        let transition_rule_dst = TransitionRule::from(AlternateTime::new(
            LocalTimeType::new(43200, false, Some(b"NZST"))?,
            LocalTimeType::new(46800, true, Some(b"NZDT"))?,
            RuleDay::month_weekday(10, 1, 0)?,
            7200,
            RuleDay::month_weekday(3, 3, 0)?,
            7200,
        )?);

        assert_eq!(
            transition_rule_dst
                .find_local_time_type(953384399)?
                .offset(),
            46800
        );
        assert_eq!(
            transition_rule_dst
                .find_local_time_type(953384400)?
                .offset(),
            43200
        );
        assert_eq!(
            transition_rule_dst
                .find_local_time_type(970322399)?
                .offset(),
            43200
        );
        assert_eq!(
            transition_rule_dst
                .find_local_time_type(970322400)?
                .offset(),
            46800
        );

        let transition_rule_negative_dst = TransitionRule::from(AlternateTime::new(
            LocalTimeType::new(3600, false, Some(b"IST"))?,
            LocalTimeType::new(0, true, Some(b"GMT"))?,
            RuleDay::month_weekday(10, 5, 0)?,
            7200,
            RuleDay::month_weekday(3, 5, 0)?,
            3600,
        )?);

        assert_eq!(
            transition_rule_negative_dst
                .find_local_time_type(954032399)?
                .offset(),
            0
        );
        assert_eq!(
            transition_rule_negative_dst
                .find_local_time_type(954032400)?
                .offset(),
            3600
        );
        assert_eq!(
            transition_rule_negative_dst
                .find_local_time_type(972781199)?
                .offset(),
            3600
        );
        assert_eq!(
            transition_rule_negative_dst
                .find_local_time_type(972781200)?
                .offset(),
            0
        );

        let transition_rule_negative_time_1 = TransitionRule::from(AlternateTime::new(
            LocalTimeType::new(0, false, None)?,
            LocalTimeType::new(0, true, None)?,
            RuleDay::julian_0(100)?,
            0,
            RuleDay::julian_0(101)?,
            -86500,
        )?);

        assert!(transition_rule_negative_time_1
            .find_local_time_type(8639899)?
            .is_dst());
        assert!(!transition_rule_negative_time_1
            .find_local_time_type(8639900)?
            .is_dst());
        assert!(!transition_rule_negative_time_1
            .find_local_time_type(8639999)?
            .is_dst());
        assert!(transition_rule_negative_time_1
            .find_local_time_type(8640000)?
            .is_dst());

        let transition_rule_negative_time_2 = TransitionRule::from(AlternateTime::new(
            LocalTimeType::new(-10800, false, Some(b"-03"))?,
            LocalTimeType::new(-7200, true, Some(b"-02"))?,
            RuleDay::month_weekday(3, 5, 0)?,
            -7200,
            RuleDay::month_weekday(10, 5, 0)?,
            -3600,
        )?);

        assert_eq!(
            transition_rule_negative_time_2
                .find_local_time_type(954032399)?
                .offset(),
            -10800
        );
        assert_eq!(
            transition_rule_negative_time_2
                .find_local_time_type(954032400)?
                .offset(),
            -7200
        );
        assert_eq!(
            transition_rule_negative_time_2
                .find_local_time_type(972781199)?
                .offset(),
            -7200
        );
        assert_eq!(
            transition_rule_negative_time_2
                .find_local_time_type(972781200)?
                .offset(),
            -10800
        );

        let transition_rule_all_year_dst = TransitionRule::from(AlternateTime::new(
            LocalTimeType::new(-18000, false, Some(b"EST"))?,
            LocalTimeType::new(-14400, true, Some(b"EDT"))?,
            RuleDay::julian_0(0)?,
            0,
            RuleDay::julian_1(365)?,
            90000,
        )?);

        assert_eq!(
            transition_rule_all_year_dst
                .find_local_time_type(946702799)?
                .offset(),
            -14400
        );
        assert_eq!(
            transition_rule_all_year_dst
                .find_local_time_type(946702800)?
                .offset(),
            -14400
        );

        Ok(())
    }

    #[test]
    fn test_transition_rule_overflow() -> Result<(), Error> {
        let transition_rule_1 = TransitionRule::from(AlternateTime::new(
            LocalTimeType::new(-1, false, None)?,
            LocalTimeType::new(-1, true, None)?,
            RuleDay::julian_1(365)?,
            0,
            RuleDay::julian_1(1)?,
            0,
        )?);

        let transition_rule_2 = TransitionRule::from(AlternateTime::new(
            LocalTimeType::new(1, false, None)?,
            LocalTimeType::new(1, true, None)?,
            RuleDay::julian_1(365)?,
            0,
            RuleDay::julian_1(1)?,
            0,
        )?);

        let min_unix_time = -67768100567971200;
        let max_unix_time = 67767976233532799;

        assert!(matches!(
            transition_rule_1.find_local_time_type(min_unix_time),
            Err(Error::OutOfRange(_))
        ));
        assert!(matches!(
            transition_rule_2.find_local_time_type(max_unix_time),
            Err(Error::OutOfRange(_))
        ));

        Ok(())
    }
