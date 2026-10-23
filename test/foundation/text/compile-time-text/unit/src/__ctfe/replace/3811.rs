#[test]
fn test_replace() {
    macro_rules! test_replace_str {
        ($input: expr, $replace_from: expr, $replace_to: expr) => {{
            const INPUT: &str = $input;
            const REPLACE_FROM: &str = $replace_from;
            const REPLACE_TO: &str = $replace_to;

            const CONSTFN: Replace<'static, 'static, &str> =
                Replace(INPUT, REPLACE_FROM, REPLACE_TO);
            const OUTPUT_LEN: usize = CONSTFN.output_len();

            let ans = INPUT.replace(REPLACE_FROM, REPLACE_TO);
            assert_eq!(OUTPUT_LEN, ans.len());

            let output_buf = CONSTFN.const_eval::<OUTPUT_LEN>();
            let output = output_buf.as_str();
            assert_eq!(output, ans);
        }};
    }

    test_replace_str!("", "", "");
    test_replace_str!("", "", "a");
    test_replace_str!("", "a", "");
    test_replace_str!("", "a", "b");
    test_replace_str!("a", "", "b");
    test_replace_str!("asd", "", "b");
    test_replace_str!("aba", "a", "c");
    test_replace_str!("this is old", "old", "new");
    test_replace_str!("我", "", "1");
    test_replace_str!("我", "", "我");
}
