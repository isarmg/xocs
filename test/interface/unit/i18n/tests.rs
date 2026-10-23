use super::*;
#[test]
fn locale_admission_matches_current_contract() {
    assert_eq!(supported("zh-Hans-CN"), Some("zh-CN"));
    assert_eq!(supported("en-US"), Some("en"));
    assert_eq!(supported("zh!"), None);
    assert_eq!(supported("en-"), None);
    assert_eq!(supported("fr"), None);
}
