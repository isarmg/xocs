#[test]
fn test_find_str_decimal_literal_prefix() {
    assert_eq!(find_str_decimal_literal_prefix("Infinitya"), Some("Infinity"));
    assert_eq!(find_str_decimal_literal_prefix("+Infinitya"), Some("+Infinity"));
    assert_eq!(find_str_decimal_literal_prefix("-Infinitya"), Some("-Infinity"));
    assert_eq!(find_str_decimal_literal_prefix("0a"), Some("0"));
    assert_eq!(find_str_decimal_literal_prefix("+0a"), Some("+0"));
    assert_eq!(find_str_decimal_literal_prefix("-0a"), Some("-0"));
    assert_eq!(find_str_decimal_literal_prefix("0."), Some("0"));
    assert_eq!(find_str_decimal_literal_prefix("0.e"), Some("0"));
    assert_eq!(find_str_decimal_literal_prefix("0.e1"), Some("0"));
    assert_eq!(find_str_decimal_literal_prefix("0.1"), Some("0.1"));
    assert_eq!(find_str_decimal_literal_prefix("0.1."), Some("0.1"));
    assert_eq!(find_str_decimal_literal_prefix("0.1e"), Some("0.1"));
    assert_eq!(find_str_decimal_literal_prefix("0.1e1"), Some("0.1e1"));
    assert_eq!(find_str_decimal_literal_prefix(".1"), Some(".1"));
    assert_eq!(find_str_decimal_literal_prefix(".1."), Some(".1"));
    assert_eq!(find_str_decimal_literal_prefix(".1e"), Some(".1"));
    assert_eq!(find_str_decimal_literal_prefix(".1e1"), Some(".1e1"));
    assert_eq!(find_str_decimal_literal_prefix("1_"), Some("1"));
    assert_eq!(find_str_decimal_literal_prefix("1_1"), Some("1_1"));
    assert_eq!(find_str_decimal_literal_prefix("1_1_"), Some("1_1"));
}
