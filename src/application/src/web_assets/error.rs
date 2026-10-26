use std::{env, fmt, io, path::PathBuf};

#[derive(thiserror::Error)]
pub enum Error {
    #[error("web asset I/O failed")]
    Io(#[from] io::Error),
    #[error("web asset descriptor operation failed")]
    Descriptor(#[from] rustix::io::Errno),
    #[error("web asset metadata JSON failed")]
    Json(#[from] serde_json::Error),
    #[error("private build metadata directory failed")]
    PrivateDirectory(#[from] crate::fs_safety::Error),
    #[error("build environment variable {name} is unavailable or not Unicode")]
    Environment {
        name: &'static str,
        #[source]
        source: env::VarError,
    },
    #[error("OUT_DIR is required in a Cargo build script")]
    MissingOutputDirectory,
    #[error("web manifest exceeds budget")]
    ManifestBudget,
    #[error("unsupported web manifest format")]
    ManifestFormat,
    #[error("invalid web inventory size")]
    InventorySize,
    #[error("web inventory is not strictly sorted")]
    InventoryOrder,
    #[error("web asset content type differs")]
    ContentType,
    #[error("invalid web asset digest")]
    AssetDigest,
    #[error("web asset exceeds budget")]
    AssetBudget,
    #[error("web asset inventory exceeds byte budget")]
    TotalBudget,
    #[error("web manifest is not canonical JSON")]
    ManifestCanonical,
    #[error("web manifest digest differs")]
    ManifestDigest,
    #[error("embedded web inventory length differs")]
    EmbeddedLength,
    #[error("embedded web metadata differs")]
    EmbeddedMetadata,
    #[error("embedded web bytes differ")]
    EmbeddedBytes,
    #[error("web directory must be absolute")]
    RelativeDirectory,
    #[error("unsafe web directory")]
    UnsafeDirectory,
    #[error("unsafe web asset path")]
    UnsafePath,
    #[error("web asset is not a regular file with a single link")]
    UnsafeFile,
    #[error("web asset path must be UTF-8: {0:?}")]
    NonUnicodePath(PathBuf),
    #[error("unsafe generated source filename")]
    UnsafeFilename,
    #[error("web asset directory is empty")]
    EmptyDirectory,
    #[error("web asset inventory exceeds entry budget")]
    EntryBudget,
    #[error("unsafe Cargo product identity")]
    ProductIdentity,
    #[error("BLOG_SOURCE_DIRTY must identify the actual source state")]
    SourceDirty,
    #[error("invalid native schema or core source hash")]
    NativeHash,
    #[error("invalid build mode")]
    BuildMode,
    #[error("development identity requires unbound debug source")]
    DevelopmentIdentity,
    #[error("source stamp must be complete or explicitly unavailable")]
    IncompleteSourceStamp,
    #[error("invalid source tree hash")]
    SourceTreeHash,
    #[error("candidate and release require a bound release profile")]
    BoundProfile,
    #[error("bound builds require the actual source stamp")]
    MissingSourceStamp,
    #[error("release requires clean source")]
    DirtyRelease,
}

impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Self::Environment { name, .. } = self {
            return formatter
                .debug_struct("Environment")
                .field("name", name)
                .finish_non_exhaustive();
        }
        let mut diagnostic = formatter.debug_tuple("WebAssetError");
        diagnostic.field(&format_args!("{self}"));
        if let Some(source) = std::error::Error::source(self) {
            diagnostic.field(&source);
        }
        diagnostic.finish()
    }
}

#[cfg(test)]
#[path = "../../../../test/application/unit/web_assets/error.rs"]
mod tests;
