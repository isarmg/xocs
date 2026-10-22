//! Source-bound native server builds without a JavaScript launcher.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

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
    directory: String,
    script: String,
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
    build_mode: String,
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

fn path(root: &Path, relative: &str, allow_root: bool) -> Result<PathBuf> {
    if allow_root && relative == "." {
        return Ok(root.to_owned());
    }
    ensure!(
        crate::inventory::relative(Path::new(relative))? == relative,
        "Build path must be canonical and relative"
    );
    Ok(root.join(relative))
}

fn read(file: &Path, maximum: u64) -> Result<Vec<u8>> {
    crate::inventory::physical(file)?;
    let before = fs::symlink_metadata(file)?;
    ensure!(
        before.is_file() && !before.file_type().is_symlink() && before.len() <= maximum,
        "Build artifact must be a bounded regular file: {}",
        file.display()
    );
    let mut handle = fs::File::open(file)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let opened = handle.metadata()?;
        ensure!(
            opened.ino() == before.ino() && opened.dev() == before.dev(),
            "Build artifact changed while opening"
        );
    }
    let mut bytes = Vec::with_capacity(usize::try_from(before.len())?);
    (&mut handle).take(maximum + 1).read_to_end(&mut bytes)?;
    let after = handle.metadata()?;
    ensure!(
        bytes.len() as u64 == before.len()
            && after.len() == before.len()
            && after.modified()? == before.modified()?,
        "Build artifact changed while reading"
    );
    Ok(bytes)
}

fn private_receipt_directory() -> Result<tempfile::TempDir> {
    let mut builder = tempfile::Builder::new();
    builder.prefix("blog-rust-web-build-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // tempfile 3.27 otherwise inherits the process umask. The foundation
        // deliberately requires an exact private 0700 receipt directory.
        builder.permissions(fs::Permissions::from_mode(0o700));
    }
    Ok(builder.tempdir()?)
}

pub(crate) fn capture(command: &mut Command, maximum: usize) -> Result<Vec<u8>> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .context("Child stdout")?
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        let _ = child.kill();
        let _ = child.wait();
        bail!("Build command exceeded its output budget");
    }
    ensure!(child.wait()?.success(), "Build command failed");
    Ok(bytes)
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(capture(
        Command::new("git").args(args).current_dir(root),
        4 * 1024 * 1024,
    )?)?
    .trim()
    .to_owned())
}

fn verify_assets(inventory: &Inventory, dist: &Path, binary: &[u8]) -> Result<()> {
    ensure!(
        inventory.format == "blog-browser-assets-v1"
            && !inventory.files.is_empty()
            && inventory.files.len() <= 16384,
        "Invalid compiled Web inventory"
    );
    let mut actual: BTreeSet<_> = crate::inventory::files(dist)?.into_keys().collect();
    let mut previous = "";
    for asset in &inventory.files {
        ensure!(
            asset.path.as_str() > previous && sha(&asset.sha256) && !asset.content_type.is_empty(),
            "Invalid compiled asset entry"
        );
        let source = path(dist, &asset.path, false)?;
        let bytes = read(&source, 256 * 1024 * 1024)?;
        ensure!(
            bytes.len() as u64 == asset.size
                && digest(&bytes) == asset.sha256
                && actual.remove(&asset.path),
            "Compiled Web bytes differ: {}",
            asset.path
        );
        ensure!(
            memchr::memmem::find(binary, &bytes).is_some(),
            "Executable omitted compiled Web bytes: {}",
            asset.path
        );
        previous = &asset.path;
    }
    ensure!(
        actual.is_empty(),
        "Compiled executable inventory omitted Web output"
    );
    Ok(())
}

/// Verify an existing receipt without running the candidate executable.
pub fn verify(binary: &Path, dist: &Path, receipt_file: &Path) -> Result<()> {
    let encoded = read(receipt_file, 16 * 1024 * 1024)?;
    let receipt: Receipt = serde_json::from_slice(&encoded)?;
    ensure!(
        serde_json::to_vec(&receipt)? == encoded && receipt.format == "blog-assets-receipt-v1",
        "Receipt is not canonical current-format JSON"
    );
    ensure!(
        sha(&receipt.binary_sha256)
            && sha(&receipt.inventory_sha256)
            && sha(&receipt.identity_sha256),
        "Invalid receipt digest"
    );
    let inventory = serde_json::to_vec(&receipt.inventory)?;
    let identity = serde_json::to_vec(&receipt.identity)?;
    ensure!(
        digest(&inventory) == receipt.inventory_sha256
            && digest(&identity) == receipt.identity_sha256
            && receipt.identity.web_assets_sha256 == receipt.inventory_sha256,
        "Receipt metadata digest mismatch"
    );
    validate_identity(&receipt.identity)?;
    let bytes = read(binary, 1024 * 1024 * 1024)?;
    ensure!(
        digest(&bytes) == receipt.binary_sha256
            && memchr::memmem::find(&bytes, &inventory).is_some()
            && memchr::memmem::find(&bytes, receipt.inventory_sha256.as_bytes()).is_some()
            && memchr::memmem::find(&bytes, &identity).is_some()
            && memchr::memmem::find(&bytes, receipt.identity_sha256.as_bytes()).is_some(),
        "Executable differs from its binary-bound receipt"
    );
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
    ensure!(
        Path::new(&git(root, &["rev-parse", "--show-toplevel"])?).canonicalize()?
            == root.canonicalize()?,
        "Source stamp requires the project's own checkout"
    );
    let current = git(root, &["rev-parse", "HEAD"])?;
    ensure!(revision(&current), "Source HEAD must be a full revision");
    let output = capture(
        Command::new("git")
            .args([
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ])
            .current_dir(root),
        64 * 1024 * 1024,
    )?;
    let mut names = BTreeSet::new();
    for name in output
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let name = std::str::from_utf8(name)?;
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
        if [
            "target/",
            "dist/",
            "web/dist/",
            "web/pkg/",
            "web/node_modules/",
            "web/.previous-",
            "tools/assets/target/",
            "src/browser/target/",
            "test/browser-domain/target/",
            ".local-run/",
        ]
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
            Err(error) => return Err(error.into()),
        };
        ensure!(
            !info.file_type().is_symlink(),
            "Source stamp forbids symbolic links: {name}"
        );
        let bytes = crate::inventory::read(&path)?;
        #[cfg(unix)]
        let executable = {
            use std::os::unix::fs::PermissionsExt;
            fs::metadata(&path)?.permissions().mode() & 0o111 != 0
        };
        #[cfg(not(unix))]
        let executable = false;
        files.insert(
            name,
            serde_json::json!({"sha256":crate::inventory::sha(&bytes),"executable":executable}),
        );
    }
    Ok(SourceStamp {
        source_revision: current,
        source_dirty: !git(root, &["status", "--porcelain=v1", "--untracked-files=all"])?
            .is_empty(),
        source_tree_sha256: crate::inventory::sha(&serde_json::to_vec(&files)?),
    })
}

fn validate_identity(identity: &Identity) -> Result<()> {
    ensure!(
        identity.format == "blog-build-v1"
            && name(&identity.product)
            && identity.target == TARGET
            && sha(&identity.web_assets_sha256)
            && sha(&identity.source_tree_sha256)
            && sha(&identity.schema_sha256)
            && sha(&identity.core_source_sha256)
            && matches!(
                identity.build_mode.as_str(),
                "development" | "candidate" | "release"
            )
            && (identity.build_mode != "release" || !identity.source_dirty),
        "Invalid compiled build identity"
    );
    ensure!(
        identity.source_revision == "unbound" || revision(&identity.source_revision),
        "Invalid source revision"
    );
    ensure!(
        regex::Regex::new(r"^\d+\.\d+\.\d+(?:[-+][a-zA-Z0-9.-]+)?$")?.is_match(&identity.version),
        "Invalid product version"
    );
    Ok(())
}

/// Build frontend and backend, then bind the exact emitted resources to the executable.
pub fn build(
    options: &crate::Options,
    config_path: Option<&str>,
    mode: &str,
    web_only: bool,
    rust_only: bool,
    requested_revision: Option<&str>,
) -> Result<PathBuf> {
    ensure!(
        cfg!(all(
            target_os = "linux",
            target_arch = "x86_64",
            target_env = "gnu"
        )),
        "Native Server builds require Linux x86_64 GNU"
    );
    ensure!(
        matches!(mode, "development" | "candidate" | "release") && !(web_only && rust_only),
        "Invalid native build mode"
    );
    let root = &options.root;
    let config_file = path(root, config_path.unwrap_or("blog-assets.json"), false)?;
    let config: Config = serde_json::from_slice(&read(&config_file, 64 * 1024)?)?;
    ensure!(
        config.format == 1
            && name(&config.rust.package)
            && name(&config.rust.binary)
            && !config.web.script.is_empty(),
        "Invalid Server build declaration"
    );
    let variable = &config.rust.source_revision_env;
    ensure!(
        (variable == "SOURCE_REVISION" || variable.ends_with("_SOURCE_REVISION"))
            && variable
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_'),
        "Invalid source identity environment name"
    );
    let web = path(root, &config.web.directory, true)?;
    let manifest = path(root, &config.rust.manifest, false)?;
    let default_dist = path(&web, &config.web.dist, false)?;
    let selected = crate::Options {
        dist: Some(options.dist.clone().unwrap_or(default_dist)),
        ..options.clone()
    };
    let dist = crate::distribution(&selected)?;
    let stamp = source_stamp(root)?;
    let source = if mode != "development" {
        ensure!(
            requested_revision.is_none_or(|value| value == stamp.source_revision),
            "Requested source differs from the actual HEAD"
        );
        ensure!(
            mode != "release" || !stamp.source_dirty,
            "Release requires clean source"
        );
        stamp.source_revision.clone()
    } else {
        ensure!(
            requested_revision.is_none_or(|value| value == "unbound"),
            "Development source must be unbound"
        );
        "unbound".into()
    };
    if !rust_only {
        crate::build(&selected)?;
    }
    if web_only {
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
        .env("BLOG_BUILD_MODE", mode);
    if mode != "development" {
        command.arg("--release");
    }
    if std::env::var_os("CARGO_BUILD_JOBS").is_none() {
        command.env("CARGO_BUILD_JOBS", "2");
    }
    if std::env::var_os("CARGO_PROFILE_DEV_DEBUG").is_none() {
        command.env("CARGO_PROFILE_DEV_DEBUG", "0");
    }
    let output = capture(&mut command, 64 * 1024 * 1024)?;
    let mut binaries = BTreeSet::new();
    for line in output
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
    {
        let message: serde_json::Value = serde_json::from_slice(line)?;
        if message["reason"] == "compiler-message"
            && let Some(rendered) = message["message"]["rendered"].as_str()
        {
            eprint!("{rendered}");
        }
        if message["reason"] == "compiler-artifact"
            && message["target"]["name"] == config.rust.binary
            && message["target"]["kind"]
                .as_array()
                .is_some_and(|v| v.iter().any(|k| k == "bin"))
            && let Some(file) = message["executable"].as_str()
        {
            ensure!(
                fs::symlink_metadata(file)?.is_file(),
                "Compiler reported a non-regular executable"
            );
            binaries.insert(PathBuf::from(file).canonicalize()?);
        }
    }
    ensure!(
        binaries.len() == 1,
        "Compiler did not report exactly one executable"
    );
    let binary = binaries.pop_first().context("Compiler executable")?;
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
    let inventory: Inventory = serde_json::from_slice(&inventory_bytes)?;
    let identity: Identity = serde_json::from_slice(&identity_bytes)?;
    validate_identity(&identity)?;
    ensure!(
        serde_json::to_vec(&inventory)? == inventory_bytes
            && serde_json::to_vec(&identity)? == identity_bytes,
        "Compiled build metadata is not canonical JSON"
    );
    ensure!(
        identity.product == config.rust.package
            && identity.source_revision == source
            && identity.source_dirty == stamp.source_dirty
            && identity.source_tree_sha256 == stamp.source_tree_sha256
            && identity.build_mode == mode
            && identity.web_assets_sha256 == digest(&inventory_bytes),
        "Compiled identity differs from requested source"
    );
    let binary_bytes = read(&binary, 1024 * 1024 * 1024)?;
    ensure!(
        memchr::memmem::find(&binary_bytes, &inventory_bytes).is_some()
            && memchr::memmem::find(&binary_bytes, identity.web_assets_sha256.as_bytes()).is_some(),
        "Executable omitted its compiled inventory"
    );
    verify_assets(&inventory, &dist, &binary_bytes)?;
    ensure!(
        source_stamp(root)? == stamp,
        "Source changed during native build"
    );
    let receipt = Receipt {
        format: "blog-assets-receipt-v1".into(),
        binary_sha256: digest(&binary_bytes),
        inventory_sha256: digest(&inventory_bytes),
        identity_sha256: digest(&identity_bytes),
        inventory,
        identity,
    };
    let destination = binary.with_file_name(format!(
        "{}.blog-assets.json",
        binary
            .file_name()
            .context("Binary filename")?
            .to_string_lossy()
    ));
    if let Ok(info) = fs::symlink_metadata(&destination) {
        ensure!(
            info.is_file() && !info.file_type().is_symlink(),
            "Unsafe existing receipt"
        );
    }
    let mut temporary =
        tempfile::NamedTempFile::new_in(binary.parent().context("Binary directory")?)?;
    std::io::Write::write_all(&mut temporary, &serde_json::to_vec(&receipt)?)?;
    temporary.as_file().sync_all()?;
    temporary.persist(&destination)?;
    verify(&binary, &dist, &destination)?;
    if let Some(export) = std::env::var_os("BLOG_BUILD_METADATA_DIR") {
        let export = PathBuf::from(export);
        ensure!(
            export.is_absolute(),
            "Metadata export requires an absolute directory"
        );
        fn create_export_directory(directory: &Path) -> Result<()> {
            let ancestor = directory
                .ancestors()
                .find(|path| path.exists())
                .context("Metadata export ancestor")?;
            crate::inventory::physical(ancestor)?;
            fs::create_dir_all(directory)?;
            crate::inventory::physical(directory)
        }
        create_export_directory(&export)?;
        for source in crate::inventory::walk(receipt_directory.path())? {
            let relative = source.strip_prefix(receipt_directory.path())?;
            crate::inventory::relative(relative)?;
            let output = export.join(relative);
            let parent = output.parent().context("Metadata export parent")?;
            create_export_directory(parent)?;
            if output.try_exists()? {
                ensure!(
                    fs::symlink_metadata(&output)?.is_file(),
                    "Unsafe metadata export output"
                );
                crate::inventory::physical(&output)?;
            }
            let bytes = crate::inventory::read(&source)?;
            let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
            std::io::Write::write_all(&mut temporary, &bytes)?;
            temporary.as_file().sync_all()?;
            temporary.persist(&output)?;
        }
    }
    eprintln!("Embedded Web verified; receipt {}", destination.display());
    Ok(binary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn native_receipt_directory_is_private_independently_of_umask() {
        use std::os::unix::fs::PermissionsExt;
        let receipt = private_receipt_directory().unwrap();
        assert_eq!(
            fs::metadata(receipt.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    #[test]
    fn source_stamp_covers_untracked_authored_bytes_and_ignores_generated_outputs() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(temp.path().join(".gitignore"), "/target/\n")?;
        fs::write(temp.path().join("source.rs"), "original source")?;
        fs::create_dir(temp.path().join("target"))?;
        fs::write(
            temp.path().join("target/tracked-output"),
            "tracked generated bytes",
        )?;
        git(temp.path(), &["init", "--quiet"])?;
        git(temp.path(), &["add", "."])?;
        git(temp.path(), &["add", "--force", "target/tracked-output"])?;
        git(
            temp.path(),
            &[
                "-c",
                "user.name=fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "fixture",
            ],
        )?;
        let original = source_stamp(temp.path())?;
        assert!(!original.source_dirty);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("missing-source", temp.path().join("dangling-source"))?;
            assert!(source_stamp(temp.path()).is_err());
            fs::remove_file(temp.path().join("dangling-source"))?;
        }
        fs::create_dir_all(temp.path().join("target"))?;
        fs::write(temp.path().join("target/generated"), "generated output")?;
        fs::create_dir_all(temp.path().join("src/foundation/example-1/src/target"))?;
        fs::write(
            temp.path().join("src/foundation/example-1/Cargo.toml"),
            "[package]\nname='example'\nversion='1.0.0'\n",
        )?;
        fs::write(
            temp.path()
                .join("src/foundation/example-1/src/target/data.rs"),
            "pub const DATA:u8=1;",
        )?;
        let with_source_target = source_stamp(temp.path())?;
        fs::write(
            temp.path()
                .join("src/foundation/example-1/src/target/data.rs"),
            "pub const DATA:u8=2;",
        )?;
        assert_ne!(
            with_source_target.source_tree_sha256,
            source_stamp(temp.path())?.source_tree_sha256
        );
        fs::remove_dir_all(temp.path().join("src/foundation"))?;
        assert_eq!(source_stamp(temp.path())?, original);
        fs::create_dir_all(temp.path().join("test/browser-domain/target/debug"))?;
        fs::write(
            temp.path()
                .join("test/browser-domain/target/debug/generated"),
            "nested generated output",
        )?;
        // The source tree digest excludes generated target components even if
        // a new local workspace has not yet acquired its own ignore rule.
        assert_eq!(
            source_stamp(temp.path())?.source_tree_sha256,
            original.source_tree_sha256
        );
        fs::write(
            temp.path().join("target/tracked-output"),
            "changed tracked output",
        )?;
        assert_eq!(
            source_stamp(temp.path())?.source_tree_sha256,
            original.source_tree_sha256
        );
        fs::write(temp.path().join("untracked.rs"), "actual local change")?;
        let changed = source_stamp(temp.path())?;
        assert!(changed.source_dirty);
        assert_ne!(changed.source_tree_sha256, original.source_tree_sha256);
        assert_eq!(changed.source_revision, original.source_revision);
        fs::write(temp.path().join("untracked.rs"), "another local change")?;
        assert_ne!(
            source_stamp(temp.path())?.source_tree_sha256,
            changed.source_tree_sha256
        );
        Ok(())
    }
    #[test]
    fn exact_bytes_and_unlisted_outputs_are_checked() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("index.html"), b"<p>hello</p>").unwrap();
        let inventory = Inventory {
            format: "blog-browser-assets-v1".into(),
            files: vec![Asset {
                path: "index.html".into(),
                content_type: "text/html".into(),
                size: 12,
                sha256: digest(b"<p>hello</p>"),
            }],
        };
        verify_assets(&inventory, temp.path(), b"prefix<p>hello</p>suffix").unwrap();
        assert!(verify_assets(&inventory, temp.path(), b"another executable").is_err());
        fs::write(temp.path().join("extra.js"), b"unrecorded").unwrap();
        assert!(verify_assets(&inventory, temp.path(), b"<p>hello</p>").is_err());
    }
    #[test]
    fn self_consistent_receipt_cannot_replace_the_identity_embedded_in_binary() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let dist = temp.path().join("dist");
        fs::create_dir(&dist)?;
        fs::write(dist.join("index.html"), "hello")?;
        let inventory = Inventory {
            format: "blog-browser-assets-v1".into(),
            files: vec![Asset {
                path: "index.html".into(),
                content_type: "text/html".into(),
                size: 5,
                sha256: digest(b"hello"),
            }],
        };
        let inventory_bytes = serde_json::to_vec(&inventory)?;
        let identity = Identity {
            format: "blog-build-v1".into(),
            product: "blog".into(),
            version: "1.0.1".into(),
            source_revision: "a".repeat(40),
            target: TARGET.into(),
            web_assets_sha256: digest(&inventory_bytes),
            source_dirty: true,
            source_tree_sha256: "b".repeat(64),
            build_mode: "candidate".into(),
            schema_sha256: "d".repeat(64),
            core_source_sha256: "e".repeat(64),
        };
        let identity_bytes = serde_json::to_vec(&identity)?;
        let binary_bytes = [
            b"hello".as_slice(),
            inventory_bytes.as_slice(),
            digest(&inventory_bytes).as_bytes(),
            identity_bytes.as_slice(),
            digest(&identity_bytes).as_bytes(),
        ]
        .concat();
        let binary = temp.path().join("blog");
        let receipt_path = temp.path().join("blog.blog-assets.json");
        fs::write(&binary, &binary_bytes)?;
        let mut receipt = Receipt {
            format: "blog-assets-receipt-v1".into(),
            binary_sha256: digest(&binary_bytes),
            inventory_sha256: digest(&inventory_bytes),
            identity_sha256: digest(&identity_bytes),
            inventory,
            identity,
        };
        fs::write(&receipt_path, serde_json::to_vec(&receipt)?)?;
        verify(&binary, &dist, &receipt_path)?;
        receipt.identity.source_tree_sha256 = "c".repeat(64);
        receipt.identity_sha256 = digest(&serde_json::to_vec(&receipt.identity)?);
        fs::write(&receipt_path, serde_json::to_vec(&receipt)?)?;
        assert!(verify(&binary, &dist, &receipt_path).is_err());
        Ok(())
    }
    #[test]
    fn receipt_cannot_hide_unknown_fields_or_cross_project_paths() {
        assert!(
            serde_json::from_str::<Inventory>(
                r#"{"format":"blog-browser-assets-v1","files":[],"extra":true}"#
            )
            .is_err()
        );
        assert!(path(Path::new("/tmp/test"), "../other", false).is_err());
        assert!(path(Path::new("/tmp/test"), "/tmp/other", false).is_err());
    }
}
