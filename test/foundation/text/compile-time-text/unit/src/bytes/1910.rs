#[test]
fn test_reversed() {
    let arr = [0, 1];
    assert_eq!(reversed(arr), [1, 0]);

    let arr = [0, 1, 2];
    assert_eq!(reversed(arr), [2, 1, 0]);
}
