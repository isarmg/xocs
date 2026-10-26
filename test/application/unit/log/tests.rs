use super::*;

#[test]
fn public_templates_and_product_records_share_one_exact_contract() {
    for event in CommonEvent::ALL {
        let record = LogRecord::common("example", event).unwrap();
        let line = record.json_line().unwrap();
        let parsed = query(&line[..], LogFilter::default(), QueryLimits::default()).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].event(), record.event());
        assert_eq!(parsed[0].scope(), Scope::Server);
        assert!(record.timestamp().ends_with('Z'));
    }
    assert!(
        LogRecord::server(
            "example",
            "device",
            "common.config.loaded",
            "another template",
            Level::Info
        )
        .is_err()
    );
    assert!(
        LogRecord::instance(
            "example",
            "device",
            "example.device.failed",
            "Device failed.",
            Level::Warn,
            ""
        )
        .is_err()
    );
}

#[test]
fn secrets_are_redacted_recursively_without_destroying_boolean_state_flags() {
    let record = LogRecord::instance("example", "device", "example.device.failed", "Device connection failed.", Level::Warn, "camera-id")
            .unwrap().with_attribute("status", json!({
                "Password":"p-sensitive", "accessTOKEN":"t-sensitive", "private-key":"k-sensitive",
                "nested":[{"api_key":17,"url":"rtsp://user:credential@camera/stream"}], "token_configured":true,
            })).unwrap();
    let bytes = record.json_line().unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    for secret in [
        "p-sensitive",
        "t-sensitive",
        "k-sensitive",
        "user:credential",
    ] {
        assert!(!text.contains(secret));
    }
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["attributes"]["status"]["token_configured"], true);
    assert_eq!(
        value["attributes"]["status"]["Password"]["configured"],
        true
    );
    assert_eq!(record.instance_id(), Some("camera-id"));
}

#[test]
fn query_filters_exact_instance_time_level_event_and_correlation_fields() {
    let first = LogRecord::instance(
        "example",
        "device",
        "example.device.failed",
        "Connection failed.",
        Level::Warn,
        "camera-1",
    )
    .unwrap()
    .with_request_id("request-1")
    .unwrap()
    .with_task_id("task-1")
    .unwrap();
    let second = LogRecord::instance(
        "example",
        "device",
        "example.device.failed",
        "Connection failed.",
        Level::Warn,
        "camera-10",
    )
    .unwrap();
    let mut bytes = first.json_line().unwrap();
    bytes.extend(second.json_line().unwrap());
    let filter = LogFilter {
        instance_id: Some("camera-1"),
        since: Some("2000-01-01T00:00:00Z"),
        until: Some("2100-01-01T00:00:00Z"),
        minimum_level: Some(Level::Warn),
        event: Some("example.device.failed"),
        request_id: Some("request-1"),
        task_id: Some("task-1"),
        scope: Some(Scope::Instance),
    };
    let records = query(&bytes[..], filter, QueryLimits::default()).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].instance_id(), Some("camera-1"));
    assert!(matches!(
        query(
            &bytes[..],
            LogFilter::default(),
            QueryLimits {
                max_records: 1,
                ..QueryLimits::default()
            }
        ),
        Err(LogError::QueryLimitExceeded)
    ));
    assert!(
        query(
            &bytes[..bytes.len() - 1],
            LogFilter::default(),
            QueryLimits::default()
        )
        .is_err()
    );
}

#[test]
fn record_and_query_budgets_reject_unbounded_inputs_without_partial_output() {
    let record = LogRecord::common("example", CommonEvent::RuntimeStarted).unwrap();
    assert!(matches!(
        record
            .clone()
            .with_attribute("data", "x".repeat(MAX_RECORD_BYTES)),
        Err(LogError::RecordTooLarge)
    ));
    let mut writer = Vec::new();
    let mut too_large = record;
    too_large
        .attributes
        .insert("data".into(), Value::String("x".repeat(MAX_RECORD_BYTES)));
    assert!(too_large.write_to(&mut writer).is_err());
    assert!(writer.is_empty());
    assert!(matches!(
        query(
            &vec![b'x'; MAX_RECORD_BYTES + 1][..],
            LogFilter::default(),
            QueryLimits::default()
        ),
        Err(LogError::RecordTooLarge)
    ));
}

#[test]
fn invalid_log_values_preserve_parse_sources_without_echoing_input() {
    use std::error::Error as _;
    let record = LogRecord::common("example", CommonEvent::RuntimeStarted).unwrap();
    let request_id = record
        .clone()
        .with_request_id("review sentinel")
        .unwrap_err();
    assert!(
        request_id
            .source()
            .unwrap()
            .is::<crate::error::InvalidRequestId>()
    );
    assert!(!request_id.to_string().contains("sentinel"));
    let error_code = record.with_error_code("review sentinel").unwrap_err();
    assert!(
        error_code
            .source()
            .unwrap()
            .is::<crate::error::InvalidErrorCode>()
    );
    assert!(!error_code.to_string().contains("sentinel"));

    let decode = query(
        &b"{\"review-sentinel\":}\n"[..],
        LogFilter::default(),
        QueryLimits::default(),
    )
    .unwrap_err();
    assert!(decode.source().unwrap().is::<serde_json::Error>());
    assert!(!decode.to_string().contains("sentinel"));
    let timestamp = query(
        &b""[..],
        LogFilter {
            since: Some("review-sentinelZ"),
            ..LogFilter::default()
        },
        QueryLimits::default(),
    )
    .unwrap_err();
    assert!(timestamp.source().unwrap().is::<chrono::ParseError>());
    assert!(!timestamp.to_string().contains("sentinel"));
}

#[cfg(unix)]
#[test]
fn malformed_archive_index_retains_the_parse_error() {
    use std::{error::Error as _, fs, os::unix::fs::PermissionsExt};
    let directory = tempfile::tempdir().unwrap();
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(directory.path().join("events.review-sentinel.jsonl"), b"").unwrap();
    let error = RotatingLogFile::open(directory.path(), "events", LogRetention::default())
        .err()
        .unwrap();
    assert!(error.source().unwrap().is::<std::num::ParseIntError>());
    assert!(!error.to_string().contains("sentinel"));
}

#[cfg(unix)]
#[test]
fn rotation_refuses_archive_aliases_outside_its_canonical_retention_names() {
    use std::{fs, os::unix::fs::PermissionsExt};
    for custom in [false, true] {
        for alias in ["01", "+1", "001"] {
            let directory = tempfile::tempdir().unwrap();
            fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let name = if custom {
                format!("events.log.{alias}")
            } else {
                format!("events.{alias}.jsonl")
            };
            let path = directory.path().join(name);
            let original = vec![b'x'; MAX_RECORD_BYTES + 1];
            fs::write(&path, &original).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            let retention = LogRetention {
                file_bytes: MAX_RECORD_BYTES as u64,
                archives: 2,
            };
            let result = if custom {
                RotatingLogFile::open_file(directory.path().join("events.log"), retention)
            } else {
                RotatingLogFile::open(directory.path(), "events", retention)
            };
            assert!(matches!(result, Err(LogError::UnsafeStorage)), "{path:?}");
            assert_eq!(fs::read(&path).unwrap(), original);
        }
    }
}

#[cfg(unix)]
#[test]
fn a_dedicated_log_directory_has_one_writer_across_overlapping_archive_names() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let directory = tempfile::tempdir().unwrap();
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let retention = LogRetention {
        file_bytes: MAX_RECORD_BYTES as u64,
        archives: 2,
    };
    let first = RotatingLogFile::open(directory.path(), "events", retention).unwrap();
    assert!(matches!(
        RotatingLogFile::open_file(directory.path().join("events.1.jsonl"), retention),
        Err(LogError::WriterLock(_))
    ));
    drop(first);
    let second =
        RotatingLogFile::open_file(directory.path().join("events.1.jsonl"), retention).unwrap();
    assert!(matches!(
        RotatingLogFile::open(directory.path(), "events", retention),
        Err(LogError::WriterLock(_))
    ));
    drop(second);
    RotatingLogFile::open(directory.path(), "events", retention).unwrap();
}

#[cfg(unix)]
#[test]
fn file_rotation_has_bounded_capacity_one_writer_and_safe_path_failures() {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };
    let directory = tempfile::tempdir().unwrap();
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let retention = LogRetention {
        file_bytes: MAX_RECORD_BYTES as u64,
        archives: 2,
    };
    let mut sink = RotatingLogFile::open(directory.path(), "events", retention).unwrap();
    assert!(RotatingLogFile::open(directory.path(), "events", retention).is_err());
    assert!(RotatingLogFile::open_file(directory.path().join("events.jsonl"), retention).is_err());
    let record = LogRecord::common("example", CommonEvent::RuntimeStarted)
        .unwrap()
        .with_attribute("padding", "x".repeat(4096))
        .unwrap();
    for _ in 0..30 {
        sink.write(&record).unwrap();
    }
    let mut count = 0;
    for entry in fs::read_dir(directory.path()).unwrap() {
        let entry = entry.unwrap();
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            count += 1;
            assert!(entry.metadata().unwrap().len() <= retention.file_bytes);
            let parsed = query(
                std::io::BufReader::new(fs::File::open(entry.path()).unwrap()),
                LogFilter::default(),
                QueryLimits::default(),
            )
            .unwrap();
            assert!(!parsed.is_empty());
        }
    }
    assert_eq!(count, 3);
    drop(sink);
    fs::remove_file(directory.path().join("events.jsonl")).unwrap();
    let other = directory.path().join("important");
    fs::write(&other, b"original").unwrap();
    symlink(&other, directory.path().join("events.jsonl")).unwrap();
    assert!(RotatingLogFile::open(directory.path(), "events", retention).is_err());
    assert_eq!(fs::read(other).unwrap(), b"original");
}
