#[test]
fn test_str_chars() {
    const X: &str = "唐可可";
    const OUTPUT_LEN: usize = str_count_chars(X);
    const OUTPUT_BUF: [char; OUTPUT_LEN] =
        unsafe { core::mem::transmute(str_chars::<OUTPUT_LEN>(X)) };
    let ans = X.chars().collect::<Vec<_>>();
    assert_eq!(OUTPUT_BUF, ans.as_slice());
}
