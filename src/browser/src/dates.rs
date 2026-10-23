use chrono::{DateTime, Datelike, Duration, Months, NaiveDate, NaiveDateTime, Utc};
pub fn parse(value: &str) -> Option<DateTime<Utc>> {
    let s = value.trim().replacen(' ', "T", 1);
    DateTime::parse_from_rfc3339(&s)
        .map(|v| v.to_utc())
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|d| d.and_utc())
        })
        .or_else(|| {
            NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .ok()
                .and_then(|v| v.and_hms_opt(0, 0, 0))
                .map(|d| d.and_utc())
        })
}
pub fn format(value: &str) -> String {
    parse(value)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| value.into())
}
pub fn input(value: &str) -> String {
    parse(value)
        .map(|d| d.format("%Y-%m-%dT%H:%M:%S").to_string())
        .unwrap_or_default()
}
pub fn iso(value: &str) -> Option<String> {
    parse(value).map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Together {
    pub years: i32,
    pub months: i32,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}
pub fn together(start: DateTime<Utc>, end: DateTime<Utc>) -> Option<Together> {
    if start > end {
        return None;
    }
    let mut months = (end.year() - start.year()) * 12 + end.month() as i32 - start.month() as i32;
    let anniversary = |n: i32| start.checked_add_months(Months::new(n as u32));
    if anniversary(months)? > end {
        months -= 1;
    }
    let mut cursor = anniversary(months)?;
    let days = (end - cursor).num_days();
    cursor += Duration::days(days);
    let sec = (end - cursor).num_seconds();
    Some(Together {
        years: months / 12,
        months: months % 12,
        days,
        hours: sec / 3600,
        minutes: sec / 60 % 60,
        seconds: sec % 60,
    })
}
pub fn password_error(value: &str, allow_empty: bool) -> Option<String> {
    use crate::common::t;
    if value.is_empty() {
        return (!allow_empty)
            .then(|| t("请设置文章访问密码。", "Set an article access password."));
    }
    (!(12..=1024).contains(&value.len()) || value.chars().any(|c| c <= '\u{1f}' || c == '\u{7f}'))
        .then(|| {
            t(
                "密码须为 12–1024 个 UTF-8 字节，不能包含 ASCII 控制字符。",
                "Use 12–1024 UTF-8 bytes without ASCII control characters.",
            )
        })
}
pub fn filename_prefix(value: &str) -> &str {
    let mut end = 0;
    for (i, c) in value.char_indices() {
        if i + c.len_utf8() > 200 {
            break;
        }
        end = i + c.len_utf8();
    }
    &value[..end]
}
#[path = "../../../test/browser/unit/dates/tests.rs"]
#[cfg(test)]
mod tests;
