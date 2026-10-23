use super::*;
#[test]
fn utc_without_offset() {
    assert_eq!(format("2026-10-16 08:07:06"), "2026-10-16 08:07:06 UTC");
    assert_eq!(
        format("2026-10-16T08:07:06+08:00"),
        "2026-10-16 00:07:06 UTC"
    );
}
#[test]
fn anniversary_clamps_short_months() {
    let d = together(
        parse("2024-01-31T12:00:00Z").unwrap(),
        parse("2024-03-30T12:00:00Z").unwrap(),
    )
    .unwrap();
    assert_eq!((d.months, d.days), (1, 30));
}
#[test]
fn utf8_filename_limit() {
    let x = "文".repeat(80);
    assert_eq!(filename_prefix(&x).len(), 198);
}
#[test]
fn utf8_password() {
    assert!(password_error("四个汉字", false).is_none());
    assert!(password_error("short", false).is_some());
}
