#[test]
fn is_identifier_name_false() {
    let cases = [
        // Empty string
        "",
        // 1 char ASCII
        "0",
        "9",
        "-",
        "~",
        "+",
        // 1 char Unicode
        "£", // 2 bytes
        "৸", // 3 bytes
        "𐄬", // 4 bytes
        // Multiple chars ASCII
        "0a",
        "9a",
        "-a",
        "+a",
        "a-Z",
        "A+z",
        "a-",
        "a+",
        // Multiple chars Unicode
        "£৸𐄬",
        // ASCII + Unicode, starting with ASCII
        "A£",
        "A৸",
        "A𐄬",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc£",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc৸",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc𐄬",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc£abcdefghijklmnopqrstuvwxyz",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc৸abcdefghijklmnopqrstuvwxyz",
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$abc𐄬abcdefghijklmnopqrstuvwxyz",
        // ASCII + Unicode, starting with Unicode
        "£A",
        "৸A",
        "𐄬A",
    ];

    for str in cases {
        assert!(!is_identifier_name(str));
    }
}
