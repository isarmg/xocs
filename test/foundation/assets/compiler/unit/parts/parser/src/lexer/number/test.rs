
    use super::*;

    #[expect(clippy::cast_precision_loss)]
    fn assert_all_ints_eq<I>(test_cases: I, kind: Kind, has_sep: bool)
    where
        I: IntoIterator<Item = (&'static str, i64)>,
    {
        for (s, expected) in test_cases {
            let parsed = parse_int(s, kind, has_sep);
            assert_eq!(
                parsed,
                Ok(expected as f64),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }
    fn assert_all_floats_eq<I>(test_cases: I, has_sep: bool)
    where
        I: IntoIterator<Item = (&'static str, f64)>,
    {
        for (s, expected) in test_cases {
            let parsed = parse_float(s, has_sep);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }

    const _: () = {
        // decimal
        assert!(decimal_byte_to_value(b'0') == 0);
        assert!(decimal_byte_to_value(b'9') == 9);

        // binary
        assert!(binary_byte_to_value(b'0') == 0);
        assert!(binary_byte_to_value(b'1') == 1);

        // octal
        assert!(octal_byte_to_value(b'0') == 0);
        assert!(octal_byte_to_value(b'7') == 7);

        // hex
        assert!(hex_byte_to_value(b'0') == 0);
        assert!(hex_byte_to_value(b'9') == 9);
        assert!(hex_byte_to_value(b'A') == 10);
        assert!(hex_byte_to_value(b'F') == 15);
        assert!(hex_byte_to_value(b'a') == 10);
        assert!(hex_byte_to_value(b'f') == 15);
    };

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn test_int_precision() {
        assert_eq!(
            // 18446744073709551616 = 1 << 64
            parse_int("18446744073709551616", Kind::Decimal, false),
            Ok(18446744073709551616_i128 as f64)
        );
        // This tests for imprecision which results from using `mul_add` loop
        assert_eq!(
            parse_int("12300000000000000000000000", Kind::Decimal, false),
            Ok(12300000000000000000000000_i128 as f64)
        );
        assert_eq!(
            // 0x10000000000000000 = 1 << 64
            parse_int("0x10000000000000000", Kind::Hex, false),
            Ok(0x10000000000000000_i128 as f64)
        );
        assert_eq!(
            // 0o2000000000000000000000 = 1 << 64
            parse_int("0o2000000000000000000000", Kind::Octal, false),
            Ok(0o2000000000000000000000_i128 as f64)
        );
        assert_eq!(
            // 0b10000000000000000000000000000000000000000000000000000000000000000 = 1 << 64
            parse_int(
                "0b10000000000000000000000000000000000000000000000000000000000000000",
                Kind::Binary,
                false
            ),
            Ok(0b10000000000000000000000000000000000000000000000000000000000000000_i128 as f64)
        );
    }

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn test_nondecimal_rounding() {
        let expected = 0x10000000000000801_i128 as f64;
        assert_eq!(parse_int("0x10000000000000801", Kind::Hex, false), Ok(expected));
        assert_eq!(parse_int("0x1_0000_0000_0000_0801", Kind::Hex, true), Ok(expected));
        assert_eq!(
            parse_int(
                "0b10000000000000000000000000000000000000000000000000000100000000001",
                Kind::Binary,
                false
            ),
            Ok(expected)
        );
        assert_eq!(
            parse_int(
                "0b1_00000000_00000000_00000000_00000000_00000000_00000000_00001000_00000001",
                Kind::Binary,
                true
            ),
            Ok(expected)
        );
        assert_eq!(parse_int("0o2000000000000000004001", Kind::Octal, false), Ok(expected));
        assert_eq!(parse_int("0o2_000_000_000_000_000_004_001", Kind::Octal, true), Ok(expected));
    }

    #[test]
    #[expect(clippy::cast_precision_loss)]
    fn test_nondecimal_rounding_boundaries() {
        for bits in 54..128 {
            let base = 1_u128 << bits;
            let half_ulp = 1_u128 << (bits - 53);
            // Both even and odd significands, immediately below, at, and above halfway.
            for offset in [
                half_ulp - 1,
                half_ulp,
                half_ulp + 1,
                3 * half_ulp - 1,
                3 * half_ulp,
                3 * half_ulp + 1,
            ] {
                let value = base + offset;
                for (digits, prefix, kind) in [
                    (format!("{value:b}"), "0b", Kind::Binary),
                    (format!("{value:o}"), "0o", Kind::Octal),
                    (format!("{value:x}"), "0x", Kind::Hex),
                    (format!("{value:o}"), "0", Kind::Octal),
                ] {
                    for leading in ["", "00000000000000000000"] {
                        let literal = format!("{prefix}{leading}{digits}");
                        assert_eq!(parse_int(&literal, kind, false), Ok(value as f64), "{literal}");
                        if prefix != "0" {
                            let separated =
                                digits.chars().map(|c| c.to_string()).collect::<Vec<_>>().join("_");
                            let literal = format!("{prefix}{leading}{separated}");
                            assert_eq!(
                                parse_int(&literal, kind, true),
                                Ok(value as f64),
                                "{literal}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_nondecimal_rounding_extremes() {
        for (digits, expected) in [
            // A sticky bit hundreds of digits after the rounding bit.
            (
                format!("1{}1{}1", "0".repeat(52), "0".repeat(969)),
                f64::from_bits(0x7fe0000000000001),
            ),
            // Largest finite f64, then immediately below and at the overflow midpoint.
            (format!("{}{}", "1".repeat(53), "0".repeat(971)), f64::MAX),
            (format!("{}0{}", "1".repeat(53), "1".repeat(970)), f64::MAX),
            (format!("{}{}", "1".repeat(54), "0".repeat(970)), f64::INFINITY),
            (format!("1{}", "0".repeat(1100)), f64::INFINITY),
            ("0".repeat(1100), 0.0),
            (format!("{}1", "0".repeat(1100)), 1.0),
        ] {
            // Convert the same exact integer to every radix, including separators.
            let value = BigInt::from_str_radix(&digits, 2).unwrap();
            for (radix, prefix, kind) in
                [(2, "0b", Kind::Binary), (8, "0o", Kind::Octal), (16, "0x", Kind::Hex)]
            {
                let digits = if radix == 2 { digits.clone() } else { value.to_str_radix(radix) };
                let literal = format!("{prefix}{}{digits}", "0".repeat(65));
                assert_eq!(parse_int(&literal, kind, false), Ok(expected), "{literal}");
                let separated = digits.chars().map(|c| c.to_string()).collect::<Vec<_>>().join("_");
                let literal = format!("{prefix}{separated}");
                assert_eq!(parse_int(&literal, kind, true), Ok(expected), "{literal}");
            }
        }
    }

    #[test]
    fn test_large_number_of_leading_zeros() {
        assert_all_ints_eq(
            vec![("000000000000000000000000000000000000000000000000000000001", 1)],
            Kind::Decimal,
            false,
        );
    }

    #[test]
    fn test_float_precision() {
        let cases = vec![
            ("1.7976931348623157e+308", 1.7976931348623157e+308),
            ("0.000000001", 0.000_000_001),
        ];
        assert_all_floats_eq(cases, false);
    }

    #[test]
    fn test_parse_int_no_sep() {
        let decimal: Vec<(&str, i64)> = vec![
            // normal
            ("0", 0),
            ("1", 1),
            ("000000000000", 0),
            ("9007199254740991", 9007199254740991), // max safe integer, 2^53 - 1
        ];
        let binary = vec![
            ("0b0", 0b0),
            ("0b1", 0b1),
            ("0b10", 0b10),
            ("0b110001001000100", 0b110001001000100),
            ("0b110001001000100", 0b110001001000100),
        ];
        let octal = vec![("0o0", 0o0), ("0o1", 0o1), ("0o10", 0o10), ("0o777", 0o777)];
        let hex: Vec<(&str, i64)> = vec![
            ("0x0", 0x0),
            ("0X0", 0x0),
            ("0xFF", 0xFF),
            ("0xc", 0xc), // :)
            ("0xdeadbeef", 0xdeadbeef),
            ("0xFfEeDdCcBbAa", 0xFfEeDdCcBbAa),
        ];

        assert_all_ints_eq(decimal, Kind::Decimal, false);
        assert_all_ints_eq(binary, Kind::Binary, false);
        assert_all_ints_eq(octal, Kind::Octal, false);
        assert_all_ints_eq(hex, Kind::Hex, false);
    }

    #[test]
    fn test_parse_int_with_sep() {
        let decimal: Vec<(&str, i64)> = vec![
            // still works without separators
            ("0", 0),
            ("1", 1),
            ("1_000_000", 1_000_000),
            ("000000000000", 0),
            ("9_007_199_254_740_991", 9_007_199_254_740_991), // max safe integer, 2^53 - 1
            // still works for illegal tokens
            ("1___000_000", 1_000_000),
            ("1_", 1),
            ("_1", 1),
        ];

        let binary = vec![
            ("0b0", 0b0),
            ("0b1", 0b1),
            ("0b10", 0b10),
            ("0b110001001000100", 0b110001001000100),
            ("0b110001001000100", 0b110001001000100),
            ("0b1_1000_1001_0001_0000", 0b1_1000_1001_0001_0000),
            // still works for illegal tokens
            ("0b1_0000__0000", 0b1_0000_0000),
            ("0b1_", 0b1),
            ("0b_0", 0b0),
        ];

        let octal = vec![
            ("0o0", 0o0),
            ("0o1", 0o1),
            ("0o10", 0o10),
            ("0o777", 0o777),
            ("0o7_7_7", 0o777),
            ("0o77_73_72", 0o77_73_72),
            // still works for illegal tokens
            ("0o1_0000__0000", 0o100_000_000),
            ("0o1_", 0o1),
            ("0o_0", 0o0),
        ];

        let hex: Vec<(&str, i64)> = vec![
            // still works without separators
            ("0x0", 0x0),
            ("0X0", 0x0),
            ("0xFF", 0xFF),
            ("0xFF_AA_11", 0xFFAA11),
            ("0xdead_beef", 0xdead_beef),
            ("0xFf_Ee_Dd_Cc_Bb_Aa", 0xFfEe_DdCc_BbAa),
            ("0xFfEe_DdCc_BbAa", 0xFfEe_DdCc_BbAa),
            // still works for illegal tokens
            ("0x1_0000__0000", 0x100_000_000),
            ("0x1_", 0x1),
            ("0x_0", 0x0),
        ];

        assert_all_ints_eq(decimal, Kind::Decimal, true);
        assert_all_ints_eq(binary, Kind::Binary, true);
        assert_all_ints_eq(octal, Kind::Octal, true);
        assert_all_ints_eq(hex, Kind::Hex, true);
    }

    #[test]
    fn test_decimal() {
        let no_sep: Vec<(&'static str, f64)> =
            vec![("0", 0.0), ("1.0", 1.0), ("1.1", 1.1), ("25.125", 25.125)];

        let sep: Vec<(&'static str, f64)> = vec![
            ("1_000.0", 1000.0),
            ("1.5_000", 1.5),
            // works on invalid tokens
            ("_0._5", 0.5),
            ("0._5", 0.5),
            ("0.5_", 0.5),
        ];

        assert_all_floats_eq(no_sep.clone(), false);
        assert_all_floats_eq(sep.clone(), true);
        for (s, expected) in no_sep {
            let parsed = parse_float(s, false);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
        for (s, expected) in sep {
            let parsed = parse_float(s, true);
            assert_eq!(
                parsed,
                Ok(expected),
                "expected {s} to parse to {expected}, but got {parsed:?}"
            );
        }
    }
