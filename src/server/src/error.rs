//! Typed failures propagated to the executable or HTTP response boundary.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Assets(#[from] crate::static_assets::AssetError),
    #[error(transparent)]
    Config(#[from] blog_application::config::ConfigError),
    #[error(transparent)]
    State(#[from] blog_application::state_file::Error),
    #[error(transparent)]
    Filesystem(#[from] blog_application::fs_safety::Error),
    #[error(transparent)]
    Snapshot(#[from] blog_application::sqlite::SnapshotError),
    #[error(transparent)]
    Sqlite(#[from] blog_application::sqlite::Error),
    #[error("schema validation failed and rollback also failed: {rollback}")]
    SchemaValidationRollback {
        #[source]
        validation: Box<blog_application::sqlite::Error>,
        rollback: sqlx::Error,
    },
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Administrator(#[from] blog_application::admin_core::Error),
    #[error(transparent)]
    AdministratorStore(#[from] blog_application::admin_sqlite::Error),
    #[error(transparent)]
    Authentication(#[from] blog_application::admin_auth::Error),
    #[error(transparent)]
    Runtime(#[from] blog_application::server_runtime::Error),
    #[error(transparent)]
    Log(#[from] blog_application::log::LogError),
    #[error("initialize tracing collector")]
    Tracing(#[from] tracing::collector::util::TryInitError),
    #[error("I/O operation failed")]
    Io(#[from] std::io::Error),
    #[error("current timestamp exceeds its numeric range")]
    TimestampRange(#[from] std::num::TryFromIntError),
    #[error("system clock precedes Unix epoch")]
    Clock(#[from] std::time::SystemTimeError),
    #[error("{variable} must be valid Unicode")]
    Environment {
        variable: &'static str,
        #[source]
        source: std::env::VarError,
    },
    #[error("ROOT must be an absolute normalized directory")]
    InvalidRoot,
    #[error("Web entry must be a regular file")]
    InvalidWebEntry,
    #[error("database identity changed while opening")]
    DatabaseIdentityChanged,
    #[error("initial database checkpoint did not complete")]
    InitialCheckpointBusy,
    #[error("initial database could not leave staging WAL mode")]
    InitialWalMode,
    #[error("retain completed initialization staging file")]
    RetainStaging(#[from] tempfile::PersistError),
    #[error("publish initialized database without replacing an existing file")]
    PublishDatabase {
        staging: std::path::PathBuf,
        #[source]
        source: blog_application::fs_safety::Error,
    },
    #[error("database contains foreign-key violations")]
    ForeignKeyViolations,
    #[error("an author must have at most one family record")]
    DuplicateFamily,
    #[error("database has no active site author")]
    MissingAuthor,
    #[error("explicit initialization is required")]
    NotInitialized,
    #[error("database integrity check failed")]
    IntegrityCheckFailed,
    #[error("administrator not found")]
    AdministratorNotFound,
    #[error(transparent)]
    MediaRecovery(#[from] crate::app::AppError),
}
