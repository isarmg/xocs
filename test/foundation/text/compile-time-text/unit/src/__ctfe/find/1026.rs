#[test]
fn test_bytes_contains() {
    let buf = b"abcdefgh";
    assert!(bytes_contains(buf, b""));
    assert!(bytes_contains(buf, b"a"));
    assert!(bytes_contains(buf, b"ef"));
    assert!(!bytes_contains(buf, b"xyz"));
}
