#[test]
fn test_char_escape_unicode() {
    macro_rules! test_char_escape_unicode {
        ($ch: expr) => {{
            let e = CharEscapeUnicode::new($ch);
            let output = e.as_str();
            let ans = $ch.escape_unicode().to_string();
            assert_eq!(output, ans);
        }};
    }

    test_char_escape_unicode!('\0');
    test_char_escape_unicode!('我');
    test_char_escape_unicode!('\u{10ffff}');
}
