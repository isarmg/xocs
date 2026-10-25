//! Build-script inventory generation. Input bytes are snapshotted into `OUT_DIR`
//! so the manifest and `include_bytes!` always describe the same resource bytes.

use crate::web_assets::{
    AssetManifest, AssetPath, AssetRecord, DirectoryAssets, Error, MANIFEST_FORMAT, MAX_FILES,
    MAX_TOTAL_BYTES, Result, content_type, safe_relative, sha256, valid_digest,
};
use std::{env, fs, path::Path};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildMode {
    Development,
    Candidate,
    Release,
}

impl BuildMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Candidate => "candidate",
            Self::Release => "release",
        }
    }
}

impl std::str::FromStr for BuildMode {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "development" => Ok(Self::Development),
            "candidate" => Ok(Self::Candidate),
            "release" => Ok(Self::Release),
            _ => Err(Error::BuildMode),
        }
    }
}

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
    build_mode: BuildMode,
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
    let output = env::var_os("OUT_DIR").ok_or(Error::MissingOutputDirectory)?;
    let output = Path::new(&output);
    generate_at(root.as_ref(), output, FILENAME)?;
    let manifest = fs::read(output.join(format!("{FILENAME}.manifest.json")))?;
    let digest = sha256(&manifest);
    let product = environment("CARGO_PKG_NAME")?;
    if !safe_relative(&product) || product.contains('/') {
        return Err(Error::ProductIdentity);
    }
    let source_revision = match env::var("BLOG_SOURCE_REVISION") {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => "unbound".into(),
        Err(source @ env::VarError::NotUnicode(_)) => {
            return Err(Error::Environment {
                name: "BLOG_SOURCE_REVISION",
                source,
            });
        }
    };
    let debug = environment("PROFILE")? == "debug";
    let destination = env::var_os("BLOG_BUILD_METADATA_DIR");
    let explicit = destination.is_some() || source_revision != "unbound";
    let (source_dirty, source_tree_sha256, build_mode) = if explicit {
        let dirty = match environment("BLOG_SOURCE_DIRTY")?.as_str() {
            "true" => true,
            "false" => false,
            _ => return Err(Error::SourceDirty),
        };
        let tree = environment("BLOG_SOURCE_TREE_SHA256")?;
        let mode = environment("BLOG_BUILD_MODE")?.parse()?;
        (Some(dirty), Some(tree), mode)
    } else {
        (None, None, BuildMode::Development)
    };
    let identity = BuildIdentity {
        format: "blog-build-v1",
        product: product.clone(),
        version: environment("CARGO_PKG_VERSION")?,
        source_revision,
        target: environment("TARGET")?,
        web_assets_sha256: &digest,
        source_dirty,
        source_tree_sha256,
        build_mode,
        schema_sha256,
        core_source_sha256,
    };
    validate_build_identity(&identity, debug)?;
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
        identity.build_mode.as_str()
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

fn environment(name: &'static str) -> Result<String> {
    env::var(name).map_err(|source| Error::Environment { name, source })
}

fn validate_build_identity(identity: &BuildIdentity<'_>, debug: bool) -> Result<()> {
    if !valid_digest(identity.schema_sha256) || !valid_digest(identity.core_source_sha256) {
        return Err(Error::NativeHash);
    }
    match identity.build_mode {
        BuildMode::Development => {
            if !debug || identity.source_revision != "unbound" {
                return Err(Error::DevelopmentIdentity);
            }
            if identity.source_dirty.is_some() != identity.source_tree_sha256.is_some() {
                return Err(Error::IncompleteSourceStamp);
            }
            if identity
                .source_tree_sha256
                .as_deref()
                .is_some_and(|tree| !valid_digest(tree))
            {
                return Err(Error::SourceTreeHash);
            }
        }
        BuildMode::Candidate | BuildMode::Release => {
            if debug
                || identity.source_revision.len() != 40
                || !identity
                    .source_revision
                    .bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
            {
                return Err(Error::BoundProfile);
            }
            if identity.source_dirty.is_none()
                || !identity
                    .source_tree_sha256
                    .as_deref()
                    .is_some_and(valid_digest)
            {
                return Err(Error::MissingSourceStamp);
            }
            if identity.build_mode == BuildMode::Release && identity.source_dirty != Some(false) {
                return Err(Error::DirtyRelease);
            }
        }
    }
    Ok(())
}

/// Explicit output location for build tools and acceptance tests; no environment mutation.
pub fn generate_at(root: &Path, output: &Path, filename: &str) -> Result<()> {
    if !safe_relative(filename) || filename.contains('/') {
        return Err(Error::UnsafeFilename);
    }
    let root = if root.is_absolute() {
        root.to_path_buf()
    } else {
        env::current_dir()?.join(root)
    };
    let directory = DirectoryAssets::open(&root)?;
    let paths = collect_paths(&root, true)?;
    if paths.is_empty() {
        return Err(Error::EmptyDirectory);
    }
    fs::create_dir_all(output)?;
    let snapshots = output.join(format!("{filename}.assets"));
    fs::create_dir_all(&snapshots)?;
    let mut files = Vec::with_capacity(paths.len());
    let mut declarations = String::from(
        "// Generated by blog_application::web_assets; do not edit.\npub static ASSETS: &[blog_application::web_assets::EmbeddedAsset] = &[\n",
    );
    let mut total = 0_u64;
    for (index, path) in paths.iter().enumerate() {
        let bytes = directory.read(AssetPath(path))?;
        total += bytes.len() as u64;
        if total > MAX_TOTAL_BYTES {
            return Err(Error::TotalBudget);
        }
        let digest = sha256(&bytes);
        let snapshot = snapshots.join(format!("{index:05}-{digest}"));
        // The source can change after this read; compilation uses this exact snapshot.
        fs::write(&snapshot, &bytes)?;
        let snapshot = fs::canonicalize(snapshot)?;
        let snapshot = snapshot
            .to_str()
            .ok_or_else(|| Error::NonUnicodePath(snapshot.clone()))?;
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
        if *visited > MAX_FILES {
            return Err(Error::EntryBudget);
        }
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .expect("invariant: directory entries are beneath the collected root")
            .to_str()
            .ok_or_else(|| Error::NonUnicodePath(path.clone()))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        AssetPath::parse(&relative)?;
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(Error::UnsafeFile);
        }
        if metadata.is_dir() {
            collect(root, &path, paths, visited, watch)?;
        } else {
            use std::os::unix::fs::MetadataExt;
            if !metadata.is_file() || metadata.nlink() != 1 {
                return Err(Error::UnsafeFile);
            }
            if watch {
                println!("cargo:rerun-if-changed={}", path.display());
            }
            paths.push(relative);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/application/unit/web_assets/build/tests.rs"]
mod tests;
