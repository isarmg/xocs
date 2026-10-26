use std::{io, path::PathBuf};

/// Failures while accepting source files and their provenance receipts.
#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("invalid relative source path {path}")]
    RelativePath {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("inspect source path {path}")]
    InspectPath {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("read source {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("enumerate source directory {path}")]
    EnumerateSource {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("resolve source path {path}")]
    ResolvePath {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("snapshot source inputs under {path}")]
    SnapshotSource {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("read source directory entry under {path}")]
    DirectoryEntry {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("inspect source modification time {path}")]
    ModificationTime {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("decode source manifest {path}")]
    ManifestEncoding {
        path: PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },
    #[error("parse source manifest {path}")]
    Manifest {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },
    #[error("parse source receipt {path}")]
    Receipt {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("decode source file inventory {path}")]
    FileInventory {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("source receipt is missing {0}")]
    MissingEvidence(&'static str),
    #[error("vendor source inventory cannot be empty")]
    EmptyVendorInventory,
    #[error("invalid SOURCE digest for {0}")]
    InvalidDigest(String),
    #[error("vendored source differs from SOURCE.json: {0}")]
    VendorChanged(PathBuf),
    #[error("vendor root must contain package directories")]
    VendorEntry,
    #[error("maintained source requires ownership and a local modification record")]
    Ownership,
    #[error("original source evidence differs from its recorded digest: {0}")]
    OriginalEvidenceChanged(String),
    #[error("historical source package identity changed")]
    OriginalIdentityChanged,
    #[error("original license and copyright notices must be preserved")]
    LicenseChanged,
    #[error("embedded Web builder requires ORIGIN.json")]
    MissingBuilderOrigin,
    #[error("builder is outside its repository layout")]
    BuilderLayout,
    #[error("Web builder must be the repository-maintained blog-assets implementation")]
    BuilderIdentity,
    #[error("local Rust dependency escapes repository: {0}")]
    DependencyOutsideRepository(PathBuf),
    #[error("local packages must use the root Cargo.lock: {0}")]
    NestedLockfile(PathBuf),
    #[error("missing workspace dependency: {0}")]
    MissingWorkspaceDependency(String),
    #[error("Rust dependency must be a local path: {name} in {directory}")]
    NonLocalDependency { name: String, directory: PathBuf },
    #[error("external source selector on path dependency: {0}")]
    ExternalSourceSelector(String),
    #[error(
        "builder executable is stale or differs from vendored source ({0}); rebuild src/tools/assets with --locked"
    )]
    StaleBuilder(String),
    #[error("Cargo.lock has no valid package records")]
    LockPackages,
}
