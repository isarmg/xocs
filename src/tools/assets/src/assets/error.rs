//! Operation-specific failures from web input and output boundaries.
use crate::inventory;
use std::{io, path::PathBuf};
use style_compiler::style_optimizer::error::{
    Error as CssError, MinifyErrorKind, ParserError, PrinterError,
};

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("public_assets_manifest must name a JSON file directly in src/web/")]
    ManifestName,
    #[error("embedded output needs explicit entry points")]
    EmbeddedEntries,
    #[error("base must be a local /path/ URL")]
    BaseUrl,
    #[error("asset budget must be in 1..67108864")]
    BudgetRange,
    #[error("configuration may tighten but cannot relax asset budgets")]
    RelaxedBudget,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("unsupported module/resource URL")]
    Url,
    #[error("only ?url asset imports are supported")]
    Query,
    #[error("unvendored bare import needs an explicit alias")]
    Bare,
    #[error("import path escapes filesystem root")]
    RootEscape,
    #[error("css imports cannot create javascript bindings")]
    CssBindings,
    #[error("?url requires a default import or export {{default as name}}")]
    DefaultAsset,
    #[error("module must be .js or .mjs")]
    ModuleExtension,
}

#[derive(Debug, thiserror::Error)]
pub enum BudgetError {
    #[error("public media {name} exceeds {limit}-byte budget ({size} bytes)")]
    Media {
        name: String,
        limit: usize,
        size: usize,
    },
    #[error("public media exceed {limit}-byte total budget ({total} bytes)")]
    MediaTotal { limit: usize, total: usize },
    #[error("asset {name} exceeds {limit}-byte budget ({size} bytes)")]
    Asset {
        name: String,
        limit: usize,
        size: usize,
    },
    #[error("web bundle exceeds {limit}-byte total budget ({total} bytes)")]
    Bundle { limit: usize, total: usize },
    #[error("expanded CSS exceeds {limit}-byte asset budget")]
    Css { limit: usize },
    #[error("output byte count overflow")]
    CountOverflow,
}

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error(transparent)]
    Inventory(#[from] inventory::InventoryError),
    #[error("read web configuration {path}")]
    Configuration {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },
    #[error(transparent)]
    InvalidConfiguration(#[from] ConfigurationError),
    #[error("validate web input {path}")]
    Input {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("decode UTF-8 source {path}")]
    Utf8 {
        path: PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },
    #[error("canonicalize web source {path}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("inspect web source {path}")]
    Inspect {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("source escapes the web workspace: {path}")]
    SourceEscape {
        path: PathBuf,
        #[source]
        source: std::path::StripPrefixError,
    },
    #[error("source must be a physical file inside its web source root: {0}")]
    SourceFile(PathBuf),
    #[error("resolve {specifier:?} in {parent}")]
    Resolve {
        parent: PathBuf,
        specifier: String,
        #[source]
        source: Box<Self>,
    },
    #[error("invalid import {specifier:?}")]
    Import {
        specifier: String,
        #[source]
        source: ImportError,
    },
    #[error("source maps cannot be published")]
    SourceMap,
    #[error(transparent)]
    Budget(#[from] BudgetError),
    #[error("public asset shadows a generated resource: {0}")]
    PublicCollision(String),
    #[error("conflicting generated output: {0}")]
    GeneratedCollision(String),
    #[error("public asset shadows generated HTML")]
    PublicHtml,
    #[error("parse public assets manifest {path}")]
    PublicManifest {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("unsupported public assets manifest format {0}")]
    PublicManifestFormat(u32),
    #[error("invalid public asset SHA-256: {0}")]
    PublicDigest(String),
    #[error("public assets differ from source inventory: {0}")]
    PublicInventory(String),
    #[error("expected WebAssembly bytes in {0}")]
    Wasm(PathBuf),
    #[error("compress WebAssembly source {path}")]
    Compression {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("embedded CSP requires authored SVG data URLs in {0}")]
    SvgDataUrl(PathBuf),
    #[error("decode embedded SVG URL in {path}")]
    SvgEncoding {
        path: PathBuf,
        #[source]
        source: std::str::Utf8Error,
    },
    #[error("css syntax error in {path}")]
    CssParse {
        path: PathBuf,
        #[source]
        source: Box<CssError<ParserError<'static>>>,
    },
    #[error("css minify error in {path}")]
    CssMinify {
        path: PathBuf,
        #[source]
        source: Box<CssError<MinifyErrorKind>>,
    },
    #[error("css print error in {path}")]
    CssPrint {
        path: PathBuf,
        #[source]
        source: Box<PrinterError>,
    },
    #[error("circular CSS @import: {0}")]
    CssCycle(PathBuf),
    #[error("conditional CSS imports must be explicitly bundled upstream: {0}")]
    ConditionalCss(PathBuf),
    #[error("circular ESM imports need a reviewed prebundled ESM distribution: {0}")]
    ModuleCycle(PathBuf),
    #[error("compile javascript in {path}")]
    JavaScript {
        path: PathBuf,
        #[source]
        source: crate::javascript::JavaScriptError,
    },
    #[error("no independent WASM output was linked")]
    MissingWasm,
    #[error("required license is empty: {0}")]
    EmptyLicense(PathBuf),
    #[error("web source root has no repository ancestor: {0}")]
    RepositoryRoot(PathBuf),
    #[error("parse vendor source record {path}")]
    VendorSource {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("index.html must have exactly one external module entry")]
    HtmlEntry,
    #[error("HTML still contains a development-only resource reference")]
    HtmlDevelopmentReference,
    #[error("HTML is missing head terminator")]
    HtmlHead,
    #[error("create output directory {path}")]
    CreateOutputDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("write output {path}")]
    WriteOutput {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
