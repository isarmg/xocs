//! Common CLI behavior, with product commands and business initialization
//! deliberately outside the crate. Machine output contains a single record.

pub mod http;
pub use http::{ContractJson, ContractPath, ContractQuery, request_context_middleware};

use axum::{
    extract::{Request, State},
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use std::{
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

pub use crate::error::{ErrorCode, ErrorEnvelope, HttpStatus};
pub use crate::sqlite::{
    SnapshotError, SnapshotLimits, ValidationSnapshot, ValidationSnapshotPool,
    open_validation_snapshot, open_validation_snapshot_with_connection_limits,
};
pub use crate::state_file::INSTANCE_LOCK_FILE;

pub const SERVICE_IDENTITY_HEADER: &str = "x-service";

/// Preserve a public machine error while crossing a product's anyhow boundary.
/// Display intentionally excludes details and any internal diagnostic chain.
#[derive(Debug)]
pub struct CliError(pub ErrorEnvelope);

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.0.code, self.0.message)
    }
}
impl std::error::Error for CliError {}

fn failure(code: &'static str, message: &'static str) -> ErrorEnvelope {
    ErrorEnvelope::with_code(
        ErrorCode::new(code).expect("static public error code"),
        message,
    )
}

/// Preserve public state-file failure semantics without exposing paths, owners,
/// permission bits or the underlying operating-system diagnostic text.
pub fn state_error(error: &crate::state_file::Error) -> ErrorEnvelope {
    use crate::state_file::Error;
    match error {
        Error::LockUnavailable { source, .. } if source.kind() == io::ErrorKind::WouldBlock => {
            failure(
                "state.lock_busy",
                "The data directory is in use by another runtime.",
            )
            .retryable(true)
        }
        Error::StateDirectoryNotAbsolute { .. }
        | Error::InvalidChildName { .. }
        | Error::NotDirectory { .. }
        | Error::NotRegularFile { .. }
        | Error::UnexpectedHardLinkCount { .. } => failure(
            "state.unsafe_path",
            "The state path does not satisfy the private directory and file contract.",
        ),
        Error::UnexpectedOwner { .. } => failure(
            "state.owner_mismatch",
            "The state path is not owned by the current process user.",
        ),
        Error::UnexpectedMode { .. } => failure(
            "state.invalid_permissions",
            "The state path does not have the required private permissions.",
        ),
        Error::FileIdentityChanged { .. } => failure(
            "state.identity_changed",
            "The state path changed identity during this operation.",
        ),
        Error::InspectPath { source, .. }
        | Error::CreateDirectory { source, .. }
        | Error::OpenFile { source, .. }
        | Error::LockUnavailable { source, .. } => state_io_error(source),
    }
}

fn state_io_error(error: &io::Error) -> ErrorEnvelope {
    match error.kind() {
        io::ErrorKind::NotFound => {
            failure("state.not_found", "The required state path does not exist.")
        }
        io::ErrorKind::PermissionDenied => failure(
            "state.permission_denied",
            "The current process cannot access the required state path.",
        ),
        _ => failure(
            "state.io_failed",
            "The state operation failed while accessing the filesystem.",
        ),
    }
}

/// Public SQLite diagnostic failures do not include filenames, SQL text or
/// adapter error chains. Only transient capture failures are retryable.
pub fn snapshot_error(error: &SnapshotError) -> ErrorEnvelope {
    let (code, message, retryable) = match error {
        SnapshotError::InvalidLimits | SnapshotError::ConnectionLimits(_) => (
            "snapshot.invalid_limits",
            "The validation snapshot limits are invalid.",
            false,
        ),
        SnapshotError::UnsafeSource => (
            "snapshot.unsafe_source",
            "The validation source does not satisfy the private file contract.",
            false,
        ),
        SnapshotError::MissingDatabase => (
            "snapshot.not_found",
            "The validation database does not exist.",
            false,
        ),
        SnapshotError::InvalidDatabase | SnapshotError::Sqlx(_) => (
            "snapshot.invalid_database",
            "The validation database cannot satisfy the current database contract.",
            false,
        ),
        SnapshotError::SourceChanged => (
            "snapshot.source_changed",
            "The validation source changed; retry after writes stop.",
            true,
        ),
        SnapshotError::BudgetExceeded => (
            "snapshot.budget_exceeded",
            "The validation snapshot exceeds its byte budget.",
            false,
        ),
        SnapshotError::Timeout => (
            "snapshot.timeout",
            "The validation snapshot exceeded its time budget.",
            true,
        ),
        SnapshotError::Busy => (
            "snapshot.busy",
            "The validation source has an active writer, checkpoint or recovery.",
            true,
        ),
        SnapshotError::LockUnavailable(_) => (
            "snapshot.lock_unavailable",
            "The validation source does not support the required native locking.",
            false,
        ),
        SnapshotError::CaptureTaskFailed => (
            "snapshot.capture_failed",
            "The validation capture task failed.",
            false,
        ),
        SnapshotError::Io(_) => (
            "snapshot.io_failed",
            "The validation snapshot could not access its filesystem input.",
            false,
        ),
    };
    failure(code, message).retryable(retryable)
}

/// Response identity comes from the product that owns this router. Validate
/// the service name before installing the middleware; invalid names fail
/// closed because a status caller requires the header to match exactly.
pub async fn service_identity_middleware(
    State(service): State<String>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&service) {
        response
            .headers_mut()
            .insert(SERVICE_IDENTITY_HEADER, value);
    }
    response
}

/// Print a public error contract; diagnostics and internal error chains must
/// be mapped before calling this boundary. Success exit codes are rejected.
pub fn report_error(error: &ErrorEnvelope, json: bool, exit: u8) -> ExitCode {
    let emitted = write_error(
        error,
        json,
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    );
    if emitted.is_err() {
        let _ = writeln!(
            io::stderr().lock(),
            "Could not write the command error response."
        );
    }
    ExitCode::from(if exit == 0 { 1 } else { exit })
}

fn write_error(
    error: &ErrorEnvelope,
    json: bool,
    output: &mut impl Write,
    diagnostics: &mut impl Write,
) -> io::Result<()> {
    if json {
        serde_json::to_writer(&mut *output, error)?;
        writeln!(output)?;
        output.flush()
    } else {
        writeln!(diagnostics, "{}: {}", error.code, error.message)?;
        diagnostics.flush()
    }
}

pub fn validate_runtime_log_directory(data_dir: &Path) -> Result<(), ErrorEnvelope> {
    crate::fs_safety::PrivateDirectory::open_existing(data_dir).map_err(|_| {
        failure(
            "unsafe_state_directory",
            "The data directory must be private and current.",
        )
    })?;
    crate::fs_safety::PrivateDirectory::open_existing(data_dir.join("logs"))
        .map(|_| ())
        .map_err(|_| {
            failure(
                "unsafe_log_directory",
                "An existing private runtime log directory is required.",
            )
        })
}

#[path = "../../../../test/application/unit/server_cli/tests.rs"]
#[cfg(test)]
mod tests;
