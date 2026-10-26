use std::{io, path::PathBuf};

/// Failures while compiling or verifying source-bound native artifacts.
#[derive(Debug, thiserror::Error)]
pub enum NativeBuildError {
    #[error(transparent)]
    Inventory(#[from] crate::inventory::InventoryError),
    #[error(transparent)]
    Frontend(#[from] crate::BuildError),
    #[error(transparent)]
    Lock(#[from] crate::LockError),
    #[error(transparent)]
    Distribution(#[from] crate::DistributionError),
    #[error("inspect build artifact {path:?}")]
    InspectArtifact {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("build artifact must be a bounded regular file: {0:?}")]
    InvalidArtifact(PathBuf),
    #[error("open build artifact {path:?}")]
    OpenArtifact {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("inspect opened build artifact {path:?}")]
    InspectOpenedArtifact {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("read build artifact {path:?}")]
    ReadArtifact {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("read build artifact modification time {path:?}")]
    ArtifactTime {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("build artifact changed while opening or reading: {0:?}")]
    ArtifactChanged(PathBuf),
    #[error("create private native receipt directory under {path:?}")]
    CreateReceiptDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("query local Git identity")]
    GitCommand(#[source] crate::ProcessError),
    #[error("decode local Git identity")]
    GitEncoding(#[source] std::string::FromUtf8Error),
    #[error("list local Git source inputs")]
    GitSourceInputs(#[source] crate::ProcessError),
    #[error("decode Git source filename")]
    GitSourceName(#[source] std::str::Utf8Error),
    #[error("canonicalize native build path {path:?}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid compiled Web inventory")]
    InvalidInventory,
    #[error("invalid compiled asset entry")]
    InvalidAssetEntry,
    #[error("compiled Web bytes differ: {0}")]
    AssetBytes(String),
    #[error("executable omitted compiled Web bytes: {0}")]
    MissingEmbeddedAsset(String),
    #[error("compiled executable inventory omitted Web output")]
    UnlistedAssets,
    #[error("parse native build receipt {path:?}")]
    DecodeReceipt {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("receipt is not canonical current-format JSON")]
    NoncanonicalReceipt,
    #[error("invalid receipt digest")]
    InvalidReceiptDigest,
    #[error("receipt metadata digest mismatch")]
    ReceiptMetadataDigest,
    #[error("executable differs from its binary-bound receipt")]
    BinaryReceiptMismatch,
    #[error("source stamp requires the project's own checkout")]
    SourceCheckout,
    #[error("source HEAD must be a full revision")]
    SourceRevision,
    #[error("inspect source stamp input {path:?}")]
    InspectSource {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("source stamp forbids symbolic links: {0}")]
    SourceSymlink(String),
    #[error("invalid compiled build identity")]
    InvalidIdentity,
    #[error("invalid source revision")]
    InvalidIdentityRevision,
    #[error("invalid product version")]
    InvalidVersion,
    #[error("parse native build declaration {path:?}")]
    DecodeConfig {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid Server build declaration")]
    InvalidConfig,
    #[error("invalid source identity environment name")]
    InvalidRevisionVariable,
    #[error("requested source differs from the actual HEAD")]
    RequestedRevision,
    #[error("release requires clean source")]
    DirtyRelease,
    #[error("development source must be unbound")]
    DevelopmentRevision,
    #[error("build native server")]
    Compilation(#[source] crate::ProcessError),
    #[error("decode native compiler message")]
    CompilerMessage(#[source] serde_json::Error),
    #[error("write native compiler diagnostic")]
    CompilerDiagnostic(#[source] io::Error),
    #[error("compiler reported a non-regular executable")]
    InvalidExecutable,
    #[error("compiler did not report exactly one executable")]
    ExecutableCount,
    #[error("decode compiled Web inventory")]
    DecodeInventory(#[source] serde_json::Error),
    #[error("decode compiled native identity")]
    DecodeIdentity(#[source] serde_json::Error),
    #[error("compiled build metadata is not canonical JSON")]
    NoncanonicalMetadata,
    #[error("compiled identity differs from requested source")]
    CompiledIdentityMismatch,
    #[error("executable omitted its compiled inventory")]
    MissingEmbeddedInventory,
    #[error("source changed during native build")]
    SourceChanged,
    #[error("inspect receipt destination {path:?}")]
    InspectDestination {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("unsafe existing receipt or metadata output: {0:?}")]
    UnsafeDestination(PathBuf),
    #[error("create staged native receipt in {path:?}")]
    StageReceipt {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("write staged native receipt for {path:?}")]
    WriteReceipt {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("sync staged native receipt for {path:?}")]
    SyncReceipt {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("publish native receipt {path:?}")]
    PersistReceipt {
        path: PathBuf,
        #[source]
        source: tempfile::PersistError,
    },
    #[error("open receipt directory for synchronization {path:?}")]
    OpenReceiptDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("sync published receipt directory {path:?}")]
    SyncReceiptDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("metadata export requires an absolute directory")]
    RelativeExport,
    #[error("create metadata export directory {path:?}")]
    CreateExportDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
