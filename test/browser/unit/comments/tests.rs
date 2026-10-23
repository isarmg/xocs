use super::*;
#[test]
fn pending_identity_and_unicode_trim() {
    assert!(valid_submission(
        &json!({"request_id":"12345678-1234-4123-8123-123456789abc","content":"hello","parent_comment_id":null})
    ));
    assert!(!valid_submission(
        &json!({"request_id":"12345678-1234-7123-8123-123456789abc","content":"hello","parent_comment_id":null})
    ));
    assert_eq!("\u{85} text \u{85}".trim(), "text");
}
