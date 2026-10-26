//! Errors for producer ownership, output selection and transactional publication.
use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error(transparent)]
    Source(#[from] crate::inventory::InventoryError),
    #[error("canonicalize product root {path:?}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("inspect build lock path {path:?}")]
    Inspect {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("create build lock directory {path:?}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("open product asset build lock {path:?}")]
    Open {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("asset build lock must be a physical regular file")]
    InvalidFile,
    #[error("asset build lock identity changed or is hard linked")]
    InvalidIdentity,
    #[error("Another asset build owns {root:?}; retry after it finishes")]
    Busy { root: PathBuf },
    #[error("lock product asset build {path:?}")]
    Acquire {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("asset build lock belongs to another product root")]
    DifferentRoot,
}

#[derive(Debug, thiserror::Error)]
pub enum DistributionError {
    #[error(transparent)]
    Configuration(#[from] crate::assets::AssetError),
    #[error(transparent)]
    Source(#[from] crate::inventory::InventoryError),
    #[error(
        "distribution must be a generated directory separate from authored source and source ancestors"
    )]
    AuthoredDirectory,
    #[error("distribution needs a parent directory and directory name")]
    MissingName,
    #[error("canonicalize distribution product root {path:?}")]
    CanonicalizeRoot {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("distribution path must be UTF-8: {0:?}")]
    NonUnicodeOutput(PathBuf),
    #[error("create distribution parent {path:?}")]
    CreateParent {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("canonicalize distribution parent {path:?}")]
    CanonicalizeParent {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    Source(#[from] crate::inventory::InventoryError),
    #[error("inspect generated destination {path:?}")]
    Inspect {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("generated destination must be a physical directory")]
    InvalidDirectory,
    #[error("refusing to replace a nonempty directory without generated-output provenance: {0:?}")]
    MissingProvenance(PathBuf),
    #[error("runtime inventory needs an absolute path")]
    RelativeReceipt,
    #[error("runtime inventory must be separate from generated output directories")]
    OverlappingReceipt,
    #[error("canonicalize publication path {path:?}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("create staged receipt in {path:?}")]
    StageReceipt {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("write staged frontend receipt")]
    WriteReceipt(#[source] io::Error),
    #[error("sync staged frontend receipt")]
    SyncReceipt(#[source] io::Error),
    #[error("publish frontend receipt")]
    PersistReceipt(#[source] tempfile::PersistError),
    #[error("rename generated output from {from:?} to {to:?}")]
    Rename {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("publication rollback failed after {operation}")]
    Rollback {
        operation: Box<Self>,
        #[source]
        rollback: Box<Self>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Lock(#[from] LockError),
    #[error(transparent)]
    Distribution(#[from] DistributionError),
    #[error(transparent)]
    Publication(#[from] PublishError),
    #[error(transparent)]
    Inventory(#[from] crate::inventory::InventoryError),
    #[error(transparent)]
    Graph(#[from] crate::graph::GraphError),
    #[error(transparent)]
    Assets(#[from] crate::assets::AssetError),
    #[error(transparent)]
    BrowserModule(#[from] crate::BrowserModuleError),
    #[error(transparent)]
    Command(#[from] crate::ProcessError),
    #[error("browser manifest has no package name")]
    MissingPackage,
    #[error("runtime receipt requires distribution inside its configured source root")]
    ExternalDistribution,
    #[error("source changed during {0}; previous outputs were preserved")]
    SourceChanged(BuildPhase),
    #[error("create browser build staging directory")]
    Staging(#[source] io::Error),
    #[error("create build output directory {path:?}")]
    CreateOutput {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("write package source metadata {path:?}")]
    WriteMetadata {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug)]
pub enum BuildPhase {
    BrowserCompilation,
    AssetLinking,
}

impl std::fmt::Display for BuildPhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::BrowserCompilation => "browser compilation",
            Self::AssetLinking => "asset linking",
        })
    }
}
