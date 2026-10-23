
    use super::*;

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_remove_ascii_whitespace() {
        let cases = [
            "\0\0\0\0",
            "abcd",
            "ab\tcd",
            "ab\ncd",
            "ab\x0Ccd",
            "ab\rcd",
            "ab cd",
            "ab\t\n\x0C\r cd",
            "ab\t\n\x0C\r =\t\n\x0C\r =\t\n\x0C\r ",
        ];

        let check = |case: &str, repeat: usize| {
            let mut buf = case.repeat(repeat).into_bytes();
            let expected = {
                let mut v = buf.clone();
                v.retain(|c| !c.is_ascii_whitespace());
                v
            };
            let ans = remove_ascii_whitespace_inplace(&mut buf);
            assert_eq!(ans, &*expected, "case = {case:?}");
        };

        for case in cases {
            check(case, 1);

            if cfg!(not(miri)) {
                check(case, 10);
            }
        }
    }
