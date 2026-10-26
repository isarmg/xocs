use crate::{inventory, process};
use std::path::PathBuf;

/// Failures while accepting Cargo metadata and the selected browser graph.
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error(transparent)]
    Inventory(#[from] inventory::InventoryError),
    #[error("query browser Cargo graph")]
    Command(#[source] process::ProcessError),
    #[error("invalid Cargo metadata")]
    Metadata(#[source] serde_json::Error),
    #[error("Cargo runtime tree is not UTF-8")]
    TreeEncoding(#[source] std::string::FromUtf8Error),
    #[error("resolve Cargo source path {path}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("browser manifest has no package name")]
    BrowserPackageName,
    #[error("workspace must contain exactly one browser manifest")]
    BrowserManifest,
    #[error("Cargo metadata repeats identifier {0}")]
    DuplicateId(String),
    #[error("every Rust package must be integrated as a local path: {0}")]
    NonLocalPackage(String),
    #[error("Rust manifest escapes repository")]
    ManifestOutsideRepository,
    #[error("Rust target source escapes repository")]
    TargetOutsideRepository,
    #[error("repository directory is absent from the browser path")]
    RepositoryLayout,
    #[error("Cargo runtime package count exceeds bound")]
    PackageBudget,
    #[error("Cargo runtime tree exceeds line bound")]
    TreeBudget,
    #[error("Cargo runtime tree depth is missing")]
    MissingTreeDepth,
    #[error("invalid Cargo runtime tree depth")]
    TreeDepth(#[source] std::num::ParseIntError),
    #[error("Cargo runtime tree depth changed")]
    InvalidTreeDepth,
    #[error("Cargo runtime tree source is ambiguous: {0}")]
    AmbiguousTreeSource(String),
    #[error("host macro was selected as browser runtime")]
    HostMacroSelected,
    #[error("Cargo runtime tree has another root")]
    RuntimeRoot,
    #[error("Cargo runtime dependency cycle")]
    RuntimeCycle,
    #[error("Cargo runtime parent has no resolved node: {0}")]
    MissingRuntimeNode(String),
    #[error("Cargo runtime tree edge differs from normal metadata")]
    RuntimeEdge,
    #[error("Cargo lock has no package records")]
    LockPackages,
    #[error("selected path package missing from root Cargo.lock: {0}")]
    PackageMissingFromLock(String),
    #[error("Cargo runtime tree is empty")]
    EmptyTree,
    #[error("missing resolved dependency node: {0}")]
    MissingNode(String),
    #[error("resolved package is missing from Cargo metadata: {0}")]
    MissingPackage(String),
    #[error("CSR requires exactly one integrated blog-interface facade")]
    FacadeCount,
    #[error("blog-interface must resolve to the integrated facade source")]
    FacadeSource,
    #[error("CSR requires exactly one integrated {name}; found {actual}")]
    IntegratedCount { name: &'static str, actual: usize },
    #[error("{0} must resolve to the project-maintained CSR source")]
    IntegratedSource(&'static str),
    #[error("{name} only permits the supported csr feature; selected {features:?}")]
    IntegratedFeatures {
        name: &'static str,
        features: Vec<String>,
    },
    #[error("{0} default must be an alias for csr alone")]
    DefaultFeatures(&'static str),
    #[error("integrated runtime/proc-macro target roles changed")]
    TargetRole,
    #[error("server/hydration package unexpectedly selected in CSR graph: {0}")]
    UnsupportedPackage(String),
    #[error("{0} selected a server/hydration feature")]
    ServerFeature(String),
}
