#![forbid(unsafe_code)]
pub mod assets;
mod browser_module;
mod error;
pub use browser_module::BrowserModuleError;
pub use error::{BuildError, BuildPhase, DistributionError, LockError, PublishError};
pub use process::ProcessError;
pub mod dev;
pub mod graph;
pub mod inventory;
pub mod javascript;
mod layout;
mod process;
pub mod server_build;
mod source_inputs;

type Result<T> = std::result::Result<T, BuildError>;
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Render diagnostic sources at a CLI or development-request boundary.
pub(crate) fn error_chain(error: &(dyn std::error::Error + 'static)) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(error) = source {
        message.push_str(": ");
        message.push_str(&error.to_string());
        source = error.source();
    }
    message
}

pub const RUST: &str = "1.99.0";
pub const BRIDGE_ABI: &str = "blog-browser-bridge/1";
pub const TARGET: &str = "wasm32-unknown-unknown";

/// Latch CLI cancellation, stop owned commands and close local preview servers.
/// Signal handlers belong to the executable; calling this installs none.
pub fn request_cancellation() {
    process::cancel();
}

/// Bind the conservative compiled implementation basis to this executable.
/// Full source verification snapshots retain the same input categories.
pub fn compiled_inputs() -> Vec<(&'static str, String)> {
    [
        (
            "Cargo.toml",
            include_bytes!("../../../../Cargo.toml").as_slice(),
        ),
        (
            "Cargo.lock",
            include_bytes!("../../../../Cargo.lock").as_slice(),
        ),
    ]
    .into_iter()
    .map(|(name, bytes)| (name, inventory::sha(bytes)))
    .chain(
        include!(concat!(env!("OUT_DIR"), "/builder-inputs.rs"))
            .into_iter()
            .map(|(name, digest)| (name, digest.to_owned())),
    )
    .chain([(
        layout::FOUNDATION,
        include!(concat!(env!("OUT_DIR"), "/foundation-inputs.rs")).to_owned(),
    )])
    .collect()
}

#[derive(Clone, Debug)]
pub struct Options {
    pub root: PathBuf,
    pub offline: bool,
    pub debug: bool,
    pub dist: Option<PathBuf>,
}

/// A stable product lock covers compilation, linking and paired publication.
/// Its file is never renamed, removed or repaired by a competing producer.
pub(crate) struct BuildLock {
    root: PathBuf,
    _file: fs::File,
}

impl BuildLock {
    pub(crate) fn acquire(root: &Path) -> std::result::Result<Self, LockError> {
        inventory::physical(root)?;
        let root = root
            .canonicalize()
            .map_err(|source| LockError::Canonicalize {
                path: root.to_path_buf(),
                source,
            })?;
        let directory = root.join("target/.blog-assets");
        let mut ancestor = directory.as_path();
        loop {
            match fs::symlink_metadata(ancestor) {
                Ok(_) => break,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    ancestor = ancestor
                        .parent()
                        .expect("invariant: an absolute path has an existing filesystem root");
                }
                Err(source) => {
                    return Err(LockError::Inspect {
                        path: ancestor.to_path_buf(),
                        source,
                    });
                }
            }
        }
        inventory::physical(ancestor)?;
        fs::create_dir_all(&directory).map_err(|source| LockError::CreateDirectory {
            path: directory.clone(),
            source,
        })?;
        inventory::physical(&directory)?;
        let path = directory.join("build.lock");
        let mut options = fs::OpenOptions::new();
        use std::os::unix::fs::OpenOptionsExt;
        options
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .custom_flags(
                (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
            );
        let file = options.open(&path).map_err(|source| LockError::Open {
            path: path.clone(),
            source,
        })?;
        inventory::physical(&path)?;
        let opened = file.metadata().map_err(|source| LockError::Inspect {
            path: path.clone(),
            source,
        })?;
        let current = fs::symlink_metadata(&path).map_err(|source| LockError::Inspect {
            path: path.clone(),
            source,
        })?;
        if !opened.is_file() || !current.is_file() {
            return Err(LockError::InvalidFile);
        }
        use std::os::unix::fs::MetadataExt;
        if opened.nlink() != 1
            || current.nlink() != 1
            || opened.dev() != current.dev()
            || opened.ino() != current.ino()
        {
            return Err(LockError::InvalidIdentity);
        }
        match file.try_lock() {
            Ok(()) => Ok(Self { root, _file: file }),
            Err(fs::TryLockError::WouldBlock) => Err(LockError::Busy { root }),
            Err(fs::TryLockError::Error(source)) => Err(LockError::Acquire { path, source }),
        }
    }

    fn validate(&self, root: &Path) -> std::result::Result<(), LockError> {
        let actual = root
            .canonicalize()
            .map_err(|source| LockError::Canonicalize {
                path: root.to_path_buf(),
                source,
            })?;
        if self.root != actual {
            return Err(LockError::DifferentRoot);
        }
        Ok(())
    }
}

fn validate_distribution(root: &Path, output: &Path) -> std::result::Result<(), DistributionError> {
    if root.starts_with(output)
        || output == root.join(layout::WEB)
        || (output.starts_with(root.join("src"))
            && !output.starts_with(root.join(layout::WEB).join("dist")))
        || ["test", "crates", "tools", "compliance", "docs"]
            .iter()
            .any(|directory| output.starts_with(root.join(directory)))
        || output == root.join(layout::WEB).join("pkg")
    {
        return Err(DistributionError::AuthoredDirectory);
    }
    Ok(())
}

pub fn distribution(options: &Options) -> std::result::Result<PathBuf, DistributionError> {
    let output = match &options.dist {
        Some(path) => path.clone(),
        None => PathBuf::from(
            assets::Config::read(&options.root.join(layout::WEB))?
                .web
                .dist,
        ),
    };
    resolve_distribution(&options.root, output)
}

fn resolve_distribution(
    root: &Path,
    output: PathBuf,
) -> std::result::Result<PathBuf, DistributionError> {
    inventory::physical(root)?;
    let normalized_root =
        root.canonicalize()
            .map_err(|source| DistributionError::CanonicalizeRoot {
                path: root.to_owned(),
                source,
            })?;
    let root = normalized_root.as_path();
    let output = if output.is_absolute() {
        output
    } else {
        root.join(output)
    };
    validate_distribution(root, &output)?;
    let parent = output.parent().ok_or(DistributionError::MissingName)?;
    let name = output.file_name().ok_or(DistributionError::MissingName)?;
    fs::create_dir_all(parent).map_err(|source| DistributionError::CreateParent {
        path: parent.to_path_buf(),
        source,
    })?;
    inventory::physical(parent)?;
    let output = parent
        .canonicalize()
        .map_err(|source| DistributionError::CanonicalizeParent {
            path: parent.to_path_buf(),
            source,
        })?
        .join(name);
    validate_distribution(root, &output)?;
    if output.to_str().is_none() {
        return Err(DistributionError::NonUnicodeOutput(output));
    }
    Ok(output)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CargoAction {
    Build,
    Check,
    Test,
}

impl CargoAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Check => "check",
            Self::Test => "test",
        }
    }
}

pub fn cargo_action(options: &Options, action: CargoAction) -> Result<()> {
    graph::resolve(options)?;
    run_cargo_action(options, action, &graph::browser_package(&options.root)?)
}

fn run_cargo_action(options: &Options, action: CargoAction, package: &str) -> Result<()> {
    let mut command = Command::new("cargo");
    command
        .arg(format!("+{RUST}"))
        .arg(action.as_str())
        .args(["--manifest-path"])
        .arg(options.root.join("Cargo.toml"))
        .args(["--package", package, "--locked", "--offline"])
        .current_dir(&options.root);
    if action != CargoAction::Test {
        command.args(["--target", TARGET]);
    }
    if action == CargoAction::Build && !options.debug {
        command.args(["--profile", "web-release"]);
    }
    command.env("CARGO_TARGET_DIR", target_directory(options));
    if env::var_os("CARGO_BUILD_JOBS").is_none() {
        command.env("CARGO_BUILD_JOBS", "2");
    }
    if env::var_os("CARGO_PROFILE_DEV_DEBUG").is_none() {
        command.env("CARGO_PROFILE_DEV_DEBUG", "0");
    }
    if env::var_os("CARGO_PROFILE_TEST_DEBUG").is_none() {
        command.env("CARGO_PROFILE_TEST_DEBUG", "0");
    }
    process::run(
        &mut command,
        process::Policy::build(64 * 1024 * 1024).live_stdout(),
        "Run Cargo browser action",
    )?;
    Ok(())
}

fn target_directory(options: &Options) -> PathBuf {
    let path = env::var_os("BLOG_WASM_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| options.root.join("target/blog-browser"));
    if path.is_absolute() {
        path
    } else {
        options.root.join(path)
    }
}

fn validate_generated(directory: &Path) -> std::result::Result<(), PublishError> {
    let metadata = match fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(PublishError::Inspect {
                path: directory.to_path_buf(),
                source,
            });
        }
    };
    inventory::physical(directory)?;
    if !metadata.is_dir() {
        return Err(PublishError::InvalidDirectory);
    }
    let files = inventory::files(directory)?;
    let first = fs::read_dir(directory)
        .map_err(|source| PublishError::Inspect {
            path: directory.to_path_buf(),
            source,
        })?
        .next()
        .transpose()
        .map_err(|source| PublishError::Inspect {
            path: directory.to_path_buf(),
            source,
        })?;
    if first.is_some()
        && !files.contains_key("blog-assets.json")
        && !files.contains_key("assets/blog-assets.json")
    {
        return Err(PublishError::MissingProvenance(directory.to_path_buf()));
    }
    Ok(())
}

fn receipt_destination(
    path: &Path,
    pkg: &Path,
    dist: &Path,
) -> std::result::Result<PathBuf, PublishError> {
    if !path.is_absolute() {
        return Err(PublishError::RelativeReceipt);
    }
    let parent = path.parent().ok_or(PublishError::RelativeReceipt)?;
    let name = path.file_name().ok_or(PublishError::RelativeReceipt)?;
    inventory::physical(parent)?;
    let path = parent
        .canonicalize()
        .map_err(|source| PublishError::Canonicalize {
            path: parent.to_path_buf(),
            source,
        })?
        .join(name);
    for output in [pkg, dist] {
        let parent = output
            .parent()
            .expect("invariant: generated output has a parent");
        let name = output
            .file_name()
            .expect("invariant: generated output has a filename");
        let output = parent
            .canonicalize()
            .map_err(|source| PublishError::Canonicalize {
                path: parent.to_path_buf(),
                source,
            })?
            .join(name);
        if path.starts_with(output) {
            return Err(PublishError::OverlappingReceipt);
        }
    }
    Ok(path)
}

fn rename_output(from: &Path, to: &Path) -> std::result::Result<(), PublishError> {
    fs::rename(from, to).map_err(|source| PublishError::Rename {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        source,
    })
}

fn restore_outputs(operation: PublishError, moves: &[(&Path, &Path)]) -> PublishError {
    let mut failure = operation;
    for (from, to) in moves {
        if let Err(rollback) = rename_output(from, to) {
            failure = PublishError::Rollback {
                operation: Box::new(failure),
                rollback: Box::new(rollback),
            };
        }
    }
    failure
}

fn backup_output(output: &Path, backup: &Path) -> std::result::Result<bool, PublishError> {
    match fs::rename(output, backup) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(PublishError::Rename {
            from: output.to_path_buf(),
            to: backup.to_path_buf(),
            source,
        }),
    }
}

fn replace_pair(
    _owner: &BuildLock,
    pkg: &Path,
    stage_pkg: &Path,
    dist: &Path,
    stage_dist: &Path,
    backup: &Path,
    receipt: Option<(&Path, &[u8])>,
) -> std::result::Result<(), PublishError> {
    validate_generated(pkg)?;
    validate_generated(dist)?;
    let receipt = receipt
        .map(|(path, bytes)| -> std::result::Result<_, PublishError> {
            use std::io::Write;
            use std::os::unix::fs::PermissionsExt;
            let path = receipt_destination(path, pkg, dist)?;
            let parent = path
                .parent()
                .expect("invariant: normalized absolute receipt has a parent");
            let mut builder = tempfile::Builder::new();
            builder
                .prefix(".blog-frontend-receipt-")
                .permissions(fs::Permissions::from_mode(0o600));
            let mut temporary =
                builder
                    .tempfile_in(parent)
                    .map_err(|source| PublishError::StageReceipt {
                        path: parent.to_path_buf(),
                        source,
                    })?;
            temporary
                .write_all(bytes)
                .map_err(PublishError::WriteReceipt)?;
            temporary
                .as_file()
                .sync_all()
                .map_err(PublishError::SyncReceipt)?;
            Ok((temporary, path))
        })
        .transpose()?;
    let old_pkg = backup.join("previous-pkg");
    let old_dist = backup.join("previous-dist");
    let had_pkg = backup_output(pkg, &old_pkg)?;
    let had_dist = match backup_output(dist, &old_dist) {
        Ok(present) => present,
        Err(error) => {
            return Err(restore_outputs(
                error,
                &if had_pkg {
                    vec![(old_pkg.as_path(), pkg)]
                } else {
                    vec![]
                },
            ));
        }
    };
    if let Err(error) = rename_output(stage_pkg, pkg) {
        let mut restore = Vec::new();
        if had_pkg {
            restore.push((old_pkg.as_path(), pkg));
        }
        if had_dist {
            restore.push((old_dist.as_path(), dist));
        }
        return Err(restore_outputs(error, &restore));
    }
    let mut published_dist = false;
    let publish = (|| -> std::result::Result<(), PublishError> {
        rename_output(stage_dist, dist)?;
        published_dist = true;
        if let Some((temporary, path)) = receipt {
            temporary
                .persist_noclobber(path)
                .map_err(PublishError::PersistReceipt)?;
        }
        Ok(())
    })();
    if let Err(error) = publish {
        let mut restore = Vec::new();
        if published_dist {
            restore.push((dist, stage_dist));
        }
        restore.push((pkg, stage_pkg));
        if had_pkg {
            restore.push((old_pkg.as_path(), pkg));
        }
        if had_dist {
            restore.push((old_dist.as_path(), dist));
        }
        return Err(restore_outputs(error, &restore));
    }
    Ok(())
}

pub fn build(options: &Options) -> Result<Value> {
    let owner = BuildLock::acquire(&options.root)?;
    build_locked(options, &owner)
}

pub(crate) fn build_locked(options: &Options, owner: &BuildLock) -> Result<Value> {
    owner.validate(&options.root)?;
    inventory::physical(&options.root)?;
    let web = options.root.join(layout::WEB);
    let manifest = options.root.join(layout::BROWSER).join("Cargo.toml");
    let declaration = inventory::manifest(&manifest)?;
    let package_name = declaration["package"]["name"]
        .as_str()
        .ok_or(BuildError::MissingPackage)?;
    let package = package_name.replace('-', "_");
    let config = assets::Config::read(&web)?;
    let runtime_root = if config.web.runtime_root == assets::RuntimeRoot::Product {
        options.root.clone()
    } else {
        web.clone()
    };
    let dist = resolve_distribution(
        &options.root,
        options
            .dist
            .clone()
            .unwrap_or_else(|| config.web.dist.clone().into()),
    )?;
    validate_generated(&web.join("pkg"))?;
    validate_generated(&dist)?;
    let inventory_path = env::var_os("BLOG_FRONTEND_METADATA_FILE")
        .map(|path| receipt_destination(Path::new(&path), &web.join("pkg"), &dist))
        .transpose()?;
    if inventory_path.is_some() && !dist.starts_with(&runtime_root) {
        return Err(BuildError::ExternalDistribution);
    }
    let csr_graph = graph::resolve(options)?;
    let before = inventory::snapshot(&options.root)?;
    let mut rust_runtime = graph::runtime(options, &before)?;
    let bindings_tool = json!({
        "package": "blog-assets",
        "execution": "in-process",
        "backend": "browser-module",
        "manifest": format!("{}/Cargo.toml", layout::BUILDER),
        "lockfile": "Cargo.lock",
        "source_snapshot_sha256": inventory::sha(&serde_json::to_vec(&before).expect("invariant: a JSON source snapshot serializes to bytes")),
    });
    run_cargo_action(options, CargoAction::Build, package_name)?;
    if inventory::snapshot(&options.root)? != before {
        return Err(BuildError::SourceChanged(BuildPhase::BrowserCompilation));
    }
    let stage = tempfile::Builder::new()
        .prefix(".rust-web-stage-")
        .tempdir_in(&web)
        .map_err(BuildError::Staging)?;
    let stage_pkg = stage.path().join("pkg");
    let stage_dist = stage.path().join("dist");
    for path in [&stage_pkg, &stage_dist] {
        fs::create_dir(path).map_err(|source| BuildError::CreateOutput {
            path: path.to_path_buf(),
            source,
        })?;
    }
    let profile = if options.debug {
        "debug"
    } else {
        "web-release"
    };
    let binary = target_directory(options)
        .join(TARGET)
        .join(profile)
        .join(format!("{package}.wasm"));
    browser_module::generate(&binary, &stage_pkg, &package)?;
    let generated = inventory::files(&stage_pkg)?;
    let mut metadata = inventory::public_provenance(&before, &rust_runtime);
    let object = metadata
        .as_object_mut()
        .expect("invariant: public provenance is an object");
    let Value::Object(fields) = json!({
        "format": 2,
        "rust": RUST,
        "browser_bridge": BRIDGE_ABI,
        "package": package,
        "profile": profile,
        "files": generated,
        "asset_builder": "blog-assets",
        "csr_graph": csr_graph,
        "bindings_tool": bindings_tool,
    }) else {
        unreachable!("invariant: a JSON object expression produces an object");
    };
    object.extend(fields);
    let mut provenance = serde_json::to_vec(&metadata)
        .expect("invariant: JSON package metadata serializes to bytes");
    provenance.push(b'\n');
    let metadata_path = stage_pkg.join("blog-assets.json");
    fs::write(&metadata_path, &provenance).map_err(|source| BuildError::WriteMetadata {
        path: metadata_path,
        source,
    })?;
    let (outputs, mut runtime) =
        assets::Linker::new(&web, &stage_pkg, config).finish(&provenance)?;
    rust_runtime["outputs"] = json!(
        outputs
            .iter()
            .filter(|(name, _)| name.ends_with(".wasm"))
            .map(|(name, bytes)| json!({"path":name,"sha256":inventory::sha(bytes)}))
            .collect::<Vec<_>>()
    );
    runtime["rust"] = rust_runtime;
    assets::write_outputs(&stage_dist, &outputs)?;
    if inventory::snapshot(&options.root)? != before {
        return Err(BuildError::SourceChanged(BuildPhase::AssetLinking));
    }
    runtime["distribution"] = json!(if let Ok(relative) = dist.strip_prefix(&runtime_root) {
        inventory::relative(relative)?
    } else {
        dist.to_str()
            .expect("invariant: distribution validated UTF-8")
            .to_owned()
    });
    let mut runtime_bytes = serde_json::to_vec_pretty(&runtime)
        .expect("invariant: JSON runtime metadata serializes to bytes");
    runtime_bytes.push(b'\n');
    // Metadata is staged with the output and published as a create-new file only
    // after every source/hash/license/budget gate has passed.
    replace_pair(
        owner,
        &web.join("pkg"),
        &stage_pkg,
        &dist,
        &stage_dist,
        stage.path(),
        inventory_path
            .as_deref()
            .map(|path| (path, runtime_bytes.as_slice())),
    )?;
    let summary = json!({
        "distribution": dist,
        "files": outputs.len(),
        "bytes": outputs.values().map(Vec::len).sum::<usize>(),
        "runtime": runtime,
    });
    println!(
        "Built {} ({} files, {} bytes)",
        dist.display(),
        summary["files"],
        summary["bytes"]
    );
    Ok(summary)
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/lib.rs"]
mod tests;
