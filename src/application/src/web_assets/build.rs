//! Build-script inventory generation. Input bytes are snapshotted into `OUT_DIR`
//! so the manifest and `include_bytes!` always describe the same resource bytes.

use crate::web_assets::{
    AssetManifest, AssetRecord, DirectoryAssets, MANIFEST_FORMAT, MAX_FILES, MAX_TOTAL_BYTES,
    content_type, safe_relative, sha256,
};
use anyhow::{Context, Result, ensure};
use std::{env, fs, path::Path};

#[derive(serde::Serialize)]
struct BuildIdentity<'a> {
    format: &'a str,
    product: String,
    version: String,
    source_revision: String,
    target: String,
    web_assets_sha256: &'a str,
    source_dirty: Option<bool>,
    source_tree_sha256: Option<String>,
    build_mode: String,
    schema_sha256: &'a str,
    core_source_sha256: &'a str,
}

/// Generate the exact Web inventory and a canonical compiled product identity.
/// The product passes its already computed current schema and local core hashes.
/// Runtime and offline setup both validate this identity before opening state.
pub fn generate(
    root: impl AsRef<Path>,
    schema_sha256: &str,
    core_source_sha256: &str,
) -> Result<()> {
    const FILENAME: &str = "blog-browser-assets.rs";
    for name in [
        "BLOG_BUILD_METADATA_DIR",
        "BLOG_SOURCE_REVISION",
        "BLOG_SOURCE_DIRTY",
        "BLOG_SOURCE_TREE_SHA256",
        "BLOG_BUILD_MODE",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let output = env::var_os("OUT_DIR").context("OUT_DIR is required in a Cargo build script")?;
    let output = Path::new(&output);
    generate_at(root.as_ref(), output, FILENAME)?;
    let manifest = fs::read(output.join(format!("{FILENAME}.manifest.json")))?;
    let digest = sha256(&manifest);
    let product = env::var("CARGO_PKG_NAME").context("Cargo product identity is required")?;
    ensure!(
        safe_relative(&product) && !product.contains('/'),
        "unsafe Cargo product identity"
    );
    let source_revision = env::var("BLOG_SOURCE_REVISION").unwrap_or_else(|_| "unbound".into());
    let destination = env::var_os("BLOG_BUILD_METADATA_DIR");
    let explicit = destination.is_some() || source_revision != "unbound";
    let (source_dirty, source_tree_sha256, build_mode) = if explicit {
        let dirty = match env::var("BLOG_SOURCE_DIRTY").as_deref() {
            Ok("true") => true,
            Ok("false") => false,
            _ => anyhow::bail!("BLOG_SOURCE_DIRTY must identify the actual source state"),
        };
        let tree = env::var("BLOG_SOURCE_TREE_SHA256")
            .context("actual source tree SHA-256 is required")?;
        let mode = env::var("BLOG_BUILD_MODE").context("explicit build mode is required")?;
        (Some(dirty), Some(tree), mode)
    } else {
        ensure!(
            env::var("PROFILE").as_deref() == Ok("debug"),
            "unbound native builds require the debug profile"
        );
        (None, None, "development".into())
    };
    let identity = BuildIdentity {
        format: "blog-build-v1",
        product: product.clone(),
        version: env::var("CARGO_PKG_VERSION").context("Cargo version is required")?,
        source_revision,
        target: env::var("TARGET").context("Cargo target is required")?,
        web_assets_sha256: &digest,
        source_dirty,
        source_tree_sha256,
        build_mode,
        schema_sha256,
        core_source_sha256,
    };
    validate_build_identity(&identity, env::var("PROFILE").as_deref() == Ok("debug"))?;
    println!(
        "cargo:rustc-env=BLOG_COMPILED_SOURCE_DIRTY={}",
        identity
            .source_dirty
            .map(|dirty| if dirty { "true" } else { "false" })
            .unwrap_or("unknown")
    );
    println!(
        "cargo:rustc-env=BLOG_COMPILED_SOURCE_TREE_SHA256={}",
        identity.source_tree_sha256.as_deref().unwrap_or("unbound")
    );
    println!(
        "cargo:rustc-env=BLOG_COMPILED_BUILD_MODE={}",
        identity.build_mode
    );
    let identity = serde_json::to_vec(&identity)?;
    let identity_sha256 = sha256(&identity);
    let generated = output.join(FILENAME);
    use std::io::Write;
    let mut declarations = fs::OpenOptions::new().append(true).open(generated)?;
    writeln!(
        declarations,
        "pub static BUILD_IDENTITY: &[u8] = &{identity:?};"
    )?;
    writeln!(
        declarations,
        "pub static BUILD_IDENTITY_SHA256: &str = {identity_sha256:?};"
    )?;
    fs::write(output.join("blog-build-identity.json"), &identity)?;
    if let Some(destination) = destination {
        let directory = crate::fs_safety::PrivateDirectory::open_existing(&destination)?;
        directory.verify_identity()?;
        fs::write(
            Path::new(&destination).join(format!("{product}-web-assets.json")),
            manifest,
        )?;
        fs::write(
            Path::new(&destination).join(format!("{product}-build-identity.json")),
            identity,
        )?;
        directory.verify_identity()?;
    }
    Ok(())
}

fn validate_build_identity(identity: &BuildIdentity<'_>, debug: bool) -> Result<()> {
    let hash = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
    };
    ensure!(
        hash(identity.schema_sha256) && hash(identity.core_source_sha256),
        "invalid native schema or core source hash"
    );
    ensure!(
        matches!(
            identity.build_mode.as_str(),
            "development" | "candidate" | "release"
        ),
        "invalid build mode"
    );
    if identity.build_mode == "development" {
        ensure!(
            debug && identity.source_revision == "unbound",
            "development identity requires unbound debug source"
        );
        ensure!(
            identity.source_dirty.is_some() == identity.source_tree_sha256.is_some(),
            "source stamp must be complete or explicitly unavailable"
        );
        if let Some(tree) = &identity.source_tree_sha256 {
            ensure!(hash(tree), "invalid source tree hash");
        }
    } else {
        ensure!(
            !debug
                && identity.source_revision.len() == 40
                && identity
                    .source_revision
                    .bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f')),
            "candidate and release require a bound release profile"
        );
        ensure!(
            identity.source_dirty.is_some()
                && identity.source_tree_sha256.as_deref().is_some_and(hash),
            "bound builds require the actual source stamp"
        );
        ensure!(
            identity.build_mode != "release" || identity.source_dirty == Some(false),
            "release requires clean source"
        );
    }
    Ok(())
}

/// Explicit output location for build tools and acceptance tests; no environment mutation.
pub fn generate_at(root: &Path, output: &Path, filename: &str) -> Result<()> {
    ensure!(
        safe_relative(filename) && !filename.contains('/'),
        "unsafe generated source filename"
    );
    let root = if root.is_absolute() {
        root.to_path_buf()
    } else {
        env::current_dir()?.join(root)
    };
    let directory = DirectoryAssets::new(&root)?;
    let paths = collect_paths(&root, true)?;
    ensure!(!paths.is_empty(), "web asset directory is empty");
    fs::create_dir_all(output)?;
    let snapshots = output.join(format!("{filename}.assets"));
    fs::create_dir_all(&snapshots)?;
    let mut files = Vec::with_capacity(paths.len());
    let mut declarations = String::from(
        "// Generated by blog_application::web_assets; do not edit.\npub static ASSETS: &[blog_application::web_assets::EmbeddedAsset] = &[\n",
    );
    let mut total = 0_u64;
    for (index, path) in paths.iter().enumerate() {
        let bytes = directory
            .read(path)
            .with_context(|| format!("read web asset {path}"))?;
        total = total
            .checked_add(bytes.len() as u64)
            .context("web asset byte overflow")?;
        ensure!(
            total <= MAX_TOTAL_BYTES,
            "web asset directory exceeds budget"
        );
        let digest = sha256(&bytes);
        let snapshot = snapshots.join(format!("{index:05}-{digest}"));
        // The source can change after this read; compilation uses this exact snapshot.
        fs::write(&snapshot, &bytes)?;
        let snapshot = fs::canonicalize(snapshot)?;
        let snapshot = snapshot
            .to_str()
            .context("generated asset path must be UTF-8")?;
        let mime = content_type(path);
        declarations.push_str(&format!("    blog_application::web_assets::EmbeddedAsset {{ path: {path:?}, content_type: {mime:?}, bytes: include_bytes!({snapshot:?}), sha256: {digest:?} }},\n"));
        files.push(AssetRecord {
            path: path.clone(),
            content_type: mime.to_owned(),
            size: bytes.len() as u64,
            sha256: digest,
        });
    }
    declarations.push_str("];\n");
    let manifest = serde_json::to_string(&AssetManifest {
        format: MANIFEST_FORMAT.to_owned(),
        files,
    })?;
    let digest = sha256(manifest.as_bytes());
    declarations.push_str(&format!(
        "pub const MANIFEST: &str = {manifest:?};\npub const DIGEST: &str = {digest:?};\n"
    ));
    fs::write(output.join(filename), declarations)?;
    fs::write(output.join(format!("{filename}.manifest.json")), manifest)?;
    Ok(())
}

pub(crate) fn collect_paths(root: &Path, watch: bool) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    let mut visited = 0_usize;
    collect(root, root, &mut paths, &mut visited, watch)?;
    paths.sort();
    Ok(paths)
}

fn collect(
    root: &Path,
    directory: &Path,
    paths: &mut Vec<String>,
    visited: &mut usize,
    watch: bool,
) -> Result<()> {
    if watch {
        println!("cargo:rerun-if-changed={}", directory.display());
    }
    for entry in fs::read_dir(directory)? {
        *visited += 1;
        ensure!(
            *visited <= MAX_FILES,
            "web asset inventory exceeds entry budget"
        );
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)?
            .to_str()
            .context("web asset path must be UTF-8")?
            .replace(std::path::MAIN_SEPARATOR, "/");
        ensure!(
            safe_relative(&relative),
            "unsafe web asset path: {relative}"
        );
        let metadata = fs::symlink_metadata(&path)?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "web asset symlink is forbidden: {relative}"
        );
        if metadata.is_dir() {
            collect(root, &path, paths, visited, watch)?;
        } else {
            ensure!(metadata.is_file(), "web asset must be regular: {relative}");
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                ensure!(
                    metadata.nlink() == 1,
                    "web asset hardlink is forbidden: {relative}"
                );
            }
            if watch {
                println!("cargo:rerun-if-changed={}", path.display());
            }
            paths.push(relative);
        }
    }
    Ok(())
}
