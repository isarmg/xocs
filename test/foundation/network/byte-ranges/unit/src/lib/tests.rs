
    use crate::{
        parse_range_header, EndPosition, ParsedRanges, RangeUnsatisfiableError, StartPosition,
        SyntacticallyCorrectRange,
    };
    use core::ops::RangeInclusive;

    const TEST_FILE_LENGTH: u64 = 10_000;
    /// Testing standard range compliance against <https://datatracker.ietf.org/doc/html/rfc7233>
    #[test]
    fn rfc_7233_standard_test1() {
        let input = "bytes=0-499";
        let expect =
            SyntacticallyCorrectRange::new(StartPosition::Index(0), EndPosition::Index(499));
        let actual = parse_range_header(input).unwrap();
        assert_eq!(single_range(expect), actual);
        let expect = RangeInclusive::new(0, 499);
        let actual = actual.validate(TEST_FILE_LENGTH).unwrap()[0].clone();
        assert_eq!(expect, actual);
    }

    #[test]
    fn rfc_7233_standard_test2() {
        let input = "bytes=500-999";
        let expect =
            SyntacticallyCorrectRange::new(StartPosition::Index(500), EndPosition::Index(999));
        let actual = parse_range_header(input).unwrap();
        assert_eq!(single_range(expect), actual);
        let expect = RangeInclusive::new(500, 999);
        let actual = actual.validate(TEST_FILE_LENGTH).unwrap()[0].clone();
        assert_eq!(expect, actual);
    }

    /// Testing suffix compliance against <https://datatracker.ietf.org/doc/html/rfc7233>
    #[test]
    fn rfc_7233_suffixed_test() {
        let input = "bytes=-500";
        let expect =
            SyntacticallyCorrectRange::new(StartPosition::FromLast(500), EndPosition::LastByte);
        let actual = parse_range_header(input).unwrap();
        assert_eq!(single_range(expect), actual);
        let expect = RangeInclusive::new(9500, 9999);
        let actual = actual.validate(10_000).unwrap()[0].clone();
        assert_eq!(expect, actual);
    }

    /// Testing open range compliance against <https://datatracker.ietf.org/doc/html/rfc7233>
    #[test]
    fn rfc_7233_open_range_test() {
        let input = "bytes=9500-";
        let expect =
            SyntacticallyCorrectRange::new(StartPosition::Index(9500), EndPosition::LastByte);
        let actual = parse_range_header(input).unwrap();
        assert_eq!(single_range(expect), actual);
        let expect = RangeInclusive::new(9500, 9999);
        let actual = actual.validate(10_000).unwrap()[0].clone();
        assert_eq!(expect, actual);
    }

    /// Testing first and last bytes compliance against <https://datatracker.ietf.org/doc/html/rfc7233>
    #[test]
    fn rfc_7233_first_and_last() {
        let input = "bytes=0-0, -1";
        let expect = vec![
            SyntacticallyCorrectRange::new(StartPosition::Index(0), EndPosition::Index(0)),
            SyntacticallyCorrectRange::new(StartPosition::FromLast(1), EndPosition::LastByte),
        ];
        let actual = parse_range_header(input).unwrap();
        assert_eq!(expect, actual.ranges);
        let expect = vec![0..=0, 9999..=9999];
        let actual = actual.validate(10_000).unwrap();
        assert_eq!(expect, actual);
    }

    #[test]
    fn parse_standard_range() {
        let input = "bytes=0-1023";
        let expect =
            SyntacticallyCorrectRange::new(StartPosition::Index(0), EndPosition::Index(1023));
        let actual = parse_range_header(input).unwrap();
        assert_eq!(single_range(expect), actual);
        let expect = RangeInclusive::new(0, 1023);
        let actual = actual.validate(TEST_FILE_LENGTH).unwrap()[0].clone();
        assert_eq!(expect, actual);
    }

    #[test]
    fn parse_open_ended_range() {
        let input = "bytes=0-";
        let expect = SyntacticallyCorrectRange::new(StartPosition::Index(0), EndPosition::LastByte);
        let actual = parse_range_header(input).unwrap();
        assert_eq!(single_range(expect), actual);
        let expect = RangeInclusive::new(0, TEST_FILE_LENGTH - 1);
        let actual = actual.validate(TEST_FILE_LENGTH).unwrap()[0].clone();
        assert_eq!(expect, actual);
    }

    #[test]
    fn parse_suffix_range_edge() {
        let input = &format!("bytes=-{}", TEST_FILE_LENGTH);
        let expect = SyntacticallyCorrectRange::new(
            StartPosition::FromLast(TEST_FILE_LENGTH),
            EndPosition::LastByte,
        );
        let actual = parse_range_header(input).unwrap();
        assert_eq!(single_range(expect), actual);
        let expect = RangeInclusive::new(0, TEST_FILE_LENGTH - 1);
        let actual = actual.validate(TEST_FILE_LENGTH).unwrap()[0].clone();
        assert_eq!(expect, actual);
    }

    #[test]
    fn parse_empty_as_invalid() {
        let input = "";
        let parsed = parse_range_header(input);
        assert_eq!(parsed, Err(RangeUnsatisfiableError::DoesNotStartWithToken));
    }

    #[test]
    fn parse_empty_range_as_invalid() {
        let input = "bytes=";
        let parsed = parse_range_header(input);
        // 0 is unexpected
        assert_eq!(
            parsed,
            Err(RangeUnsatisfiableError::UnexpectedNumberOfDashes)
        );
    }

    #[test]
    fn parse_range_starting_with_whitespace_as_invalid() {
        let input = "bytes= 0-15";
        let parsed = parse_range_header(input);
        // 0 is unexpected
        assert_eq!(parsed, Err(RangeUnsatisfiableError::StartsWithWhitespace));
    }

    #[test]
    fn parse_range_token_starting_with_whitespace_as_invalid() {
        let input = " bytes=0-15";
        let parsed = parse_range_header(input);
        // 0 is unexpected
        assert_eq!(parsed, Err(RangeUnsatisfiableError::DoesNotStartWithToken));
    }

    #[test]
    fn parse_range_strict_parse_numerical() {
        let input = "bytes=+0-15";
        let parsed = parse_range_header(input);
        // 0 is unexpected
        assert_eq!(parsed, Err(RangeUnsatisfiableError::BadStartOfRange));
    }

    #[test]
    fn parse_bad_unit_as_invalid() {
        let input = "abcde=0-10";
        let parsed = parse_range_header(input);
        assert_eq!(parsed, Err(RangeUnsatisfiableError::DoesNotStartWithToken));
    }

    #[test]
    fn parse_missing_equals_as_malformed() {
        let input = "bytes0-10";
        let parsed = parse_range_header(input);
        assert_eq!(parsed, Err(RangeUnsatisfiableError::DoesNotStartWithToken));
    }

    #[test]
    fn parse_negative_bad_characters_in_range_as_malformed() {
        let input = "bytes=1-10a";
        let parsed = parse_range_header(input);
        assert_eq!(parsed, Err(RangeUnsatisfiableError::BadEndOfRange));
    }

    #[test]
    fn parse_negative_numbers_as_malformed() {
        let input = "bytes=-1-10";
        let parsed = parse_range_header(input);
        // Becomes bad eor, since -1 signals suffixed range
        assert_eq!(parsed, Err(RangeUnsatisfiableError::BadEndOfRange));
    }

    #[test]
    fn parse_bad_characters_in_start_of_range() {
        let input = "bytes=a1-10";
        let parsed = parse_range_header(input);
        // Becomes bad eor, since -1 signals suffixed range
        assert_eq!(parsed, Err(RangeUnsatisfiableError::BadStartOfRange));
    }

    #[test]
    fn parse_out_of_bounds_overrun_as_content_length() {
        let input = &format!("bytes=0-{}", TEST_FILE_LENGTH);
        let expect = vec![RangeInclusive::new(0, TEST_FILE_LENGTH - 1)];
        let actual = parse_range_header(input)
            .unwrap()
            .validate(TEST_FILE_LENGTH)
            .unwrap();
        assert_eq!(expect, actual);
    }

    #[test]
    fn suffix_larger_than_file_selects_the_entire_file() {
        let input = &format!("bytes=-{}", TEST_FILE_LENGTH + 1);
        let parsed = parse_range_header(input)
            .unwrap()
            .validate(TEST_FILE_LENGTH);
        assert_eq!(parsed, Ok(vec![0..=TEST_FILE_LENGTH - 1]));
        assert_eq!(
            parse_range_header("bytes=-18446744073709551615")
                .unwrap()
                .validate(1),
            Ok(vec![0..=0])
        );
    }

    #[test]
    fn parse_zero_length_suffix_as_unsatisfiable() {
        let input = "bytes=-0";
        let parsed = parse_range_header(input);
        assert_eq!(parsed, Err(RangeUnsatisfiableError::ZeroSuffix));
    }

    #[test]
    fn parse_single_reversed_as_invalid() {
        let input = "bytes=15-0";
        let parsed = parse_range_header(input).unwrap();
        assert_eq!(
            parsed.validate(TEST_FILE_LENGTH),
            Err(RangeUnsatisfiableError::RangeReversed)
        );
    }

    #[test]
    fn parse_zero_range_as_invalid() {
        let input = "bytes=15-";
        let parsed = parse_range_header(input).unwrap();
        assert_eq!(parsed.validate(0), Err(RangeUnsatisfiableError::EmptyFile));
    }

    #[test]
    fn no_range_can_select_a_byte_from_an_empty_file() {
        for input in ["bytes=0-", "bytes=0-0", "bytes=-1", "bytes=0-99"] {
            assert_eq!(
                parse_range_header(input).unwrap().validate(0),
                Err(RangeUnsatisfiableError::EmptyFile),
                "{input}"
            );
        }
    }

    #[test]
    fn parse_multi_range() {
        let input = "bytes=0-1023, 2015-3000, 4000-4500, 8000-9999";
        let expected_ranges = vec![
            SyntacticallyCorrectRange::new(StartPosition::Index(0), EndPosition::Index(1023)),
            SyntacticallyCorrectRange::new(StartPosition::Index(2015), EndPosition::Index(3000)),
            SyntacticallyCorrectRange::new(StartPosition::Index(4000), EndPosition::Index(4500)),
            SyntacticallyCorrectRange::new(StartPosition::Index(8000), EndPosition::Index(9999)),
        ];
        let parsed = parse_range_header(input).unwrap();
        assert_eq!(expected_ranges, parsed.ranges);
        let validated = parsed.validate(TEST_FILE_LENGTH).unwrap();
        assert_eq!(
            vec![0..=1023, 2015..=3000, 4000..=4500, 8000..=9999],
            validated
        );
    }

    #[test]
    fn parse_multi_range_with_open() {
        let input = "bytes=0-1023, 1024-";
        let expected_ranges = vec![
            SyntacticallyCorrectRange::new(StartPosition::Index(0), EndPosition::Index(1023)),
            SyntacticallyCorrectRange::new(StartPosition::Index(1024), EndPosition::LastByte),
        ];
        let parsed = parse_range_header(input).unwrap();
        assert_eq!(expected_ranges, parsed.ranges);
        let validated = parsed.validate(TEST_FILE_LENGTH).unwrap();
        assert_eq!(vec![0..=1023, 1024..=9999], validated);
    }

    #[test]
    fn parse_multi_range_with_suffix() {
        let input = "bytes=0-1023, -1000";
        let expected_ranges = vec![
            SyntacticallyCorrectRange::new(StartPosition::Index(0), EndPosition::Index(1023)),
            SyntacticallyCorrectRange::new(StartPosition::FromLast(1000), EndPosition::LastByte),
        ];
        let parsed = parse_range_header(input).unwrap();
        assert_eq!(expected_ranges, parsed.ranges);
        assert_eq!(expected_ranges, parsed.ranges);
        let validated = parsed.validate(TEST_FILE_LENGTH).unwrap();
        assert_eq!(vec![0..=1023, 9000..=9999], validated);
    }

    #[test]
    fn parse_overlapping_multi_range_as_unsatisfiable_standard() {
        let input = "bytes=0-1023, 500-800";
        assert_validation_err(input, RangeUnsatisfiableError::OverlappingRanges);
        let input = "bytes=0-0, 0-15";
        assert_validation_err(input, RangeUnsatisfiableError::OverlappingRanges);
        let input = "bytes=0-20, 20-35";
        assert_validation_err(input, RangeUnsatisfiableError::OverlappingRanges);
    }

    #[test]
    fn parse_overlapping_multi_range_as_unsatisfiable_open() {
        let input = "bytes=0-, 5000-6000";
        assert_validation_err(input, RangeUnsatisfiableError::OverlappingRanges);
    }

    #[test]
    fn parse_overlapping_multi_range_as_unsatisfiable_suffixed() {
        let input = "bytes=8000-9000, -1001";
        assert_validation_err(input, RangeUnsatisfiableError::OverlappingRanges);
        let input = "bytes=8000-9000, -1000";
        assert_validation_err(input, RangeUnsatisfiableError::OverlappingRanges);
        // This doesn't overlap
        let input = "bytes=8000-9000, -999";
        parse_range_header(input)
            .unwrap()
            .validate(TEST_FILE_LENGTH)
            .unwrap();
    }

    #[test]
    fn parse_overlapping_multi_range_as_unsatisfiable_suffixed_open() {
        let input = "bytes=0-, -1";
        assert_validation_err(input, RangeUnsatisfiableError::OverlappingRanges);
    }

    #[test]
    fn parse_multi_range_with_a_reversed_as_invalid() {
        let input = "bytes=0-15, 30-20";
        assert_validation_err(input, RangeUnsatisfiableError::RangeReversed);
    }

    fn assert_validation_err(input: &str, err: RangeUnsatisfiableError) {
        let parsed = parse_range_header(input)
            .unwrap()
            .validate(TEST_FILE_LENGTH);
        assert_eq!(Err(err), parsed);
    }

    #[test]
    fn parse_multi_range_rejects_invalid() {
        let input = "bytes=0-15, 25, 9, ";
        let parsed = parse_range_header(input);
        assert!(parsed.is_err());
    }

    #[test]
    fn unsorted_validation_matches_pairwise_overlap_reference() {
        // Exhaustively compare all triples of small ranges, including reversed,
        // duplicate, touching, nested and disjoint intervals. Keep the reference
        // deliberately independent of the sorted production algorithm.
        let candidates: Vec<_> = (0..4)
            .flat_map(|start| (0..4).map(move |end| start..=end))
            .collect();
        for first in &candidates {
            for second in &candidates {
                for third in &candidates {
                    let ranges = [first, second, third];
                    let expected = if ranges.iter().any(|r| r.start() > r.end()) {
                        Err(RangeUnsatisfiableError::RangeReversed)
                    } else if (0..ranges.len()).any(|i| {
                        (i + 1..ranges.len()).any(|j| {
                            ranges[i].start() <= ranges[j].end()
                                && ranges[j].start() <= ranges[i].end()
                        })
                    }) {
                        Err(RangeUnsatisfiableError::OverlappingRanges)
                    } else {
                        Ok(ranges.iter().map(|r| (*r).clone()).collect())
                    };
                    let input = format!(
                        "bytes={}-{},{}-{},{}-{}",
                        first.start(),
                        first.end(),
                        second.start(),
                        second.end(),
                        third.start(),
                        third.end(),
                    );
                    assert_eq!(parse_range_header(&input).unwrap().validate(4), expected);
                }
            }
        }
    }

    #[test]
    fn malformed_ascii_in_range_numbers_is_rejected() {
        for byte in 0..=127u8 {
            if byte.is_ascii_digit() {
                continue;
            }
            for input in [
                format!("bytes=1{}2-5", char::from(byte)),
                format!("bytes=1-2{}3", char::from(byte)),
                format!("bytes=-2{}3", char::from(byte)),
            ] {
                assert!(parse_range_header(&input).is_err(), "{:?}", input);
            }
        }
    }

    #[test]
    fn many_disjoint_ranges_keep_request_order() {
        let expected: Vec<_> = (0..8192u64).rev().map(|i| i * 2..=i * 2).collect();
        let ranges = expected
            .iter()
            .map(|r| {
                SyntacticallyCorrectRange::new(
                    StartPosition::Index(*r.start()),
                    EndPosition::Index(*r.end()),
                )
            })
            .collect();
        assert_eq!(ParsedRanges::new(ranges).validate(16384), Ok(expected));
    }

    fn single_range(syntactically_correct: SyntacticallyCorrectRange) -> ParsedRanges {
        ParsedRanges::new(vec![syntactically_correct])
    }
