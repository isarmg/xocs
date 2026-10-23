
    use super::super::CompactSerializer;
    use super::*;

    fn run_test<T: ESTree>(cases: &[(T, &str)]) {
        for (input, output) in cases {
            let mut serializer = CompactSerializer::default();
            input.serialize(&mut serializer);
            let s = serializer.into_string();
            assert_eq!(&s, output);
        }
    }

    #[test]
    fn serialize_bool() {
        run_test(&[(true, "true"), (false, "false")]);
    }

    #[test]
    fn serialize_f64() {
        run_test(&[
            (0.0, "0"),
            (1.0, "1"),
            (123_456.0, "123456"),
            (0.12345, "0.12345"),
            (123.45, "123.45"),
            (f64::MIN, "-1.7976931348623157e+308"),
            (f64::MAX, "1.7976931348623157e+308"),
            (f64::INFINITY, "1e+400"),
            (-f64::INFINITY, "-1e+400"),
        ]);
    }

    #[test]
    fn serialize_u8() {
        run_test(&[(0, "0"), (1, "1"), (123, "123"), (u8::MAX, "255")]);
    }

    #[test]
    fn serialize_u16() {
        run_test(&[(0, "0"), (1, "1"), (12_345, "12345"), (u16::MAX, "65535")]);
    }

    #[test]
    fn serialize_u32() {
        run_test(&[(0, "0"), (1, "1"), (123_456, "123456"), (u32::MAX, "4294967295")]);
    }

    #[test]
    fn serialize_u64() {
        run_test(&[(0, "0"), (1, "1"), (123_456, "123456"), (u64::MAX, "18446744073709551615")]);
    }

    #[test]
    fn serialize_u128() {
        run_test(&[
            (0, "0"),
            (1, "1"),
            (123_456, "123456"),
            (u128::MAX, "340282366920938463463374607431768211455"),
        ]);
    }

    #[test]
    fn serialize_usize() {
        run_test(&[(0, "0"), (1, "1"), (123_456, "123456"), (u32::MAX as usize, "4294967295")]);
    }

    #[test]
    fn serialize_i8() {
        run_test(&[
            (0, "0"),
            (1, "1"),
            (-1, "-1"),
            (123, "123"),
            (i8::MIN, "-128"),
            (i8::MAX, "127"),
        ]);
    }

    #[test]
    fn serialize_i16() {
        run_test(&[
            (0, "0"),
            (1, "1"),
            (-1, "-1"),
            (12_345, "12345"),
            (i16::MIN, "-32768"),
            (i16::MAX, "32767"),
        ]);
    }

    #[test]
    fn serialize_i32() {
        run_test(&[
            (0, "0"),
            (1, "1"),
            (-1, "-1"),
            (123_456, "123456"),
            (i32::MIN, "-2147483648"),
            (i32::MAX, "2147483647"),
        ]);
    }

    #[test]
    fn serialize_i64() {
        run_test(&[
            (0, "0"),
            (1, "1"),
            (-1, "-1"),
            (123_456, "123456"),
            (i64::MIN, "-9223372036854775808"),
            (i64::MAX, "9223372036854775807"),
        ]);
    }

    #[test]
    fn serialize_i128() {
        run_test(&[
            (0, "0"),
            (1, "1"),
            (-1, "-1"),
            (123_456, "123456"),
            (i128::MIN, "-170141183460469231731687303715884105728"),
            (i128::MAX, "170141183460469231731687303715884105727"),
        ]);
    }

    #[test]
    fn serialize_isize() {
        run_test(&[
            (0, "0"),
            (1, "1"),
            (-1, "-1"),
            (123_456, "123456"),
            (i32::MIN as isize, "-2147483648"),
            (i32::MAX as isize, "2147483647"),
        ]);
    }

    #[test]
    fn serialize_unit() {
        run_test(&[((), "null")]);
    }
