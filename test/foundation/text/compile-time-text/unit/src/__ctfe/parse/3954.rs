#[test]
fn test_parse() {
    macro_rules! test_parse {
        ($s: expr, $ty: tt) => {{
            const OUTPUT: $ty = $crate::parse!($s, $ty);
            let ans: $ty = $s.parse().unwrap();
            assert_eq!(OUTPUT, ans)
        }};
    }

    test_parse!("true", bool);
    test_parse!("false", bool);

    test_parse!("啊", char);

    test_parse!("0", u8);
    test_parse!("-1", i8);
    test_parse!("+42000", u32);
    test_parse!("-42000", i32);
}
