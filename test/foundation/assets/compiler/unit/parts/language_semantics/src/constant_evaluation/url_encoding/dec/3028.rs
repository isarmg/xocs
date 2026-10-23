#[test]
fn dec_borrows() {
    assert!(matches!(decode("hello".into(), |_| false), Some(Cow::Borrowed("hello"))));
    assert!(matches!(decode("hello%20".into(), |_| false), Some(Cow::Owned(s)) if s == "hello "));
    assert!(matches!(decode("%20hello".into(), |_| false), Some(Cow::Owned(s)) if s == " hello"));
}
