use super::*;
use tracing::collector::prelude::*;

#[derive(Clone)]
struct Output(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn tracing_events_inherit_scope_and_safe_fields_without_legacy_error_messages() {
    let output = Output(Arc::default());
    let layer = BlogStructuredLayer::new("example")
        .unwrap()
        .with_writer(output.clone());
    let subscriber = tracing::collector::registry().with(layer.clone());
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!(
            "device",
            instance_id = "camera-1",
            request_id = "req-1",
            password = "span-password"
        );
        let _entered = span.enter();
        tracing::warn!(
            event = "example.device.failed",
            error_code = "dependency_unavailable",
            error = "rtsp://user:secret@camera/stream",
            "raw internal failure includes invisible-secret"
        );
        tracing::info!(
            event = "example.device.retrying",
            attempt = 3u64,
            detail = "x".repeat(MAX_FIELD_BYTES + 100),
            "retry"
        );
    });
    assert_eq!(layer.rejected_count(), 0);
    let bytes = output.0.lock().unwrap().clone();
    let records = crate::log::query(
        &bytes[..],
        crate::log::LogFilter::default(),
        crate::log::QueryLimits::default(),
    )
    .unwrap();
    assert_eq!(records.len(), 2);
    assert!(
        records
            .iter()
            .all(|record| record.instance_id() == Some("camera-1"))
    );
    let text = String::from_utf8(bytes).unwrap();
    for secret in ["span-password", "user:secret", "invisible-secret"] {
        assert!(!text.contains(secret));
    }
    assert!(text.contains("fields_truncated"));
    assert!(text.contains("dependency_unavailable"));
}

#[test]
fn common_events_keep_registered_templates_and_reject_unknown_names() {
    let output = Output(Arc::default());
    let layer = BlogStructuredLayer::new("example")
        .unwrap()
        .with_writer(output.clone());
    let subscriber = tracing::collector::registry().with(layer.clone());
    tracing::subscriber::with_default(subscriber, || {
        tracing::error!(
            event = "common.runtime.started",
            component = "untrusted",
            "SECRET"
        );
        tracing::info!(event = "common.invented", "SECRET");
        tracing::info!(event = "common.runtime.started", instance_id = "camera-1");
        tracing::warn!(event = "common.task.failed", instance_id = "camera-1");
    });
    assert_eq!(layer.rejected_count(), 2);
    let bytes = output.0.lock().unwrap().clone();
    let values: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0]["component"], "runtime");
    assert_eq!(values[0]["level"], "INFO");
    assert_eq!(values[0]["message"], "Runtime started.");
    assert_eq!(values[1]["scope"], "instance");
    assert!(!std::str::from_utf8(&bytes).unwrap().contains("SECRET"));
}

#[cfg(unix)]
#[test]
fn installed_layer_switches_to_a_bounded_custom_file_and_counts_sink_failure() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let output = Output(Arc::default());
    let layer = BlogStructuredLayer::new("example")
        .unwrap()
        .with_writer(output.clone());
    let subscriber = tracing::collector::registry().with(layer.clone());
    let parent = tempfile::tempdir().unwrap();
    let directory = parent.path().join("logs");
    fs::create_dir(&directory).unwrap();
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
    let retention = crate::log::LogRetention {
        file_bytes: crate::log::MAX_RECORD_BYTES as u64,
        archives: 2,
    };
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(event = "common.config.loaded");
        layer
            .set_rotating_file(
                crate::log::RotatingLogFile::open_file(directory.join("custom.log"), retention)
                    .unwrap(),
            )
            .unwrap();
        for _ in 0..100 {
            tracing::info!(event = "common.runtime.started", padding = "x".repeat(1024));
        }
        let moved = parent.path().join("moved");
        fs::rename(&directory, &moved).unwrap();
        fs::create_dir(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
        tracing::info!(event = "common.runtime.stopped");
        for name in ["custom.log", "custom.log.1", "custom.log.2"] {
            assert!(fs::metadata(moved.join(name)).unwrap().len() <= retention.file_bytes);
        }
    });
    assert_eq!(layer.rejected_count(), 1);
    let bytes = output.0.lock().unwrap();
    assert_eq!(std::str::from_utf8(&bytes).unwrap().lines().count(), 1);
    assert!(
        !std::str::from_utf8(&bytes)
            .unwrap()
            .contains("common.runtime.started")
    );
}
