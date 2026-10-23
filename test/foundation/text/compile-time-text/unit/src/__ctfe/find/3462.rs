#[test]
fn test_bytes_starts_with() {
    assert!(bytes_starts_with(b"", b""));
    assert!(bytes_starts_with(b"a", b""));
    assert!(bytes_starts_with(b"a", b"a"));
    assert!(!bytes_starts_with(b"", b"a"));
    assert!(!bytes_starts_with(b"ba", b"a"));
}
