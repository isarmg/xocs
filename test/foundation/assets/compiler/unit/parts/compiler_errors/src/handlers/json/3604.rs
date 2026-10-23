#[test]
fn test_escape() {
    assert_eq!(escape("a\nb").to_string(), r"a\nb");
    assert_eq!(escape("C:\\Oxc").to_string(), r"C:\\Oxc");
}
