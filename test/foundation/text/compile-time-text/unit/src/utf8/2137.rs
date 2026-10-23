#[test]
fn test_char_encode_utf8() {
    macro_rules! test_char_encode_utf8 {
        ($ch: expr) => {{
            let e = CharEncodeUtf8::new($ch);
            let output = e.as_str();
            let mut ans = [0; 4];
            let ans = $ch.encode_utf8(&mut ans);
            assert_eq!(output, ans);
        }};
    }

    test_char_encode_utf8!('\0');
    test_char_encode_utf8!('我');
    test_char_encode_utf8!('\u{10ffff}');
}
