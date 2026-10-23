#[test]
fn is_identifier_name_true() {
    let cases = [
        // 1 char ASCII
        "a",
        "z",
        "A",
        "Z",
        "_",
        "$",
        // 1 char Unicode
        "µ", // 2 bytes
        "ख", // 3 bytes
        "𐀀", // 4 bytes
        // Multiple chars ASCII
        "az",
        "AZ",
        "_a",
        "$Z",
        "a0",
        "A9",
        "_0",
        "$9",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_$",
        "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789$",
        "$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_",
        // Multiple chars Unicode
        "µख𐀀",
        // ASCII + Unicode, starting with ASCII
        "AµBखC𐀀D",
        // ASCII + Unicode, starting with Unicode
        "µAखB𐀀",
    ];

    for str in cases {
        assert!(is_identifier_name(str));
    }
}
