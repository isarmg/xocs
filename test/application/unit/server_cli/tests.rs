use super::*;

#[test]
fn filesystem_failures_keep_machine_semantics_and_hide_diagnostics() {
    use crate::state_file::{Error, LockKind};
    let private = tempfile::tempdir().unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(private.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let directory = crate::state_file::PrivateStateDirectory::open(private.path()).unwrap();
    let missing = directory.open_existing("SECRET-path").err().unwrap();
    let busy = Error::LockUnavailable {
        kind: LockKind::Instance,
        source: io::Error::new(io::ErrorKind::WouldBlock, "SECRET-owner"),
    };
    for (error, code, retryable) in [
        (missing, "state.not_found", false),
        (busy, "state.lock_busy", true),
        (
            Error::OpenFile {
                path: "SECRET-path".into(),
                source: io::Error::new(io::ErrorKind::PermissionDenied, "SECRET-source"),
            },
            "state.permission_denied",
            false,
        ),
    ] {
        let error = state_error(&error);
        assert_eq!(error.code.as_str(), code);
        assert_eq!(error.retryable, retryable);
        assert!(!serde_json::to_string(&error).unwrap().contains("SECRET"));
    }
    let io = SnapshotError::Io(io::Error::other("SECRET-database"));
    assert_eq!(snapshot_error(&io).code.as_str(), "snapshot.io_failed");
    assert!(
        !serde_json::to_string(&snapshot_error(&io))
            .unwrap()
            .contains("SECRET")
    );
    assert!(snapshot_error(&SnapshotError::Busy).retryable);
    assert!(!snapshot_error(&SnapshotError::InvalidDatabase).retryable);
}

#[tokio::test]
async fn snapshot_worker_failure_keeps_source_and_hides_panic_details_on_the_wire() {
    use std::error::Error as _;

    let source = tokio::task::spawn_blocking(|| panic!("SECRET capture panic"))
        .await
        .unwrap_err();
    let failure = SnapshotError::CaptureTaskFailed(source);
    assert!(
        failure
            .source()
            .unwrap()
            .downcast_ref::<tokio::task::JoinError>()
            .unwrap()
            .is_panic()
    );
    let envelope = snapshot_error(&failure);
    assert_eq!(envelope.code.as_str(), "snapshot.capture_failed");
    assert!(!serde_json::to_string(&envelope).unwrap().contains("SECRET"));
}

#[test]
fn machine_errors_emit_one_contract_record_and_no_human_output() {
    let error = failure("contract_violation", "Configuration is invalid.");
    let mut output = Vec::new();
    let mut diagnostics = Vec::new();
    write_error(&error, true, &mut output, &mut diagnostics).unwrap();
    assert_eq!(
        serde_json::from_slice::<ErrorEnvelope>(&output).unwrap(),
        error
    );
    assert_eq!(output.iter().filter(|byte| **byte == b'\n').count(), 1);
    assert!(diagnostics.is_empty());
    output.clear();
    write_error(&error, false, &mut output, &mut diagnostics).unwrap();
    assert!(output.is_empty());
    assert!(
        String::from_utf8(diagnostics)
            .unwrap()
            .contains("contract_violation")
    );
}

#[test]
fn runtime_log_validation_never_creates_or_repairs_a_directory() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let parent = tempfile::tempdir().unwrap();
    let state = parent.path().join("state");
    std::fs::create_dir(&state).unwrap();
    std::fs::set_permissions(&state, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(validate_runtime_log_directory(&state).is_err());
    assert!(!state.join("logs").exists());
    std::fs::create_dir(state.join("logs")).unwrap();
    std::fs::set_permissions(state.join("logs"), std::fs::Permissions::from_mode(0o700)).unwrap();
    validate_runtime_log_directory(&state).unwrap();
    std::fs::set_permissions(state.join("logs"), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(validate_runtime_log_directory(&state).is_err());
    assert_eq!(
        std::fs::metadata(state.join("logs"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
    std::fs::remove_dir(state.join("logs")).unwrap();
    symlink(parent.path(), state.join("logs")).unwrap();
    assert!(validate_runtime_log_directory(&state).is_err());
    assert!(validate_runtime_log_directory(&state).is_err());
}
