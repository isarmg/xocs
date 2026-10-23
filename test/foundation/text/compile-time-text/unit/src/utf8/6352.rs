#[test]
fn test_char_escape_debug() {
    macro_rules! test_char_escape_debug {
        ($ch: expr) => {{
            let e = CharEscapeDebug::new($ch);
            let output = e.as_str();
            let ans = $ch.escape_debug().to_string();
            assert_eq!(output, ans);
        }};
    }

    // ignore '\0' because the std behavior has been changed
    // fixed since v0.4
    for ch in '\u{1}'..='\u{7f}' {
        test_char_escape_debug!(ch);
    }

    // test_char_escape_debug!('我');
    test_char_escape_debug!('\u{10ffff}');
}
