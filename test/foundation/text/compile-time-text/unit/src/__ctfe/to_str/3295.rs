#[test]
fn test_to_str() {
    macro_rules! test_to_str {
        ($ty: ty, $x: expr) => {{
            const X: $ty = $x;
            const OUTPUT_LEN: usize = ToStr(X).output_len();
            const OUTPUT_BUF: StrBuf<OUTPUT_LEN> = ToStr(X).const_eval();

            let output = OUTPUT_BUF.as_str();
            let ans = X.to_string();
            assert_eq!(OUTPUT_LEN, ans.len());
            assert_eq!(output, ans);
        }};
    }

    test_to_str!(&str, "lovelive superstar");

    test_to_str!(bool, true);
    test_to_str!(bool, false);

    test_to_str!(char, '鲤');
    test_to_str!(char, '鱼');

    test_to_str!(u8, 0);
    test_to_str!(u16, 0);
    test_to_str!(u32, 0);
    test_to_str!(u64, 0);
    test_to_str!(u128, 0);

    test_to_str!(u8, 10);
    test_to_str!(u8, 128);
    test_to_str!(u8, u8::MAX);

    test_to_str!(u64, 1);
    test_to_str!(u64, 10);
    test_to_str!(u64, 42);
    test_to_str!(u64, u64::MAX);

    test_to_str!(u128, u128::MAX);

    test_to_str!(i8, 0);
    test_to_str!(i16, 0);
    test_to_str!(i32, 0);
    test_to_str!(i64, 0);
    test_to_str!(i128, 0);

    test_to_str!(i8, -10);
    test_to_str!(i8, -42);
    test_to_str!(i8, i8::MAX);
    test_to_str!(i8, i8::MIN);

    test_to_str!(i64, 1);
    test_to_str!(i64, 10);
    test_to_str!(i64, -42);
    test_to_str!(i64, i64::MAX);
    test_to_str!(i64, i64::MIN);

    test_to_str!(i128, i128::MAX);
    test_to_str!(i128, i128::MIN);
}
