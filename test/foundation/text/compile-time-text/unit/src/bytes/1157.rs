#[test]
fn test_subslice() {
    let buf = b"abcdefgh";
    assert_eq!(subslice(buf, 0..0), &[]);
    assert_eq!(subslice(buf, 0..1), b"a");
    assert_eq!(subslice(buf, 1..3), b"bc");
    assert_eq!(subslice(buf, 7..8), b"h");

    assert_eq!(subslice::<u8>(&[], 0..0), &[]);
}
