#[test]
fn test_bytes_ends_with() {
    assert!(bytes_ends_with(b"", b""));
    assert!(bytes_ends_with(b"a", b""));
    assert!(bytes_ends_with(b"a", b"a"));
    assert!(!bytes_ends_with(b"", b"a"));
    assert!(!bytes_ends_with(b"ab", b"a"));
}
