#[test]
fn is_identifier_name_patched_rejects_katakana_dots() {
    // Katakana middle dots are valid identifier parts per Unicode 15+,
    // but we reject them in the patched version for compat with Unicode 4.1-15.
    // U+30FB KATAKANA MIDDLE DOT
    assert!(is_identifier_name("x\u{30FB}"));
    assert!(!is_identifier_name_patched("x\u{30FB}"));
    // U+FF65 HALFWIDTH KATAKANA MIDDLE DOT
    assert!(is_identifier_name("x\u{FF65}"));
    assert!(!is_identifier_name_patched("x\u{FF65}"));
    // As start character (neither is a valid start, so both should reject)
    assert!(!is_identifier_name("\u{30FB}"));
    assert!(!is_identifier_name_patched("\u{30FB}"));
    // Normal identifiers still work
    assert!(is_identifier_name_patched("foo"));
    assert!(is_identifier_name_patched("_bar"));
    assert!(is_identifier_name_patched("$baz"));
    assert!(is_identifier_name_patched("µ"));
    // Empty string rejected
    assert!(!is_identifier_name_patched(""));
}
