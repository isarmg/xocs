//! Date and JSON contracts exercised through the server's maintained dependency set.
use chrono::{DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, TimeDelta, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[test]
fn timestamp_json_preserves_instants_and_rejects_invalid_dates() {
    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Record {
        created_at: DateTime<Utc>,
    }
    let input = r#"{"created_at":"2024-03-01T00:59:59.123+01:00"}"#;
    let record: Record = serde_json::from_str(input).unwrap();
    assert_eq!(
        record.created_at.to_rfc3339(),
        "2024-02-29T23:59:59.123+00:00"
    );
    let encoded = serde_json::to_string(&record).unwrap();
    assert_eq!(encoded, r#"{"created_at":"2024-02-29T23:59:59.123Z"}"#);
    assert_eq!(serde_json::from_str::<Record>(&encoded).unwrap(), record);
    for invalid in [
        "2023-02-29T00:00:00Z",
        "2024-13-01T00:00:00Z",
        "2024-01-01T25:00:00Z",
    ] {
        assert!(
            serde_json::from_value::<Record>(serde_json::json!({ "created_at": invalid })).is_err()
        );
    }
}

#[test]
fn calendar_and_negative_fractional_timestamps_preserve_boundaries() {
    assert!(NaiveDate::from_ymd_opt(1900, 2, 29).is_none());
    assert!(NaiveDate::from_ymd_opt(2000, 2, 29).is_some());
    assert_eq!(
        NaiveDate::from_ymd_opt(2024, 2, 29)
            .unwrap()
            .succ_opt()
            .unwrap(),
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
    );
    let instant = DateTime::parse_from_rfc3339("1969-12-31T23:59:59.999999999Z").unwrap();
    assert_eq!(instant.timestamp(), -1);
    assert_eq!(instant.timestamp_subsec_nanos(), 999_999_999);
    assert_eq!(instant.timestamp_nanos_opt(), Some(-1));
    assert_eq!(Utc.timestamp_opt(-1, 999_999_999).unwrap(), instant);
}

#[test]
fn linux_local_timezone_observes_transition_rules() {
    const CHILD: &str = "BLOG_DATE_TIME_CONTRACT_CHILD";
    if let Ok(case) = std::env::var(CHILD) {
        check_local_transitions(&case);
        return;
    }

    let directory = tempfile::tempdir().unwrap();
    let fixture_path = directory.path().join("transitions.tzif");
    std::fs::write(&fixture_path, transition_fixture()).unwrap();
    let fixture_tz = format!(":{}", fixture_path.display());
    // Each child starts with its TZ, without mutating the parent's environment or
    // depending on the host timezone database and its thread-local cache.
    for (case, tz) in [
        ("north", "EST5EDT,M3.2.0/2,M11.1.0/2"),
        ("south", "AEST-10AEDT-11,M10.1.0/2,M4.1.0/2"),
        ("negative-late-start", "IST-1GMT0,M10.5.0/2,M3.5.0/2"),
        ("negative-early-start", "STD-2DST-1,M3.2.0/2,M11.1.0/2"),
        ("equal-offset", "STD-1DST-1,M3.2.0/2,M11.1.0/2"),
        ("same-month", "STD0DST-1,M3.2.0/2,M3.5.0/2"),
        ("tzif", fixture_tz.as_str()),
    ] {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "linux_local_timezone_observes_transition_rules"])
            .env(CHILD, case)
            .env("TZ", tz)
            .status()
            .unwrap();
        assert!(status.success(), "timezone case {case}: {tz}");
    }
}

fn local(month: u32, day: u32, hour: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2024, month, day)
        .expect("invariant: transition fixtures use valid 2024 calendar dates")
        .and_hms_opt(hour, 0, 0)
        .expect("invariant: transition fixtures use valid whole hours")
}

fn check_local_transitions(case: &str) {
    const HOUR: i32 = 3600;
    match case {
        "north" | "tzif" => {
            assert_single(local(1, 15, 12), -5 * HOUR);
            assert_single(local(7, 15, 12), -4 * HOUR);
            assert_gap(local(3, 10, 2), -5 * HOUR, -4 * HOUR);
            assert_fold(local(11, 3, 1), -4 * HOUR, -5 * HOUR);
            if case == "tzif" {
                // The fixture changes only the designation at this instant.
                for delta in [-1, 0, 1] {
                    assert_single(local(1, 1, 0) + TimeDelta::nanoseconds(delta), -5 * HOUR);
                }
            }
        }
        "south" => {
            assert_single(local(1, 15, 12), 11 * HOUR);
            assert_single(local(7, 15, 12), 10 * HOUR);
            assert_gap(local(10, 6, 2), 10 * HOUR, 11 * HOUR);
            assert_fold(local(4, 7, 1), 11 * HOUR, 10 * HOUR);
        }
        "negative-late-start" => {
            assert_single(local(1, 15, 12), 0);
            assert_single(local(7, 15, 12), HOUR);
            assert_fold(local(10, 27, 1), HOUR, 0);
            assert_gap(local(3, 31, 2), 0, HOUR);
        }
        "negative-early-start" => {
            assert_single(local(1, 15, 12), 2 * HOUR);
            assert_single(local(7, 15, 12), HOUR);
            assert_fold(local(3, 10, 1), 2 * HOUR, HOUR);
            assert_gap(local(11, 3, 2), HOUR, 2 * HOUR);
        }
        "equal-offset" => {
            for transition in [local(3, 10, 2), local(11, 3, 2)] {
                for delta in [-1, 0, 1] {
                    assert_single(transition + TimeDelta::nanoseconds(delta), HOUR);
                }
            }
        }
        "same-month" => {
            assert_gap(local(3, 10, 2), 0, HOUR);
            assert_fold(local(3, 31, 1), HOUR, 0);
        }
        _ => panic!("unknown timezone case: {case}"),
    }
}

fn assert_single(local: NaiveDateTime, expected_offset: i32) {
    let result = Local.from_local_datetime(&local);
    let LocalResult::Single(actual) = result else {
        panic!("{local} should have one offset, got {result:?}");
    };
    assert_eq!(
        actual.offset().local_minus_utc(),
        expected_offset,
        "{local}"
    );
    assert_eq!(actual.naive_local(), local);
    let roundtrip = actual.with_timezone(&Utc).with_timezone(&Local);
    assert_eq!(roundtrip.naive_local(), local);
    assert_eq!(roundtrip.offset(), actual.offset());
}

fn assert_gap(start: NaiveDateTime, before: i32, after: i32) {
    let end = start + TimeDelta::hours(1);
    assert_single(start - TimeDelta::nanoseconds(1), before);
    for local in [
        start,
        start + TimeDelta::minutes(30),
        end - TimeDelta::nanoseconds(1),
    ] {
        assert!(
            matches!(Local.from_local_datetime(&local), LocalResult::None),
            "{local}"
        );
    }
    assert_single(end, after);
}

fn assert_fold(start: NaiveDateTime, before: i32, after: i32) {
    let end = start + TimeDelta::hours(1);
    assert_single(start - TimeDelta::nanoseconds(1), before);
    for local in [
        start,
        start + TimeDelta::minutes(30),
        end - TimeDelta::nanoseconds(1),
    ] {
        let result = Local.from_local_datetime(&local);
        let LocalResult::Ambiguous(early, late) = result else {
            panic!("{local} must map to two distinct instants, got {result:?}");
        };
        assert_eq!(
            late.signed_duration_since(early).num_seconds(),
            3600,
            "{local}"
        );
        assert_eq!(early.offset().local_minus_utc(), before, "{local}");
        assert_eq!(late.offset().local_minus_utc(), after, "{local}");
        assert_eq!(early.naive_local(), local);
        assert_eq!(late.naive_local(), local);
        assert_eq!(
            early
                .with_timezone(&Utc)
                .with_timezone(&Local)
                .naive_local(),
            local
        );
        assert_eq!(
            late.with_timezone(&Utc).with_timezone(&Local).naive_local(),
            local
        );
        assert_eq!(result.earliest(), Some(early));
        assert_eq!(result.latest(), Some(late));
    }
    assert_single(end, after);
}

fn transition_fixture() -> Vec<u8> {
    // TZif v1: no leap seconds or footer, three transitions, and three local
    // types. This exercises the transition table independently of POSIX rules.
    let mut bytes = b"TZif\0".to_vec();
    bytes.extend_from_slice(&[0; 15]);
    for count in [0_u32, 0, 0, 3, 3, 12] {
        bytes.extend_from_slice(&count.to_be_bytes());
    }
    for timestamp in [
        1_704_085_200_i32, // 2024-01-01 05:00 UTC: EST -> WIN, same offset.
        1_710_054_000,     // 2024-03-10 07:00 UTC: WIN -> EDT.
        1_730_613_600,     // 2024-11-03 06:00 UTC: EDT -> EST.
    ] {
        bytes.extend_from_slice(&timestamp.to_be_bytes());
    }
    bytes.extend_from_slice(&[2, 1, 0]);
    for (offset, dst, name) in [(-18_000_i32, 0, 0), (-14_400, 1, 4), (-18_000, 0, 8)] {
        bytes.extend_from_slice(&offset.to_be_bytes());
        bytes.extend_from_slice(&[dst, name]);
    }
    bytes.extend_from_slice(b"EST\0EDT\0WIN\0");
    bytes
}
