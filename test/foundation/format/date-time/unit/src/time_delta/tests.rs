
    use super::OutOfRangeError;
    use super::{TimeDelta, MAX, MIN};
    use crate::expect;
    use core::time::Duration;

    #[test]
    fn test_duration() {
        let days = |d| TimeDelta::try_days(d).unwrap();
        let seconds = |s| TimeDelta::try_seconds(s).unwrap();

        assert!(seconds(1) != TimeDelta::zero());
        assert_eq!(seconds(1) + seconds(2), seconds(3));
        assert_eq!(seconds(86_399) + seconds(4), days(1) + seconds(3));
        assert_eq!(days(10) - seconds(1000), seconds(863_000));
        assert_eq!(days(10) - seconds(1_000_000), seconds(-136_000));
        assert_eq!(
            days(2) + seconds(86_399) + TimeDelta::nanoseconds(1_234_567_890),
            days(3) + TimeDelta::nanoseconds(234_567_890)
        );
        assert_eq!(-days(3), days(-3));
        assert_eq!(-(days(3) + seconds(70)), days(-4) + seconds(86_400 - 70));

        let mut d = TimeDelta::default();
        d += TimeDelta::try_minutes(1).unwrap();
        d -= seconds(30);
        assert_eq!(d, seconds(30));
    }

    #[test]
    fn test_duration_num_days() {
        assert_eq!(TimeDelta::zero().num_days(), 0);
        assert_eq!(TimeDelta::try_days(1).unwrap().num_days(), 1);
        assert_eq!(TimeDelta::try_days(-1).unwrap().num_days(), -1);
        assert_eq!(TimeDelta::try_seconds(86_399).unwrap().num_days(), 0);
        assert_eq!(TimeDelta::try_seconds(86_401).unwrap().num_days(), 1);
        assert_eq!(TimeDelta::try_seconds(-86_399).unwrap().num_days(), 0);
        assert_eq!(TimeDelta::try_seconds(-86_401).unwrap().num_days(), -1);
        assert_eq!(
            TimeDelta::try_days(i32::MAX as i64).unwrap().num_days(),
            i32::MAX as i64
        );
        assert_eq!(
            TimeDelta::try_days(i32::MIN as i64).unwrap().num_days(),
            i32::MIN as i64
        );
    }

    #[test]
    fn test_duration_num_seconds() {
        assert_eq!(TimeDelta::zero().num_seconds(), 0);
        assert_eq!(TimeDelta::try_seconds(1).unwrap().num_seconds(), 1);
        assert_eq!(TimeDelta::try_seconds(-1).unwrap().num_seconds(), -1);
        assert_eq!(TimeDelta::try_milliseconds(999).unwrap().num_seconds(), 0);
        assert_eq!(TimeDelta::try_milliseconds(1001).unwrap().num_seconds(), 1);
        assert_eq!(TimeDelta::try_milliseconds(-999).unwrap().num_seconds(), 0);
        assert_eq!(
            TimeDelta::try_milliseconds(-1001).unwrap().num_seconds(),
            -1
        );
    }

    #[test]
    fn test_duration_seconds_max_allowed() {
        let duration = TimeDelta::try_seconds(i64::MAX / 1_000).unwrap();
        assert_eq!(duration.num_seconds(), i64::MAX / 1_000);
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MAX as i128 / 1_000 * 1_000_000_000
        );
    }

    #[test]
    fn test_duration_seconds_max_overflow() {
        assert!(TimeDelta::try_seconds(i64::MAX / 1_000 + 1).is_none());
    }

    #[test]
    #[should_panic(expected = "TimeDelta::seconds out of bounds")]
    fn test_duration_seconds_max_overflow_panic() {
        let _ = TimeDelta::seconds(i64::MAX / 1_000 + 1);
    }

    #[test]
    fn test_duration_seconds_min_allowed() {
        let duration = TimeDelta::try_seconds(i64::MIN / 1_000).unwrap(); // Same as -i64::MAX / 1_000 due to rounding
        assert_eq!(duration.num_seconds(), i64::MIN / 1_000); // Same as -i64::MAX / 1_000 due to rounding
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            -i64::MAX as i128 / 1_000 * 1_000_000_000
        );
    }

    #[test]
    fn test_duration_seconds_min_underflow() {
        assert!(TimeDelta::try_seconds(-i64::MAX / 1_000 - 1).is_none());
    }

    #[test]
    #[should_panic(expected = "TimeDelta::seconds out of bounds")]
    fn test_duration_seconds_min_underflow_panic() {
        let _ = TimeDelta::seconds(-i64::MAX / 1_000 - 1);
    }

    #[test]
    fn test_duration_as_seconds_f64() {
        assert_eq!(TimeDelta::seconds(1).as_seconds_f64(), 1.0);
        assert_eq!(TimeDelta::seconds(-1).as_seconds_f64(), -1.0);
        assert_eq!(TimeDelta::seconds(100).as_seconds_f64(), 100.0);
        assert_eq!(TimeDelta::seconds(-100).as_seconds_f64(), -100.0);

        assert_eq!(TimeDelta::milliseconds(500).as_seconds_f64(), 0.5);
        assert_eq!(TimeDelta::milliseconds(-500).as_seconds_f64(), -0.5);
        assert_eq!(TimeDelta::milliseconds(1_500).as_seconds_f64(), 1.5);
        assert_eq!(TimeDelta::milliseconds(-1_500).as_seconds_f64(), -1.5);
    }

    #[test]
    fn test_duration_as_seconds_f32() {
        assert_eq!(TimeDelta::seconds(1).as_seconds_f32(), 1.0);
        assert_eq!(TimeDelta::seconds(-1).as_seconds_f32(), -1.0);
        assert_eq!(TimeDelta::seconds(100).as_seconds_f32(), 100.0);
        assert_eq!(TimeDelta::seconds(-100).as_seconds_f32(), -100.0);

        assert_eq!(TimeDelta::milliseconds(500).as_seconds_f32(), 0.5);
        assert_eq!(TimeDelta::milliseconds(-500).as_seconds_f32(), -0.5);
        assert_eq!(TimeDelta::milliseconds(1_500).as_seconds_f32(), 1.5);
        assert_eq!(TimeDelta::milliseconds(-1_500).as_seconds_f32(), -1.5);
    }

    #[test]
    fn test_duration_subsec_nanos() {
        assert_eq!(TimeDelta::zero().subsec_nanos(), 0);
        assert_eq!(TimeDelta::nanoseconds(1).subsec_nanos(), 1);
        assert_eq!(TimeDelta::nanoseconds(-1).subsec_nanos(), -1);
        assert_eq!(TimeDelta::seconds(1).subsec_nanos(), 0);
        assert_eq!(TimeDelta::nanoseconds(1_000_000_001).subsec_nanos(), 1);
    }

    #[test]
    fn test_duration_subsec_micros() {
        assert_eq!(TimeDelta::zero().subsec_micros(), 0);
        assert_eq!(TimeDelta::microseconds(1).subsec_micros(), 1);
        assert_eq!(TimeDelta::microseconds(-1).subsec_micros(), -1);
        assert_eq!(TimeDelta::seconds(1).subsec_micros(), 0);
        assert_eq!(TimeDelta::microseconds(1_000_001).subsec_micros(), 1);
        assert_eq!(TimeDelta::nanoseconds(1_000_001_999).subsec_micros(), 1);
    }

    #[test]
    fn test_duration_subsec_millis() {
        assert_eq!(TimeDelta::zero().subsec_millis(), 0);
        assert_eq!(TimeDelta::milliseconds(1).subsec_millis(), 1);
        assert_eq!(TimeDelta::milliseconds(-1).subsec_millis(), -1);
        assert_eq!(TimeDelta::seconds(1).subsec_millis(), 0);
        assert_eq!(TimeDelta::milliseconds(1_001).subsec_millis(), 1);
        assert_eq!(TimeDelta::microseconds(1_001_999).subsec_millis(), 1);
    }

    #[test]
    fn test_duration_num_milliseconds() {
        assert_eq!(TimeDelta::zero().num_milliseconds(), 0);
        assert_eq!(
            TimeDelta::try_milliseconds(1).unwrap().num_milliseconds(),
            1
        );
        assert_eq!(
            TimeDelta::try_milliseconds(-1).unwrap().num_milliseconds(),
            -1
        );
        assert_eq!(TimeDelta::microseconds(999).num_milliseconds(), 0);
        assert_eq!(TimeDelta::microseconds(1001).num_milliseconds(), 1);
        assert_eq!(TimeDelta::microseconds(-999).num_milliseconds(), 0);
        assert_eq!(TimeDelta::microseconds(-1001).num_milliseconds(), -1);
    }

    #[test]
    fn test_duration_milliseconds_max_allowed() {
        // The maximum number of milliseconds acceptable through the constructor is
        // equal to the number that can be stored in a TimeDelta.
        let duration = TimeDelta::try_milliseconds(i64::MAX).unwrap();
        assert_eq!(duration.num_milliseconds(), i64::MAX);
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MAX as i128 * 1_000_000
        );
    }

    #[test]
    fn test_duration_milliseconds_max_overflow() {
        // Here we ensure that trying to add one millisecond to the maximum storable
        // value will fail.
        assert!(TimeDelta::try_milliseconds(i64::MAX)
            .unwrap()
            .checked_add(&TimeDelta::try_milliseconds(1).unwrap())
            .is_none());
    }

    #[test]
    fn test_duration_milliseconds_min_allowed() {
        // The minimum number of milliseconds acceptable through the constructor is
        // not equal to the number that can be stored in a TimeDelta - there is a
        // difference of one (i64::MIN vs -i64::MAX).
        let duration = TimeDelta::try_milliseconds(-i64::MAX).unwrap();
        assert_eq!(duration.num_milliseconds(), -i64::MAX);
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            -i64::MAX as i128 * 1_000_000
        );
    }

    #[test]
    fn test_duration_milliseconds_min_underflow() {
        // Here we ensure that trying to subtract one millisecond from the minimum
        // storable value will fail.
        assert!(TimeDelta::try_milliseconds(-i64::MAX)
            .unwrap()
            .checked_sub(&TimeDelta::try_milliseconds(1).unwrap())
            .is_none());
    }

    #[test]
    #[should_panic(expected = "TimeDelta::milliseconds out of bounds")]
    fn test_duration_milliseconds_min_underflow_panic() {
        // Here we ensure that trying to create a value one millisecond below the
        // minimum storable value will fail. This test is necessary because the
        // storable range is -i64::MAX, but the constructor type of i64 will allow
        // i64::MIN, which is one value below.
        let _ = TimeDelta::milliseconds(i64::MIN); // Same as -i64::MAX - 1
    }

    #[test]
    fn test_duration_num_microseconds() {
        assert_eq!(TimeDelta::zero().num_microseconds(), Some(0));
        assert_eq!(TimeDelta::microseconds(1).num_microseconds(), Some(1));
        assert_eq!(TimeDelta::microseconds(-1).num_microseconds(), Some(-1));
        assert_eq!(TimeDelta::nanoseconds(999).num_microseconds(), Some(0));
        assert_eq!(TimeDelta::nanoseconds(1001).num_microseconds(), Some(1));
        assert_eq!(TimeDelta::nanoseconds(-999).num_microseconds(), Some(0));
        assert_eq!(TimeDelta::nanoseconds(-1001).num_microseconds(), Some(-1));

        // overflow checks
        const MICROS_PER_DAY: i64 = 86_400_000_000;
        assert_eq!(
            TimeDelta::try_days(i64::MAX / MICROS_PER_DAY)
                .unwrap()
                .num_microseconds(),
            Some(i64::MAX / MICROS_PER_DAY * MICROS_PER_DAY)
        );
        assert_eq!(
            TimeDelta::try_days(-i64::MAX / MICROS_PER_DAY)
                .unwrap()
                .num_microseconds(),
            Some(-i64::MAX / MICROS_PER_DAY * MICROS_PER_DAY)
        );
        assert_eq!(
            TimeDelta::try_days(i64::MAX / MICROS_PER_DAY + 1)
                .unwrap()
                .num_microseconds(),
            None
        );
        assert_eq!(
            TimeDelta::try_days(-i64::MAX / MICROS_PER_DAY - 1)
                .unwrap()
                .num_microseconds(),
            None
        );
    }
    #[test]
    fn test_duration_microseconds_max_allowed() {
        // The number of microseconds acceptable through the constructor is far
        // fewer than the number that can actually be stored in a TimeDelta, so this
        // is not a particular insightful test.
        let duration = TimeDelta::microseconds(i64::MAX);
        assert_eq!(duration.num_microseconds(), Some(i64::MAX));
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MAX as i128 * 1_000
        );
        // Here we create a TimeDelta with the maximum possible number of
        // microseconds by creating a TimeDelta with the maximum number of
        // milliseconds and then checking that the number of microseconds matches
        // the storage limit.
        let duration = TimeDelta::try_milliseconds(i64::MAX).unwrap();
        assert!(duration.num_microseconds().is_none());
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MAX as i128 * 1_000_000
        );
    }
    #[test]
    fn test_duration_microseconds_max_overflow() {
        // This test establishes that a TimeDelta can store more microseconds than
        // are representable through the return of duration.num_microseconds().
        let duration = TimeDelta::microseconds(i64::MAX) + TimeDelta::microseconds(1);
        assert!(duration.num_microseconds().is_none());
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            (i64::MAX as i128 + 1) * 1_000
        );
        // Here we ensure that trying to add one microsecond to the maximum storable
        // value will fail.
        assert!(TimeDelta::try_milliseconds(i64::MAX)
            .unwrap()
            .checked_add(&TimeDelta::microseconds(1))
            .is_none());
    }
    #[test]
    fn test_duration_microseconds_min_allowed() {
        // The number of microseconds acceptable through the constructor is far
        // fewer than the number that can actually be stored in a TimeDelta, so this
        // is not a particular insightful test.
        let duration = TimeDelta::microseconds(i64::MIN);
        assert_eq!(duration.num_microseconds(), Some(i64::MIN));
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MIN as i128 * 1_000
        );
        // Here we create a TimeDelta with the minimum possible number of
        // microseconds by creating a TimeDelta with the minimum number of
        // milliseconds and then checking that the number of microseconds matches
        // the storage limit.
        let duration = TimeDelta::try_milliseconds(-i64::MAX).unwrap();
        assert!(duration.num_microseconds().is_none());
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            -i64::MAX as i128 * 1_000_000
        );
    }
    #[test]
    fn test_duration_microseconds_min_underflow() {
        // This test establishes that a TimeDelta can store more microseconds than
        // are representable through the return of duration.num_microseconds().
        let duration = TimeDelta::microseconds(i64::MIN) - TimeDelta::microseconds(1);
        assert!(duration.num_microseconds().is_none());
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            (i64::MIN as i128 - 1) * 1_000
        );
        // Here we ensure that trying to subtract one microsecond from the minimum
        // storable value will fail.
        assert!(TimeDelta::try_milliseconds(-i64::MAX)
            .unwrap()
            .checked_sub(&TimeDelta::microseconds(1))
            .is_none());
    }

    #[test]
    fn test_duration_num_nanoseconds() {
        assert_eq!(TimeDelta::zero().num_nanoseconds(), Some(0));
        assert_eq!(TimeDelta::nanoseconds(1).num_nanoseconds(), Some(1));
        assert_eq!(TimeDelta::nanoseconds(-1).num_nanoseconds(), Some(-1));

        // overflow checks
        const NANOS_PER_DAY: i64 = 86_400_000_000_000;
        assert_eq!(
            TimeDelta::try_days(i64::MAX / NANOS_PER_DAY)
                .unwrap()
                .num_nanoseconds(),
            Some(i64::MAX / NANOS_PER_DAY * NANOS_PER_DAY)
        );
        assert_eq!(
            TimeDelta::try_days(-i64::MAX / NANOS_PER_DAY)
                .unwrap()
                .num_nanoseconds(),
            Some(-i64::MAX / NANOS_PER_DAY * NANOS_PER_DAY)
        );
        assert_eq!(
            TimeDelta::try_days(i64::MAX / NANOS_PER_DAY + 1)
                .unwrap()
                .num_nanoseconds(),
            None
        );
        assert_eq!(
            TimeDelta::try_days(-i64::MAX / NANOS_PER_DAY - 1)
                .unwrap()
                .num_nanoseconds(),
            None
        );
    }
    #[test]
    fn test_duration_nanoseconds_max_allowed() {
        // The number of nanoseconds acceptable through the constructor is far fewer
        // than the number that can actually be stored in a TimeDelta, so this is not
        // a particular insightful test.
        let duration = TimeDelta::nanoseconds(i64::MAX);
        assert_eq!(duration.num_nanoseconds(), Some(i64::MAX));
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MAX as i128
        );
        // Here we create a TimeDelta with the maximum possible number of nanoseconds
        // by creating a TimeDelta with the maximum number of milliseconds and then
        // checking that the number of nanoseconds matches the storage limit.
        let duration = TimeDelta::try_milliseconds(i64::MAX).unwrap();
        assert!(duration.num_nanoseconds().is_none());
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MAX as i128 * 1_000_000
        );
    }

    #[test]
    fn test_duration_nanoseconds_max_overflow() {
        // This test establishes that a TimeDelta can store more nanoseconds than are
        // representable through the return of duration.num_nanoseconds().
        let duration = TimeDelta::nanoseconds(i64::MAX) + TimeDelta::nanoseconds(1);
        assert!(duration.num_nanoseconds().is_none());
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MAX as i128 + 1
        );
        // Here we ensure that trying to add one nanosecond to the maximum storable
        // value will fail.
        assert!(TimeDelta::try_milliseconds(i64::MAX)
            .unwrap()
            .checked_add(&TimeDelta::nanoseconds(1))
            .is_none());
    }

    #[test]
    fn test_duration_nanoseconds_min_allowed() {
        // The number of nanoseconds acceptable through the constructor is far fewer
        // than the number that can actually be stored in a TimeDelta, so this is not
        // a particular insightful test.
        let duration = TimeDelta::nanoseconds(i64::MIN);
        assert_eq!(duration.num_nanoseconds(), Some(i64::MIN));
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MIN as i128
        );
        // Here we create a TimeDelta with the minimum possible number of nanoseconds
        // by creating a TimeDelta with the minimum number of milliseconds and then
        // checking that the number of nanoseconds matches the storage limit.
        let duration = TimeDelta::try_milliseconds(-i64::MAX).unwrap();
        assert!(duration.num_nanoseconds().is_none());
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            -i64::MAX as i128 * 1_000_000
        );
    }

    #[test]
    fn test_duration_nanoseconds_min_underflow() {
        // This test establishes that a TimeDelta can store more nanoseconds than are
        // representable through the return of duration.num_nanoseconds().
        let duration = TimeDelta::nanoseconds(i64::MIN) - TimeDelta::nanoseconds(1);
        assert!(duration.num_nanoseconds().is_none());
        assert_eq!(
            duration.secs as i128 * 1_000_000_000 + duration.nanos as i128,
            i64::MIN as i128 - 1
        );
        // Here we ensure that trying to subtract one nanosecond from the minimum
        // storable value will fail.
        assert!(TimeDelta::try_milliseconds(-i64::MAX)
            .unwrap()
            .checked_sub(&TimeDelta::nanoseconds(1))
            .is_none());
    }

    #[test]
    fn test_max() {
        assert_eq!(
            MAX.secs as i128 * 1_000_000_000 + MAX.nanos as i128,
            i64::MAX as i128 * 1_000_000
        );
        assert_eq!(MAX, TimeDelta::try_milliseconds(i64::MAX).unwrap());
        assert_eq!(MAX.num_milliseconds(), i64::MAX);
        assert_eq!(MAX.num_microseconds(), None);
        assert_eq!(MAX.num_nanoseconds(), None);
    }

    #[test]
    fn test_min() {
        assert_eq!(
            MIN.secs as i128 * 1_000_000_000 + MIN.nanos as i128,
            -i64::MAX as i128 * 1_000_000
        );
        assert_eq!(MIN, TimeDelta::try_milliseconds(-i64::MAX).unwrap());
        assert_eq!(MIN.num_milliseconds(), -i64::MAX);
        assert_eq!(MIN.num_microseconds(), None);
        assert_eq!(MIN.num_nanoseconds(), None);
    }

    #[test]
    fn test_duration_ord() {
        let milliseconds = |ms| TimeDelta::try_milliseconds(ms).unwrap();

        assert!(milliseconds(1) < milliseconds(2));
        assert!(milliseconds(2) > milliseconds(1));
        assert!(milliseconds(-1) > milliseconds(-2));
        assert!(milliseconds(-2) < milliseconds(-1));
        assert!(milliseconds(-1) < milliseconds(1));
        assert!(milliseconds(1) > milliseconds(-1));
        assert!(milliseconds(0) < milliseconds(1));
        assert!(milliseconds(0) > milliseconds(-1));
        assert!(milliseconds(1_001) < milliseconds(1_002));
        assert!(milliseconds(-1_001) > milliseconds(-1_002));
        assert!(TimeDelta::nanoseconds(1_234_567_890) < TimeDelta::nanoseconds(1_234_567_891));
        assert!(TimeDelta::nanoseconds(-1_234_567_890) > TimeDelta::nanoseconds(-1_234_567_891));
        assert!(milliseconds(i64::MAX) > milliseconds(i64::MAX - 1));
        assert!(milliseconds(-i64::MAX) < milliseconds(-i64::MAX + 1));
    }

    #[test]
    fn test_duration_checked_ops() {
        let milliseconds = |ms| TimeDelta::try_milliseconds(ms).unwrap();
        let seconds = |s| TimeDelta::try_seconds(s).unwrap();

        assert_eq!(
            milliseconds(i64::MAX).checked_add(&milliseconds(0)),
            Some(milliseconds(i64::MAX))
        );
        assert_eq!(
            milliseconds(i64::MAX - 1).checked_add(&TimeDelta::microseconds(999)),
            Some(milliseconds(i64::MAX - 2) + TimeDelta::microseconds(1999))
        );
        assert!(milliseconds(i64::MAX)
            .checked_add(&TimeDelta::microseconds(1000))
            .is_none());
        assert!(milliseconds(i64::MAX)
            .checked_add(&TimeDelta::nanoseconds(1))
            .is_none());

        assert_eq!(
            milliseconds(-i64::MAX).checked_sub(&milliseconds(0)),
            Some(milliseconds(-i64::MAX))
        );
        assert_eq!(
            milliseconds(-i64::MAX + 1).checked_sub(&TimeDelta::microseconds(999)),
            Some(milliseconds(-i64::MAX + 2) - TimeDelta::microseconds(1999))
        );
        assert!(milliseconds(-i64::MAX)
            .checked_sub(&milliseconds(1))
            .is_none());
        assert!(milliseconds(-i64::MAX)
            .checked_sub(&TimeDelta::nanoseconds(1))
            .is_none());

        assert!(seconds(i64::MAX / 1000).checked_mul(2000).is_none());
        assert!(seconds(i64::MIN / 1000).checked_mul(2000).is_none());
        assert!(seconds(1).checked_div(0).is_none());
    }

    #[test]
    fn test_duration_abs() {
        let milliseconds = |ms| TimeDelta::try_milliseconds(ms).unwrap();

        assert_eq!(milliseconds(1300).abs(), milliseconds(1300));
        assert_eq!(milliseconds(1000).abs(), milliseconds(1000));
        assert_eq!(milliseconds(300).abs(), milliseconds(300));
        assert_eq!(milliseconds(0).abs(), milliseconds(0));
        assert_eq!(milliseconds(-300).abs(), milliseconds(300));
        assert_eq!(milliseconds(-700).abs(), milliseconds(700));
        assert_eq!(milliseconds(-1000).abs(), milliseconds(1000));
        assert_eq!(milliseconds(-1300).abs(), milliseconds(1300));
        assert_eq!(milliseconds(-1700).abs(), milliseconds(1700));
        assert_eq!(milliseconds(-i64::MAX).abs(), milliseconds(i64::MAX));
    }

    #[test]
    #[allow(clippy::erasing_op)]
    fn test_duration_mul() {
        assert_eq!(TimeDelta::zero() * i32::MAX, TimeDelta::zero());
        assert_eq!(TimeDelta::zero() * i32::MIN, TimeDelta::zero());
        assert_eq!(TimeDelta::nanoseconds(1) * 0, TimeDelta::zero());
        assert_eq!(TimeDelta::nanoseconds(1) * 1, TimeDelta::nanoseconds(1));
        assert_eq!(
            TimeDelta::nanoseconds(1) * 1_000_000_000,
            TimeDelta::try_seconds(1).unwrap()
        );
        assert_eq!(
            TimeDelta::nanoseconds(1) * -1_000_000_000,
            -TimeDelta::try_seconds(1).unwrap()
        );
        assert_eq!(
            -TimeDelta::nanoseconds(1) * 1_000_000_000,
            -TimeDelta::try_seconds(1).unwrap()
        );
        assert_eq!(
            TimeDelta::nanoseconds(30) * 333_333_333,
            TimeDelta::try_seconds(10).unwrap() - TimeDelta::nanoseconds(10)
        );
        assert_eq!(
            (TimeDelta::nanoseconds(1)
                + TimeDelta::try_seconds(1).unwrap()
                + TimeDelta::try_days(1).unwrap())
                * 3,
            TimeDelta::nanoseconds(3)
                + TimeDelta::try_seconds(3).unwrap()
                + TimeDelta::try_days(3).unwrap()
        );
        assert_eq!(
            TimeDelta::try_milliseconds(1500).unwrap() * -2,
            TimeDelta::try_seconds(-3).unwrap()
        );
        assert_eq!(
            TimeDelta::try_milliseconds(-1500).unwrap() * 2,
            TimeDelta::try_seconds(-3).unwrap()
        );
    }

    #[test]
    fn test_duration_div() {
        assert_eq!(TimeDelta::zero() / i32::MAX, TimeDelta::zero());
        assert_eq!(TimeDelta::zero() / i32::MIN, TimeDelta::zero());
        assert_eq!(
            TimeDelta::nanoseconds(123_456_789) / 1,
            TimeDelta::nanoseconds(123_456_789)
        );
        assert_eq!(
            TimeDelta::nanoseconds(123_456_789) / -1,
            -TimeDelta::nanoseconds(123_456_789)
        );
        assert_eq!(
            -TimeDelta::nanoseconds(123_456_789) / -1,
            TimeDelta::nanoseconds(123_456_789)
        );
        assert_eq!(
            -TimeDelta::nanoseconds(123_456_789) / 1,
            -TimeDelta::nanoseconds(123_456_789)
        );
        assert_eq!(
            TimeDelta::try_seconds(1).unwrap() / 3,
            TimeDelta::nanoseconds(333_333_333)
        );
        assert_eq!(
            TimeDelta::try_seconds(4).unwrap() / 3,
            TimeDelta::nanoseconds(1_333_333_333)
        );
        assert_eq!(
            TimeDelta::try_seconds(-1).unwrap() / 2,
            TimeDelta::try_milliseconds(-500).unwrap()
        );
        assert_eq!(
            TimeDelta::try_seconds(1).unwrap() / -2,
            TimeDelta::try_milliseconds(-500).unwrap()
        );
        assert_eq!(
            TimeDelta::try_seconds(-1).unwrap() / -2,
            TimeDelta::try_milliseconds(500).unwrap()
        );
        assert_eq!(
            TimeDelta::try_seconds(-4).unwrap() / 3,
            TimeDelta::nanoseconds(-1_333_333_333)
        );
        assert_eq!(
            TimeDelta::try_seconds(-4).unwrap() / -3,
            TimeDelta::nanoseconds(1_333_333_333)
        );
    }

    #[test]
    fn test_duration_sum() {
        let duration_list_1 = [TimeDelta::zero(), TimeDelta::try_seconds(1).unwrap()];
        let sum_1: TimeDelta = duration_list_1.iter().sum();
        assert_eq!(sum_1, TimeDelta::try_seconds(1).unwrap());

        let duration_list_2 = [
            TimeDelta::zero(),
            TimeDelta::try_seconds(1).unwrap(),
            TimeDelta::try_seconds(6).unwrap(),
            TimeDelta::try_seconds(10).unwrap(),
        ];
        let sum_2: TimeDelta = duration_list_2.iter().sum();
        assert_eq!(sum_2, TimeDelta::try_seconds(17).unwrap());

        let duration_arr = [
            TimeDelta::zero(),
            TimeDelta::try_seconds(1).unwrap(),
            TimeDelta::try_seconds(6).unwrap(),
            TimeDelta::try_seconds(10).unwrap(),
        ];
        let sum_3: TimeDelta = duration_arr.into_iter().sum();
        assert_eq!(sum_3, TimeDelta::try_seconds(17).unwrap());
    }

    #[test]
    fn test_duration_fmt() {
        assert_eq!(TimeDelta::zero().to_string(), "P0D");
        assert_eq!(TimeDelta::try_days(42).unwrap().to_string(), "PT3628800S");
        assert_eq!(TimeDelta::try_days(-42).unwrap().to_string(), "-PT3628800S");
        assert_eq!(TimeDelta::try_seconds(42).unwrap().to_string(), "PT42S");
        assert_eq!(
            TimeDelta::try_milliseconds(42).unwrap().to_string(),
            "PT0.042S"
        );
        assert_eq!(TimeDelta::microseconds(42).to_string(), "PT0.000042S");
        assert_eq!(TimeDelta::nanoseconds(42).to_string(), "PT0.000000042S");
        assert_eq!(
            (TimeDelta::try_days(7).unwrap() + TimeDelta::try_milliseconds(6543).unwrap())
                .to_string(),
            "PT604806.543S"
        );
        assert_eq!(
            TimeDelta::try_seconds(-86_401).unwrap().to_string(),
            "-PT86401S"
        );
        assert_eq!(TimeDelta::nanoseconds(-1).to_string(), "-PT0.000000001S");

        // the format specifier should have no effect on `TimeDelta`
        assert_eq!(
            format!(
                "{:30}",
                TimeDelta::try_days(1).unwrap() + TimeDelta::try_milliseconds(2345).unwrap()
            ),
            "PT86402.345S"
        );
    }

    #[test]
    fn test_to_std() {
        assert_eq!(
            TimeDelta::try_seconds(1).unwrap().to_std(),
            Ok(Duration::new(1, 0))
        );
        assert_eq!(
            TimeDelta::try_seconds(86_401).unwrap().to_std(),
            Ok(Duration::new(86_401, 0))
        );
        assert_eq!(
            TimeDelta::try_milliseconds(123).unwrap().to_std(),
            Ok(Duration::new(0, 123_000_000))
        );
        assert_eq!(
            TimeDelta::try_milliseconds(123_765).unwrap().to_std(),
            Ok(Duration::new(123, 765_000_000))
        );
        assert_eq!(
            TimeDelta::nanoseconds(777).to_std(),
            Ok(Duration::new(0, 777))
        );
        assert_eq!(
            MAX.to_std(),
            Ok(Duration::new(9_223_372_036_854_775, 807_000_000))
        );
        assert_eq!(
            TimeDelta::try_seconds(-1).unwrap().to_std(),
            Err(OutOfRangeError(()))
        );
        assert_eq!(
            TimeDelta::try_milliseconds(-1).unwrap().to_std(),
            Err(OutOfRangeError(()))
        );
    }

    #[test]
    fn test_from_std() {
        assert_eq!(
            Ok(TimeDelta::try_seconds(1).unwrap()),
            TimeDelta::from_std(Duration::new(1, 0))
        );
        assert_eq!(
            Ok(TimeDelta::try_seconds(86_401).unwrap()),
            TimeDelta::from_std(Duration::new(86_401, 0))
        );
        assert_eq!(
            Ok(TimeDelta::try_milliseconds(123).unwrap()),
            TimeDelta::from_std(Duration::new(0, 123_000_000))
        );
        assert_eq!(
            Ok(TimeDelta::try_milliseconds(123_765).unwrap()),
            TimeDelta::from_std(Duration::new(123, 765_000_000))
        );
        assert_eq!(
            Ok(TimeDelta::nanoseconds(777)),
            TimeDelta::from_std(Duration::new(0, 777))
        );
        assert_eq!(
            Ok(MAX),
            TimeDelta::from_std(Duration::new(9_223_372_036_854_775, 807_000_000))
        );
        assert_eq!(
            TimeDelta::from_std(Duration::new(9_223_372_036_854_776, 0)),
            Err(OutOfRangeError(()))
        );
        assert_eq!(
            TimeDelta::from_std(Duration::new(9_223_372_036_854_775, 807_000_001)),
            Err(OutOfRangeError(()))
        );
    }

    #[test]
    fn test_duration_const() {
        const ONE_WEEK: TimeDelta = expect(TimeDelta::try_weeks(1), "");
        const ONE_DAY: TimeDelta = expect(TimeDelta::try_days(1), "");
        const ONE_HOUR: TimeDelta = expect(TimeDelta::try_hours(1), "");
        const ONE_MINUTE: TimeDelta = expect(TimeDelta::try_minutes(1), "");
        const ONE_SECOND: TimeDelta = expect(TimeDelta::try_seconds(1), "");
        const ONE_MILLI: TimeDelta = expect(TimeDelta::try_milliseconds(1), "");
        const ONE_MICRO: TimeDelta = TimeDelta::microseconds(1);
        const ONE_NANO: TimeDelta = TimeDelta::nanoseconds(1);
        let combo: TimeDelta = ONE_WEEK
            + ONE_DAY
            + ONE_HOUR
            + ONE_MINUTE
            + ONE_SECOND
            + ONE_MILLI
            + ONE_MICRO
            + ONE_NANO;

        assert!(ONE_WEEK != TimeDelta::zero());
        assert!(ONE_DAY != TimeDelta::zero());
        assert!(ONE_HOUR != TimeDelta::zero());
        assert!(ONE_MINUTE != TimeDelta::zero());
        assert!(ONE_SECOND != TimeDelta::zero());
        assert!(ONE_MILLI != TimeDelta::zero());
        assert!(ONE_MICRO != TimeDelta::zero());
        assert!(ONE_NANO != TimeDelta::zero());
        assert_eq!(
            combo,
            TimeDelta::try_seconds(86400 * 7 + 86400 + 3600 + 60 + 1).unwrap()
                + TimeDelta::nanoseconds(1 + 1_000 + 1_000_000)
        );
    }
