//! Source-bound native server builds without a JavaScript launcher.
use crate::{layout, process};
mod error;
pub use error::NativeBuildError;
type Result<T> = std::result::Result<T, NativeBuildError>;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

/// Compilation profile and source binding accepted by the native producer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NativeMode {
    Development,
    Candidate,
    Release,
}

impl NativeMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Candidate => "candidate",
            Self::Release => "release",
        }
    }
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
#[error("native build mode must be development, candidate or release")]
pub struct InvalidNativeMode;

impl std::str::FromStr for NativeMode {
    type Err = InvalidNativeMode;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "development" => Ok(Self::Development),
            "candidate" => Ok(Self::Candidate),
            "release" => Ok(Self::Release),
            _ => Err(InvalidNativeMode),
        }
    }
}

/// Stages selected by a native build command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildSelection {
    All,
    Web,
    Rust,
}

/// Parsed native command options.
#[derive(Clone, Copy, Debug)]
pub struct BuildRequest<'a> {
    pub config_path: Option<&'a str>,
    pub mode: NativeMode,
    pub selection: BuildSelection,
    pub requested_revision: Option<&'a str>,
}

const TARGET: &str = "x86_64-unknown-linux-gnu";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    format: u8,
    web: WebConfig,
    rust: RustConfig,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WebConfig {
    dist: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RustConfig {
    manifest: String,
    package: String,
    binary: String,
    source_revision_env: String,
}

// Member order is part of the existing canonical, binary-bound receipt format.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    path: String,
    content_type: String,
    size: u64,
    sha256: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    format: String,
    files: Vec<Asset>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    format: String,
    product: String,
    version: String,
    source_revision: String,
    target: String,
    web_assets_sha256: String,
    source_dirty: bool,
    source_tree_sha256: String,
    build_mode: NativeMode,
    schema_sha256: String,
    core_source_sha256: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    format: String,
    binary_sha256: String,
    inventory_sha256: String,
    identity_sha256: String,
    inventory: Inventory,
    identity: Identity,
}

#[derive(Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
enum CargoMessage {
    CompilerMessage {
        message: CompilerDiagnostic,
    },
    CompilerArtifact {
        target: CompilerTarget,
        executable: Option<PathBuf>,
    },
    BuildScriptExecuted {},
    BuildFinished {},
}

#[derive(Deserialize)]
struct CompilerDiagnostic {
    rendered: Option<String>,
}

#[derive(Deserialize)]
struct CompilerTarget {
    name: String,
    kind: Vec<String>,
}

fn digest(bytes: &[u8]) -> String {
    crate::inventory::sha(bytes)
}
fn sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
fn revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))
}

fn path(root: &Path, relative: &str) -> Result<PathBuf> {
    crate::inventory::relative(Path::new(relative))?;
    Ok(root.join(relative))
}

fn read(file: &Path, maximum: u64) -> Result<Vec<u8>> {
    crate::inventory::physical(file)?;
    let before =
        fs::symlink_metadata(file).map_err(|source| NativeBuildError::InspectArtifact {
            path: file.to_owned(),
            source,
        })?;
    if !before.is_file() || before.file_type().is_symlink() || before.len() > maximum {
        return Err(NativeBuildError::InvalidArtifact(file.to_owned()));
    }
    let mut handle = fs::File::open(file).map_err(|source| NativeBuildError::OpenArtifact {
        path: file.to_owned(),
        source,
    })?;
    use std::os::unix::fs::MetadataExt;
    let opened = handle
        .metadata()
        .map_err(|source| NativeBuildError::InspectOpenedArtifact {
            path: file.to_owned(),
            source,
        })?;
    if opened.ino() != before.ino() || opened.dev() != before.dev() {
        return Err(NativeBuildError::ArtifactChanged(file.to_owned()));
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    (&mut handle)
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| NativeBuildError::ReadArtifact {
            path: file.to_owned(),
            source,
        })?;
    let after = handle
        .metadata()
        .map_err(|source| NativeBuildError::InspectOpenedArtifact {
            path: file.to_owned(),
            source,
        })?;
    let before_modified = before
        .modified()
        .map_err(|source| NativeBuildError::ArtifactTime {
            path: file.to_owned(),
            source,
        })?;
    let after_modified = after
        .modified()
        .map_err(|source| NativeBuildError::ArtifactTime {
            path: file.to_owned(),
            source,
        })?;
    if bytes.len() as u64 != before.len()
        || after.len() != before.len()
        || after_modified != before_modified
    {
        return Err(NativeBuildError::ArtifactChanged(file.to_owned()));
    }
    Ok(bytes)
}

fn private_receipt_directory() -> Result<tempfile::TempDir> {
    use std::os::unix::fs::PermissionsExt;
    let directory = std::env::temp_dir();
    tempfile::Builder::new()
        .prefix("blog-rust-web-build-")
        .permissions(fs::Permissions::from_mode(0o700))
        .tempdir_in(&directory)
        .map_err(|source| NativeBuildError::CreateReceiptDirectory {
            path: directory,
            source,
        })
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let bytes = process::run(
        Command::new("git").args(args).current_dir(root),
        process::Policy::git(4 * 1024 * 1024),
        "Query local Git identity",
    )
    .map_err(NativeBuildError::GitCommand)?;
    Ok(String::from_utf8(bytes)
        .map_err(NativeBuildError::GitEncoding)?
        .trim()
        .to_owned())
}

fn canonicalize(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .map_err(|source| NativeBuildError::Canonicalize {
            path: path.to_owned(),
            source,
        })
}

fn verify_assets(inventory: &Inventory, dist: &Path, binary: &[u8]) -> Result<()> {
    if inventory.format != "blog-browser-assets-v1"
        || inventory.files.is_empty()
        || inventory.files.len() > 16384
    {
        return Err(NativeBuildError::InvalidInventory);
    }
    let mut actual = crate::inventory::walk(dist)?
        .into_iter()
        .map(|path| {
            crate::inventory::relative(
                path.strip_prefix(dist)
                    .expect("invariant: walked assets remain under their distribution"),
            )
        })
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    let mut previous = "";
    for asset in &inventory.files {
        if asset.path.as_str() <= previous || !sha(&asset.sha256) || asset.content_type.is_empty() {
            return Err(NativeBuildError::InvalidAssetEntry);
        }
        let source = path(dist, &asset.path)?;
        let bytes = read(&source, 256 * 1024 * 1024)?;
        if bytes.len() as u64 != asset.size
            || digest(&bytes) != asset.sha256
            || !actual.remove(&asset.path)
        {
            return Err(NativeBuildError::AssetBytes(asset.path.clone()));
        }
        if memchr::memmem::find(binary, &bytes).is_none() {
            return Err(NativeBuildError::MissingEmbeddedAsset(asset.path.clone()));
        }
        previous = &asset.path;
    }
    if !actual.is_empty() {
        return Err(NativeBuildError::UnlistedAssets);
    }
    Ok(())
}

/// Verify an existing receipt without running the candidate executable.
pub fn verify(binary: &Path, dist: &Path, receipt_file: &Path) -> Result<()> {
    let encoded = read(receipt_file, 16 * 1024 * 1024)?;
    let receipt: Receipt =
        serde_json::from_slice(&encoded).map_err(|source| NativeBuildError::DecodeReceipt {
            path: receipt_file.to_owned(),
            source,
        })?;
    if serde_json::to_vec(&receipt).expect("invariant: receipt fields serialize to JSON") != encoded
        || receipt.format != "blog-assets-receipt-v1"
    {
        return Err(NativeBuildError::NoncanonicalReceipt);
    }
    if !sha(&receipt.binary_sha256)
        || !sha(&receipt.inventory_sha256)
        || !sha(&receipt.identity_sha256)
    {
        return Err(NativeBuildError::InvalidReceiptDigest);
    }
    let inventory = serde_json::to_vec(&receipt.inventory)
        .expect("invariant: receipt fields serialize to JSON");
    let identity =
        serde_json::to_vec(&receipt.identity).expect("invariant: receipt fields serialize to JSON");
    if digest(&inventory) != receipt.inventory_sha256
        || digest(&identity) != receipt.identity_sha256
        || receipt.identity.web_assets_sha256 != receipt.inventory_sha256
    {
        return Err(NativeBuildError::ReceiptMetadataDigest);
    }
    validate_identity(&receipt.identity)?;
    let bytes = read(binary, 1024 * 1024 * 1024)?;
    if !(digest(&bytes) == receipt.binary_sha256
        && memchr::memmem::find(&bytes, &inventory).is_some()
        && memchr::memmem::find(&bytes, receipt.inventory_sha256.as_bytes()).is_some()
        && memchr::memmem::find(&bytes, &identity).is_some()
        && memchr::memmem::find(&bytes, receipt.identity_sha256.as_bytes()).is_some())
    {
        return Err(NativeBuildError::BinaryReceiptMismatch);
    }
    verify_assets(&receipt.inventory, dist, &bytes)
}

/// Digest the actual checked-out source, including untracked authored files.
/// Git's excludes omit build/runtime state; explicit generated output prefixes
/// cannot enter the source stamp even when an old ignore rule was incomplete.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceStamp {
    pub source_revision: String,
    pub source_dirty: bool,
    pub source_tree_sha256: String,
}

pub fn source_stamp(root: &Path) -> Result<SourceStamp> {
    if canonicalize(Path::new(&git(root, &["rev-parse", "--show-toplevel"])?))?
        != canonicalize(root)?
    {
        return Err(NativeBuildError::SourceCheckout);
    }
    let current = git(root, &["rev-parse", "HEAD"])?;
    if !revision(&current) {
        return Err(NativeBuildError::SourceRevision);
    }
    let output = process::run(
        Command::new("git")
            .args([
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ])
            .current_dir(root),
        process::Policy::git(64 * 1024 * 1024),
        "List local Git source inputs",
    )
    .map_err(NativeBuildError::GitSourceInputs)?;
    let generated_prefixes = [
        "target/".to_owned(),
        "dist/".into(),
        format!("{}/dist/", layout::WEB),
        format!("{}/pkg/", layout::WEB),
        format!("{}/node_modules/", layout::WEB),
        format!("{}/.previous-", layout::WEB),
        format!("{}/target/", layout::BUILDER),
        format!("{}/target/", layout::BROWSER),
        "test/browser-domain/target/".into(),
        ".local-run/".into(),
    ];
    let mut names = BTreeSet::new();
    for name in output
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let name = std::str::from_utf8(name).map_err(NativeBuildError::GitSourceName)?;
        crate::inventory::relative(Path::new(name))?;
        let relative = Path::new(name);
        if relative.ancestors().any(|ancestor| {
            ancestor.file_name().is_some_and(|name| name == "target")
                && ancestor
                    .parent()
                    .is_some_and(|parent| root.join(parent).join("Cargo.toml").is_file())
        }) {
            continue;
        }
        if generated_prefixes
            .iter()
            .any(|prefix| name.starts_with(prefix))
        {
            continue;
        }
        names.insert(name.to_owned());
    }
    let mut files = std::collections::BTreeMap::new();
    for name in names {
        let path = root.join(&name);
        let info = match fs::symlink_metadata(&path) {
            Ok(info) => info,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(source) => {
                return Err(NativeBuildError::InspectSource { path, source });
            }
        };
        if info.file_type().is_symlink() {
            return Err(NativeBuildError::SourceSymlink(name));
        }
        let bytes = crate::inventory::read(&path)?;
        use std::os::unix::fs::PermissionsExt;
        let executable = info.permissions().mode() & 0o111 != 0;
        files.insert(
            name,
            serde_json::json!({"sha256":crate::inventory::sha(&bytes),"executable":executable}),
        );
    }
    Ok(SourceStamp {
        source_revision: current,
        source_dirty: !git(root, &["status", "--porcelain=v1", "--untracked-files=all"])?
            .is_empty(),
        source_tree_sha256: crate::inventory::sha(
            &serde_json::to_vec(&files).expect("invariant: source stamp maps serialize to JSON"),
        ),
    })
}

fn validate_identity(identity: &Identity) -> Result<()> {
    if identity.format != "blog-build-v1"
        || !name(&identity.product)
        || identity.target != TARGET
        || !sha(&identity.web_assets_sha256)
        || !sha(&identity.source_tree_sha256)
        || !sha(&identity.schema_sha256)
        || !sha(&identity.core_source_sha256)
    {
        return Err(NativeBuildError::InvalidIdentity);
    }
    match identity.build_mode {
        NativeMode::Development => {
            if identity.source_revision != "unbound" {
                return Err(NativeBuildError::InvalidIdentityRevision);
            }
        }
        NativeMode::Candidate | NativeMode::Release => {
            if !revision(&identity.source_revision) {
                return Err(NativeBuildError::InvalidIdentityRevision);
            }
            if identity.build_mode == NativeMode::Release && identity.source_dirty {
                return Err(NativeBuildError::DirtyRelease);
            }
        }
    }
    let version_pattern = regex::Regex::new(r"^\d+\.\d+\.\d+(?:[-+][a-zA-Z0-9.-]+)?$")
        .expect("invariant: product version regex is valid");
    if !version_pattern.is_match(&identity.version) {
        return Err(NativeBuildError::InvalidVersion);
    }
    Ok(())
}

/// Build frontend and backend, then bind the exact emitted resources to the executable.
pub fn build(options: &crate::Options, request: BuildRequest<'_>) -> Result<PathBuf> {
    let BuildRequest {
        config_path,
        mode,
        selection,
        requested_revision,
    } = request;
    let root = &options.root;
    let owner = crate::BuildLock::acquire(root)?;
    let config_file = path(root, config_path.unwrap_or("blog-assets.json"))?;
    let config: Config =
        serde_json::from_slice(&read(&config_file, 64 * 1024)?).map_err(|source| {
            NativeBuildError::DecodeConfig {
                path: config_file,
                source,
            }
        })?;
    if config.format != 1 || !name(&config.rust.package) || !name(&config.rust.binary) {
        return Err(NativeBuildError::InvalidConfig);
    }
    let variable = &config.rust.source_revision_env;
    if !((variable == "SOURCE_REVISION" || variable.ends_with("_SOURCE_REVISION"))
        && variable
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_'))
    {
        return Err(NativeBuildError::InvalidRevisionVariable);
    }
    let web = root.join(layout::WEB);
    let manifest = path(root, &config.rust.manifest)?;
    let default_dist = path(&web, &config.web.dist)?;
    let selected = crate::Options {
        dist: Some(options.dist.clone().unwrap_or(default_dist)),
        ..options.clone()
    };
    let dist = crate::distribution(&selected)?;
    let stamp = source_stamp(root)?;
    let source = if mode != NativeMode::Development {
        if requested_revision.is_some_and(|value| value != stamp.source_revision) {
            return Err(NativeBuildError::RequestedRevision);
        }
        if mode == NativeMode::Release && stamp.source_dirty {
            return Err(NativeBuildError::DirtyRelease);
        }
        stamp.source_revision.clone()
    } else {
        if requested_revision.is_some_and(|value| value != "unbound") {
            return Err(NativeBuildError::DevelopmentRevision);
        }
        "unbound".into()
    };
    if selection != BuildSelection::Rust {
        crate::build_locked(&selected, &owner)?;
    }
    if selection == BuildSelection::Web {
        return Ok(dist);
    }
    let receipt_directory = private_receipt_directory()?;
    let mut command = Command::new("cargo");
    command
        .arg(format!("+{}", crate::RUST))
        .args(["build", "--locked", "--offline", "--manifest-path"])
        .arg(&manifest)
        .args([
            "--package",
            &config.rust.package,
            "--bin",
            &config.rust.binary,
            "--target",
            TARGET,
            "--message-format=json-render-diagnostics",
        ])
        .current_dir(root)
        .env("BLOG_WEB_DIST", &dist)
        .env(variable, &source)
        .env("BLOG_BUILD_METADATA_DIR", receipt_directory.path())
        .env("BLOG_SOURCE_REVISION", &source)
        .env("BLOG_SOURCE_DIRTY", stamp.source_dirty.to_string())
        .env("BLOG_SOURCE_TREE_SHA256", &stamp.source_tree_sha256)
        .env("BLOG_BUILD_MODE", mode.as_str());
    if mode != NativeMode::Development {
        command.arg("--release");
    }
    if std::env::var_os("CARGO_BUILD_JOBS").is_none() {
        command.env("CARGO_BUILD_JOBS", "2");
    }
    if std::env::var_os("CARGO_PROFILE_DEV_DEBUG").is_none() {
        command.env("CARGO_PROFILE_DEV_DEBUG", "0");
    }
    let output = process::run(
        &mut command,
        process::Policy::build(64 * 1024 * 1024),
        "Build native server",
    )
    .map_err(NativeBuildError::Compilation)?;
    let mut binaries = BTreeSet::new();
    for line in output
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
    {
        let message: CargoMessage =
            serde_json::from_slice(line).map_err(NativeBuildError::CompilerMessage)?;
        match message {
            CargoMessage::CompilerMessage { message } => {
                if let Some(rendered) = message.rendered {
                    write!(io::stderr().lock(), "{rendered}")
                        .map_err(NativeBuildError::CompilerDiagnostic)?;
                }
            }
            CargoMessage::CompilerArtifact { target, executable } => {
                if target.name == config.rust.binary
                    && target.kind.iter().any(|kind| kind == "bin")
                    && let Some(file) = executable
                {
                    if !fs::symlink_metadata(&file)
                        .map_err(|source| NativeBuildError::InspectArtifact {
                            path: file.clone(),
                            source,
                        })?
                        .is_file()
                    {
                        return Err(NativeBuildError::InvalidExecutable);
                    }
                    binaries.insert(canonicalize(&file)?);
                }
            }
            CargoMessage::BuildScriptExecuted {} | CargoMessage::BuildFinished {} => {}
        }
    }

    if binaries.len() != 1 {
        return Err(NativeBuildError::ExecutableCount);
    }
    let binary = binaries
        .pop_first()
        .expect("invariant: compiler reported exactly one executable");
    let inventory_bytes = read(
        &receipt_directory
            .path()
            .join(format!("{}-web-assets.json", config.rust.package)),
        16 * 1024 * 1024,
    )?;
    let identity_bytes = read(
        &receipt_directory
            .path()
            .join(format!("{}-build-identity.json", config.rust.package)),
        16 * 1024 * 1024,
    )?;
    let inventory: Inventory =
        serde_json::from_slice(&inventory_bytes).map_err(NativeBuildError::DecodeInventory)?;
    let identity: Identity =
        serde_json::from_slice(&identity_bytes).map_err(NativeBuildError::DecodeIdentity)?;
    validate_identity(&identity)?;
    if serde_json::to_vec(&inventory).expect("invariant: inventory fields serialize to JSON")
        != inventory_bytes
        || serde_json::to_vec(&identity).expect("invariant: identity fields serialize to JSON")
            != identity_bytes
    {
        return Err(NativeBuildError::NoncanonicalMetadata);
    }
    if !(identity.product == config.rust.package
        && identity.source_revision == source
        && identity.source_dirty == stamp.source_dirty
        && identity.source_tree_sha256 == stamp.source_tree_sha256
        && identity.build_mode == mode
        && identity.web_assets_sha256 == digest(&inventory_bytes))
    {
        return Err(NativeBuildError::CompiledIdentityMismatch);
    }
    let binary_bytes = read(&binary, 1024 * 1024 * 1024)?;
    let identity_sha256 = digest(&identity_bytes);
    if memchr::memmem::find(&binary_bytes, &inventory_bytes).is_none()
        || memchr::memmem::find(&binary_bytes, identity.web_assets_sha256.as_bytes()).is_none()
    {
        return Err(NativeBuildError::MissingEmbeddedInventory);
    }
    if memchr::memmem::find(&binary_bytes, &identity_bytes).is_none()
        || memchr::memmem::find(&binary_bytes, identity_sha256.as_bytes()).is_none()
    {
        return Err(NativeBuildError::BinaryReceiptMismatch);
    }
    verify_assets(&inventory, &dist, &binary_bytes)?;
    if source_stamp(root)? != stamp {
        return Err(NativeBuildError::SourceChanged);
    }
    let receipt = Receipt {
        format: "blog-assets-receipt-v1".into(),
        binary_sha256: digest(&binary_bytes),
        inventory_sha256: digest(&inventory_bytes),
        identity_sha256,
        inventory,
        identity,
    };
    let destination = binary.with_file_name(format!(
        "{}.blog-assets.json",
        binary
            .file_name()
            .expect("invariant: canonical executable has a filename")
            .to_string_lossy()
    ));
    validate_destination(&destination)?;
    publish_bytes(
        &destination,
        &serde_json::to_vec(&receipt).expect("invariant: receipt fields serialize to JSON"),
    )?;
    if let Some(export) = std::env::var_os("BLOG_BUILD_METADATA_DIR") {
        let export = PathBuf::from(export);
        if !export.is_absolute() {
            return Err(NativeBuildError::RelativeExport);
        }
        create_export_directory(&export)?;
        for source in crate::inventory::walk(receipt_directory.path())? {
            let relative = source
                .strip_prefix(receipt_directory.path())
                .expect("invariant: receipt traversal stays within its temporary directory");
            let output = export.join(relative);
            let parent = output
                .parent()
                .expect("invariant: metadata export remains beneath its directory");
            create_export_directory(parent)?;
            validate_destination(&output)?;
            publish_bytes(&output, &crate::inventory::read(&source)?)?;
        }
    }
    Ok(binary)
}

fn validate_destination(destination: &Path) -> Result<()> {
    match fs::symlink_metadata(destination) {
        Ok(info) => {
            if !info.is_file() || info.file_type().is_symlink() {
                return Err(NativeBuildError::UnsafeDestination(destination.to_owned()));
            }
            crate::inventory::physical(destination)?;
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(NativeBuildError::InspectDestination {
                path: destination.to_owned(),
                source,
            });
        }
    }
    Ok(())
}

fn publish_bytes(destination: &Path, bytes: &[u8]) -> Result<()> {
    let parent = destination
        .parent()
        .expect("invariant: native output has a parent directory");
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|source| {
        NativeBuildError::StageReceipt {
            path: parent.to_owned(),
            source,
        }
    })?;
    temporary
        .write_all(bytes)
        .map_err(|source| NativeBuildError::WriteReceipt {
            path: destination.to_owned(),
            source,
        })?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|source| NativeBuildError::SyncReceipt {
            path: destination.to_owned(),
            source,
        })?;
    temporary
        .persist(destination)
        .map_err(|source| NativeBuildError::PersistReceipt {
            path: destination.to_owned(),
            source,
        })?;
    fs::File::open(parent)
        .map_err(|source| NativeBuildError::OpenReceiptDirectory {
            path: parent.to_owned(),
            source,
        })?
        .sync_all()
        .map_err(|source| NativeBuildError::SyncReceiptDirectory {
            path: parent.to_owned(),
            source,
        })?;
    Ok(())
}

fn create_export_directory(directory: &Path) -> Result<()> {
    let mut ancestor = directory;
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => break,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                ancestor = ancestor
                    .parent()
                    .expect("invariant: absolute export path has a filesystem root");
            }
            Err(source) => {
                return Err(NativeBuildError::InspectDestination {
                    path: ancestor.to_owned(),
                    source,
                });
            }
        }
    }
    crate::inventory::physical(ancestor)?;
    fs::create_dir_all(directory).map_err(|source| NativeBuildError::CreateExportDirectory {
        path: directory.to_owned(),
        source,
    })?;
    crate::inventory::physical(directory)?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/server_build.rs"]
mod tests;
