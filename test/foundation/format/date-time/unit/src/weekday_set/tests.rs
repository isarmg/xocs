
    use crate::Weekday;

    use super::WeekdaySet;

    impl WeekdaySet {
        /// Iterate over all 128 possible sets, from `EMPTY` to `ALL`.
        fn iter_all() -> impl Iterator<Item = Self> {
            (0b0000_0000..0b1000_0000).map(Self)
        }
    }

    /// Panics if the 8-th bit of `self` is not 0.
    fn assert_8th_bit_invariant(days: WeekdaySet) {
        assert!(
            days.0 & 0b1000_0000 == 0,
            "the 8-th bit of {days:?} is not 0"
        );
    }

    #[test]
    fn debug_prints_8th_bit_if_not_zero() {
        assert_eq!(
            format!("{:?}", WeekdaySet(0b1000_0000)),
            "WeekdaySet(10000000)"
        );
    }

    #[test]
    fn bitwise_set_operations_preserve_8th_bit_invariant() {
        for set1 in WeekdaySet::iter_all() {
            for set2 in WeekdaySet::iter_all() {
                assert_8th_bit_invariant(set1.union(set2));
                assert_8th_bit_invariant(set1.intersection(set2));
                assert_8th_bit_invariant(set1.symmetric_difference(set2));
            }
        }
    }

    /// Test `split_at` on all possible arguments.
    #[test]
    fn split_at_is_equivalent_to_iterating() {
        use Weekday::*;

        // `split_at()` is used in `iter()`, so we must not iterate
        // over all days with `WeekdaySet::ALL.iter(Mon)`.
        const WEEK: [Weekday; 7] = [Mon, Tue, Wed, Thu, Fri, Sat, Sun];

        for weekdays in WeekdaySet::iter_all() {
            for split_day in WEEK {
                let expected_before: WeekdaySet = WEEK
                    .into_iter()
                    .take_while(|&day| day != split_day)
                    .filter(|&day| weekdays.contains(day))
                    .collect();
                let expected_after: WeekdaySet = WEEK
                    .into_iter()
                    .skip_while(|&day| day != split_day)
                    .filter(|&day| weekdays.contains(day))
                    .collect();

                assert_eq!(
                    (expected_before, expected_after),
                    weekdays.split_at(split_day),
                    "split_at({split_day}) failed for {weekdays}",
                );
            }
        }
    }
