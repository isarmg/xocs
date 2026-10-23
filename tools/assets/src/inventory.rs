use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

pub type Files = BTreeMap<String, String>;
pub const MAX_FILE: u64 = 32 * 1024 * 1024;

pub fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn relative(path: &Path) -> Result<String> {
    ensure!(
        !path.as_os_str().is_empty() && !path.is_absolute(),
        "Expected a nonempty relative path"
    );
    ensure!(
        path.components().all(|p| matches!(p, Component::Normal(_))),
        "Uncontrolled relative path: {}",
        path.display()
    );
    Ok(path
        .to_str()
        .context("Path must be UTF-8")?
        .replace('\\', "/"))
}

pub fn physical(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        let info = fs::symlink_metadata(ancestor)
            .with_context(|| format!("Inspect {}", ancestor.display()))?;
        ensure!(
            !info.file_type().is_symlink(),
            "Symlink input/output is forbidden: {}",
            ancestor.display()
        );
    }
    Ok(())
}

pub fn read(path: &Path) -> Result<Vec<u8>> {
    physical(path)?;
    let info = fs::symlink_metadata(path)?;
    ensure!(
        info.is_file() && info.len() <= MAX_FILE,
        "Expected bounded physical file: {}",
        path.display()
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(
            info.nlink() == 1,
            "Hard linked input/output is forbidden: {}",
            path.display()
        );
    }
    let bytes = fs::read(path)?;
    ensure!(
        bytes.len() as u64 <= MAX_FILE,
        "Input grew beyond byte bound"
    );
    Ok(bytes)
}

pub fn walk(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if !root.exists() {
        return Ok(paths);
    }
    physical(root)?;
    let mut entries = fs::read_dir(root)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        if matches!(
            name.to_str(),
            Some(".git" | "__pycache__" | ".pytest_cache")
        ) {
            continue;
        }
        if name == "target" && root.join("Cargo.toml").is_file() {
            continue;
        }
        let path = entry.path();
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "Symlink input/output is forbidden: {}",
            path.display()
        );
        if kind.is_dir() {
            paths.extend(walk(&path)?);
        } else if kind.is_file() {
            paths.push(path);
        } else {
            bail!("Special object is forbidden: {}", path.display());
        }
        ensure!(
            paths.len() <= 16384,
            "Input/output file count exceeds bound"
        );
    }
    Ok(paths)
}

pub fn files(root: &Path) -> Result<Files> {
    walk(root)?
        .into_iter()
        .map(|path| Ok((relative(path.strip_prefix(root)?)?, sha(&read(&path)?))))
        .collect()
}

pub fn validate_vendor(root: &Path) -> Result<()> {
    let provenance = root.join("SOURCE.json");
    let value: Value = serde_json::from_slice(&read(&provenance)?)?;
    let expected: Files = serde_json::from_value(
        value
            .get("files")
            .context("SOURCE.json is missing files")?
            .clone(),
    )?;
    ensure!(
        !expected.is_empty(),
        "Vendor source inventory cannot be empty"
    );
    for (name, digest) in &expected {
        relative(Path::new(name))?;
        ensure!(
            digest.len() == 64 && digest.bytes().all(|v| v.is_ascii_hexdigit()),
            "Invalid SOURCE digest"
        );
    }
    let mut actual = files(root)?;
    actual.remove("SOURCE.json");
    ensure!(
        actual == expected,
        "Vendored source differs from SOURCE.json: {}",
        root.display()
    );
    Ok(())
}

pub fn validate_vendors(root: &Path) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        ensure!(
            path.is_dir(),
            "Vendor root must contain package directories"
        );
        validate_vendor(&path)?;
    }
    Ok(())
}

/// Validate immutable historical evidence while allowing the maintained source
/// to evolve. Current implementation bytes are recorded separately by snapshot.
pub fn maintained_source(repository: &Path, directory: &Path) -> Result<Value> {
    let source: Value = serde_json::from_slice(&read(&directory.join("ORIGIN.json"))?)?;
    ensure!(
        source["format"] == 1
            && source["ownership"] == "repository-maintained"
            && source["changes"]
                .as_array()
                .is_some_and(|changes| !changes.is_empty()),
        "Maintained source requires ownership and a local modification record"
    );
    let upstream = &source["upstream"];
    let mut original = None;
    for kind in ["receipt", "archive"] {
        let name = upstream[kind]
            .as_str()
            .context("Missing original source evidence")?;
        relative(Path::new(name))?;
        let bytes = read(&repository.join(name))?;
        ensure!(
            upstream[format!("{kind}_sha256")].as_str() == Some(sha(&bytes).as_str()),
            "Original source evidence differs from its recorded digest: {name}"
        );
        if kind == "receipt" {
            original = Some(serde_json::from_slice::<Value>(&bytes)?);
        }
    }
    let original = original.context("Original source receipt")?;
    ensure!(
        original["package"] == upstream["package"]
            && original["version"] == upstream["version"]
            && original["files"]
                .as_object()
                .is_some_and(|files| !files.is_empty()),
        "Historical source package identity changed"
    );
    if let Some(license) = original["files"]["LICENSE"].as_str() {
        ensure!(
            sha(&read(&directory.join("LICENSE"))?) == license,
            "Original license and copyright notices must be preserved"
        );
    }
    Ok(source)
}

pub fn builder_source(directory: &Path, required: bool) -> Result<Option<Value>> {
    if !directory.join("ORIGIN.json").exists() {
        ensure!(!required, "Embedded Web builder requires ORIGIN.json");
        return Ok(None);
    }
    let repository = directory
        .parent()
        .and_then(Path::parent)
        .context("Builder repository")?;
    let source = maintained_source(repository, directory)?;
    let manifest: toml::Value =
        toml::from_str(&String::from_utf8(read(&directory.join("Cargo.toml"))?)?)?;
    ensure!(
        source["package"] == "blog-assets"
            && manifest["package"]["name"].as_str() == Some("blog-assets"),
        "Web builder must be the repository-maintained blog-assets implementation"
    );
    Ok(Some(source))
}

/// Reuse source digests within a single snapshot. A fresh cache is created
/// for every snapshot; before/after checks always reread source bytes.
#[derive(Default)]
struct SnapshotFiles {
    hashes: BTreeMap<PathBuf, String>,
}

impl SnapshotFiles {
    fn read(&mut self, path: &Path) -> Result<Vec<u8>> {
        let bytes = read(path)?;
        self.hashes.insert(path.into(), sha(&bytes));
        Ok(bytes)
    }

    fn hash(&mut self, path: &Path) -> Result<String> {
        if let Some(digest) = self.hashes.get(path) {
            return Ok(digest.clone());
        }
        self.read(path)?;
        Ok(self.hashes[path].clone())
    }

    fn files(&mut self, root: &Path) -> Result<Files> {
        walk(root)?
            .into_iter()
            .map(|path| Ok((relative(path.strip_prefix(root)?)?, self.hash(&path)?)))
            .collect()
    }
}

/// Include complete local packages, including build-script data outside src/.
fn cargo_sources(
    repository: &Path,
    base: &Path,
    workspace: &toml::Value,
    seen: &mut BTreeSet<PathBuf>,
    hashes: &mut SnapshotFiles,
    records: &mut BTreeMap<String, Files>,
) -> Result<()> {
    physical(base)?;
    let base = base.canonicalize()?;
    ensure!(
        base.starts_with(repository),
        "Local Rust dependency escapes repository: {}",
        base.display()
    );
    if !seen.insert(base.clone()) {
        return Ok(());
    }
    ensure!(
        !base.join("Cargo.lock").exists(),
        "Local packages must use the root Cargo.lock: {}",
        base.display()
    );
    let manifest: toml::Value =
        toml::from_str(&String::from_utf8(hashes.read(&base.join("Cargo.toml"))?)?)?;
    if base.join("ORIGIN.json").exists() {
        maintained_source(repository, &base)?;
    }
    if base.join("SOURCE.json").exists() {
        validate_vendor(&base)?;
    }
    let input = hashes.files(&base)?;
    records.insert(relative(base.strip_prefix(repository)?)?, input);
    fn dependencies(
        value: &toml::Value,
        base: &Path,
        repository: &Path,
        workspace: &toml::Value,
        found: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let Some(table) = value.as_table() else {
            return Ok(());
        };
        for (name, declaration) in table {
            let (declaration, directory) =
                if declaration.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
                    (
                        workspace
                            .get("workspace")
                            .and_then(|value| value.get("dependencies"))
                            .and_then(|value| value.get(name))
                            .with_context(|| format!("Missing workspace dependency: {name}"))?,
                        repository,
                    )
                } else {
                    (declaration, base)
                };
            let path = declaration
                .get("path")
                .and_then(toml::Value::as_str)
                .with_context(|| {
                    format!(
                        "Rust dependency must be a local path: {name} in {}",
                        base.display()
                    )
                })?;
            ensure!(
                declaration.get("git").is_none() && declaration.get("registry").is_none(),
                "External source selector on path dependency: {name}"
            );
            found.push(directory.join(path));
        }
        Ok(())
    }
    fn sections(
        value: &toml::Value,
        base: &Path,
        repository: &Path,
        workspace: &toml::Value,
        found: &mut Vec<PathBuf>,
    ) -> Result<()> {
        for name in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(value) = value.get(name) {
                dependencies(value, base, repository, workspace, found)?;
            }
        }
        if let Some(targets) = value.get("target").and_then(toml::Value::as_table) {
            for target in targets.values() {
                sections(target, base, repository, workspace, found)?;
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    sections(&manifest, &base, repository, workspace, &mut paths)?;
    for dependency in paths {
        cargo_sources(repository, &dependency, workspace, seen, hashes, records)?;
    }
    Ok(())
}

/// Source moved outside crate directories can still be consumed by include_str!.
/// This tree digest is also embedded when compiling the builder, so an old
/// executable cannot claim newly modified local dependencies as its own inputs.
fn foundation_paths(root: &Path) -> Result<Vec<PathBuf>> {
    fn collect(directory: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
        if !directory.exists() {
            return Ok(());
        }
        physical(directory)?;
        let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if matches!(
                entry.file_name().to_str(),
                Some(".git" | "__pycache__" | ".pytest_cache")
            ) {
                continue;
            }
            if entry.file_name() == "target" && directory.join("Cargo.toml").is_file() {
                continue;
            }
            let path = entry.path();
            let kind = entry.file_type()?;
            ensure!(
                !kind.is_symlink(),
                "Symlink foundation input: {}",
                path.display()
            );
            if kind.is_dir() {
                collect(&path, paths)?;
            } else {
                ensure!(
                    kind.is_file(),
                    "Non-regular foundation input: {}",
                    path.display()
                );
                paths.push(path);
                ensure!(paths.len() <= 131072, "Foundation file count exceeds bound");
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    for name in [
        "src/foundation",
        "docs/dependencies",
        "test/foundation",
        "compliance/licenses/upstream",
    ] {
        collect(&root.join(name), &mut paths)?;
    }
    Ok(paths)
}

pub fn foundation_inputs(root: &Path) -> Result<Files> {
    foundation_inputs_with(root, &mut SnapshotFiles::default())
}

fn foundation_inputs_with(root: &Path, hashes: &mut SnapshotFiles) -> Result<Files> {
    foundation_paths(root)?
        .into_iter()
        .map(|path| Ok((relative(path.strip_prefix(root)?)?, hashes.hash(&path)?)))
        .collect()
}

pub fn foundation_digest(files: &Files) -> String {
    let mut digest = Sha256::new();
    for (name, hash) in files {
        digest.update(name.as_bytes());
        digest.update([0]);
        digest.update(hash.as_bytes());
        digest.update(b"\n");
    }
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn snapshot(root: &Path) -> Result<Value> {
    let web = root.join("web");
    let frontend = root.join("src/browser");
    let mut rust = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut hashes = SnapshotFiles::default();
    let workspace: toml::Value =
        toml::from_str(&String::from_utf8(hashes.read(&root.join("Cargo.toml"))?)?)?;
    cargo_sources(
        root,
        &frontend,
        &workspace,
        &mut seen,
        &mut hashes,
        &mut rust,
    )?;
    let builder = root.join("tools/assets");
    if builder.join("Cargo.toml").exists() {
        cargo_sources(
            root,
            &builder,
            &workspace,
            &mut seen,
            &mut hashes,
            &mut rust,
        )?;
    }
    validate_vendors(&web.join("vendor"))?;
    let mut assets = Files::new();
    for entry in fs::read_dir(&web)? {
        let path = entry?.path();
        if path.is_file() {
            assets.insert(relative(path.strip_prefix(&web)?)?, hashes.hash(&path)?);
        }
    }
    for name in ["src", "runtime", "vendor", "styles", "public", "licenses"] {
        for path in walk(&web.join(name))? {
            assets.insert(relative(path.strip_prefix(&web)?)?, hashes.hash(&path)?);
        }
    }
    if web.join("styles/ORIGIN.json").exists() {
        maintained_source(root, &web.join("styles"))?;
    }
    let origin_inputs = hashes.files(&root.join("compliance/web-origins"))?;
    let builder_source = builder_source(
        &builder,
        root.join("blog-assets.json").is_file() && builder.join("Cargo.toml").is_file(),
    )?;
    let foundation_inputs = foundation_inputs_with(root, &mut hashes)?;
    let mut workspace_inputs = Files::new();
    for name in ["Cargo.toml", "Cargo.lock", ".cargo/config.toml"] {
        let path = root.join(name);
        if path.exists() {
            workspace_inputs.insert(name.into(), hashes.hash(&path)?);
        }
    }
    let mut tool_inputs = Files::new();
    if builder.join("Cargo.toml").exists() {
        tool_inputs.extend(workspace_inputs.clone());
        tool_inputs.insert(
            "src/foundation".into(),
            foundation_digest(&foundation_inputs),
        );
        for name in ["Cargo.toml", "build.rs", "ORIGIN.json"] {
            let path = builder.join(name);
            if path.exists() {
                tool_inputs.insert(relative(path.strip_prefix(root)?)?, hashes.hash(&path)?);
            }
        }
        for path in walk(&builder.join("src"))? {
            tool_inputs.insert(relative(path.strip_prefix(root)?)?, hashes.hash(&path)?);
        }
    }
    if !tool_inputs.is_empty() {
        for (name, digest) in crate::compiled_inputs() {
            ensure!(
                tool_inputs
                    .get(name)
                    .is_some_and(|actual| actual == &digest),
                "Builder executable is stale or differs from vendored source ({name}); rebuild tools/assets with --locked"
            );
        }
    }
    let lock: toml::Value =
        toml::from_str(&String::from_utf8(hashes.read(&root.join("Cargo.lock"))?)?)?;
    let packages = lock
        .get("package")
        .context("Cargo.lock has no package records")?
        .as_array()
        .context("Invalid Cargo lock packages")?;
    let cargo_packages = packages
        .iter()
        .map(|package| {
            let mut record = serde_json::Map::new();
            for name in ["name", "version", "source", "checksum"] {
                if let Some(value) = package.get(name) {
                    record.insert(
                        name.into(),
                        serde_json::to_value(value).expect("TOML scalar"),
                    );
                }
            }
            Value::Object(record)
        })
        .collect::<Vec<_>>();
    let browser_inputs = rust.remove("src/browser").unwrap_or_default();
    let shared_inputs = rust.get("src/interface").cloned().unwrap_or_default();
    Ok(
        json!({"browser_inputs":browser_inputs,"shared_inputs":shared_inputs,"patched_inputs":rust,"asset_inputs":assets,
        "builder_sha256":sha(&serde_json::to_vec(&tool_inputs)?),"builder_inputs":tool_inputs,"builder_source":builder_source,"cargo_packages":cargo_packages,"origin_inputs":origin_inputs,"workspace_inputs":workspace_inputs,"foundation_inputs":foundation_inputs,"lockfile":"Cargo.lock"}),
    )
}

/// Public assets retain the existing 512 KiB budget. Full per-file inputs are
/// checked internally and exported in the private frontend receipt; public
/// provenance binds those exact inventories by SHA and per-package summaries.
pub fn public_provenance(snapshot: &Value, runtime: &Value) -> Result<Value> {
    let mut public = snapshot.clone();
    let object = public.as_object_mut().context("Source snapshot object")?;
    let dependencies = object
        .remove("patched_inputs")
        .context("Local source packages")?;
    let summaries = dependencies.as_object().context("Local package inventories")?.iter().map(|(name, files)| {
        Ok((name.clone(), json!({"sha256":sha(&serde_json::to_vec(files)?),"file_count":files.as_object().context("Package files")?.len()})))
    }).collect::<Result<BTreeMap<_,_>>>()?;
    let foundation = object
        .remove("foundation_inputs")
        .context("Foundation source inventory")?;
    object.insert("source_packages".into(), json!(summaries));
    object.insert(
        "source_snapshot_sha256".into(),
        json!(sha(&serde_json::to_vec(snapshot)?)),
    );
    object.insert("foundation_inventory".into(), json!({"sha256":sha(&serde_json::to_vec(&foundation)?),"file_count":foundation.as_object().context("Foundation inputs")?.len()}));
    let mut runtime = runtime.clone();
    let runtime_object = runtime.as_object_mut().context("Rust runtime object")?;
    let sources = runtime_object
        .remove("source_inputs")
        .context("Runtime source inventory")?;
    runtime_object.insert("source_inputs_summary".into(), json!({"sha256":sha(&serde_json::to_vec(&sources)?),"file_count":sources.as_object().context("Runtime sources")?.len()}));
    object.insert("rust_runtime".into(), runtime);
    Ok(public)
}

/// Cheap watcher probe; the release build still hashes all inputs before and
/// after compilation/linking. Include inode/ctime on Unix to notice replacement
/// even when an editor restores the previous modification timestamp.
pub fn signature(root: &Path) -> Result<String> {
    let web = root.join("web");
    let mut result = String::new();
    let mut paths = Vec::new();
    for entry in fs::read_dir(&web)? {
        let path = entry?.path();
        if path.is_file() {
            paths.push(path);
        }
    }
    for name in ["src", "runtime", "vendor", "styles", "public", "licenses"] {
        paths.extend(walk(&web.join(name))?);
    }
    for name in ["src/browser", "src/interface", "tools/assets"] {
        paths.extend(walk(&root.join(name))?);
    }
    paths.extend(foundation_paths(root)?);
    for name in ["Cargo.toml", "Cargo.lock", ".cargo/config.toml"] {
        let path = root.join(name);
        if path.exists() {
            paths.push(path);
        }
    }
    for path in paths {
        let info = fs::symlink_metadata(&path)?;
        result.push_str(&format!(
            "{}:{}:{:?}",
            path.display(),
            info.len(),
            info.modified()?
        ));
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            result.push_str(&format!(
                ":{}:{}:{}",
                info.ino(),
                info.ctime(),
                info.ctime_nsec()
            ));
        }
        result.push('\n');
    }
    Ok(sha(result.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maintained_sources_allow_edits_but_preserve_original_evidence_and_license() -> Result<()> {
        let root = tempfile::tempdir()?;
        let directory = root.path().join("src/interface");
        fs::create_dir_all(&directory)?;
        fs::create_dir_all(root.path().join("compliance"))?;
        fs::write(directory.join("LICENSE"), "Original copyright")?;
        let original = serde_json::to_vec(&json!({"package":"original-ui","version":"1.0.0",
            "files":{"LICENSE":sha(b"Original copyright")}}))?;
        fs::write(root.path().join("compliance/original.json"), &original)?;
        fs::write(
            root.path().join("compliance/original.tar.gz"),
            b"Original archive",
        )?;
        let origin = json!({"format":1,"ownership":"repository-maintained","package":"blog-interface",
            "changes":[{"description":"Local integration"}],"upstream":{"package":"original-ui","version":"1.0.0",
            "receipt":"compliance/original.json","receipt_sha256":sha(&original),
            "archive":"compliance/original.tar.gz","archive_sha256":sha(b"Original archive")}});
        fs::write(directory.join("ORIGIN.json"), serde_json::to_vec(&origin)?)?;
        maintained_source(root.path(), &directory)?;
        fs::write(
            directory.join("implementation.rs"),
            "Locally maintained implementation",
        )?;
        maintained_source(root.path(), &directory)?;
        fs::write(
            root.path().join("compliance/original.tar.gz"),
            "Changed archive",
        )?;
        assert!(maintained_source(root.path(), &directory).is_err());
        fs::write(
            root.path().join("compliance/original.tar.gz"),
            b"Original archive",
        )?;
        fs::write(directory.join("LICENSE"), "Changed copyright")?;
        assert!(maintained_source(root.path(), &directory).is_err());
        Ok(())
    }

    #[test]
    fn vendor_inventory_rejects_added_and_modified_inputs() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(dir.path().join("LICENSE"), "license")?;
        fs::write(dir.path().join("module.js"), "export const x=1;")?;
        fs::write(
            dir.path().join("SOURCE.json"),
            serde_json::to_vec(&json!({"files":files(dir.path())?}))?,
        )?;
        validate_vendor(dir.path())?;
        fs::write(dir.path().join("module.js"), "export const x=2;")?;
        assert!(validate_vendor(dir.path()).is_err());
        Ok(())
    }
    #[test]
    fn paths_cannot_escape() {
        for name in ["", "/etc/passwd", "../file", "a/../file"] {
            assert!(relative(Path::new(name)).is_err());
        }
    }
    #[cfg(unix)]
    #[test]
    fn links_are_not_build_inputs() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(dir.path().join("real"), "x")?;
        std::os::unix::fs::symlink("real", dir.path().join("link"))?;
        assert!(files(dir.path()).is_err());
        Ok(())
    }

    #[test]
    fn refreshing_a_vendor_manifest_does_not_hide_build_time_source_drift() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let frontend = dir.path().join("src/browser");
        fs::create_dir_all(frontend.join("src"))?;
        fs::write(
            frontend.join("Cargo.toml"),
            "[package]\nname='test-web'\nversion='0.1.0'\nedition='2024'\n",
        )?;
        fs::write(
            dir.path().join("Cargo.lock"),
            "version=4\n[[package]]\nname='test-web'\nversion='0.1.0'\n",
        )?;
        fs::write(frontend.join("src/lib.rs"), "pub fn test() {}")?;
        let vendor = dir.path().join("web/vendor/example");
        fs::create_dir_all(&vendor)?;
        fs::write(vendor.join("LICENSE"), "license")?;
        fs::write(vendor.join("module.js"), "export const x=1;")?;
        fs::write(
            vendor.join("SOURCE.json"),
            serde_json::to_vec(&json!({"files":files(&vendor)?}))?,
        )?;
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers=['src/browser']\nresolver='3'\n",
        )?;
        let before = snapshot(dir.path())?;
        fs::write(vendor.join("module.js"), "export const x=2;")?;
        let mut refreshed = files(&vendor)?;
        refreshed.remove("SOURCE.json");
        fs::write(
            vendor.join("SOURCE.json"),
            serde_json::to_vec(&json!({"files":refreshed}))?,
        )?;
        validate_vendor(&vendor)?;
        assert_ne!(snapshot(dir.path())?, before);
        Ok(())
    }

    #[test]
    fn refreshing_public_media_inventory_does_not_hide_build_time_source_drift() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let web = dir.path().join("web");
        let frontend = dir.path().join("src/browser");
        fs::create_dir_all(frontend.join("src"))?;
        fs::write(
            frontend.join("Cargo.toml"),
            "[package]\nname='test-web'\nversion='0.1.0'\nedition='2024'\n",
        )?;
        fs::write(
            dir.path().join("Cargo.lock"),
            "version=4\n[[package]]\nname='test-web'\nversion='0.1.0'\n",
        )?;
        fs::write(frontend.join("src/lib.rs"), "pub fn test() {}")?;
        fs::create_dir_all(web.join("public"))?;
        fs::write(web.join("public/image.svg"), "<svg/>")?;
        fs::write(
            web.join("rust-web.toml"),
            "[web]\npublic_assets_manifest='public-assets.json'\n",
        )?;
        let write_manifest = || -> Result<()> {
            fs::write(
                web.join("public-assets.json"),
                serde_json::to_vec(&json!({"format":1,"files":files(&web.join("public"))?}))?,
            )?;
            Ok(())
        };
        write_manifest()?;
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers=['src/browser']\nresolver='3'\n",
        )?;
        let before = snapshot(dir.path())?;
        fs::write(web.join("public/image.svg"), "<svg>changed</svg>")?;
        write_manifest()?;
        let after = snapshot(dir.path())?;
        assert_ne!(
            after["asset_inputs"]["public/image.svg"],
            before["asset_inputs"]["public/image.svg"]
        );
        assert_ne!(
            after["asset_inputs"]["public-assets.json"],
            before["asset_inputs"]["public-assets.json"]
        );
        assert_ne!(after, before);
        Ok(())
    }
    #[test]
    fn root_workspace_paths_cover_complete_foundation_and_relocated_inputs() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let root = directory.path();
        fs::create_dir(root.join("web"))?;
        let frontend = root.join("src/browser");
        let dependency = root.join("src/foundation/local-1.0.0");
        fs::create_dir_all(frontend.join("src"))?;
        fs::create_dir_all(dependency.join("src"))?;
        fs::create_dir_all(dependency.join("native"))?;
        fs::create_dir_all(dependency.join("src/target"))?;
        fs::write(dependency.join("src/target/data.rs"), "const DATA:u8=1;")?;
        fs::create_dir_all(root.join("docs/dependencies/origins/local-1.0.0"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers=['src/browser']\nresolver='3'\n[workspace.dependencies]\nlocal={path='src/foundation/local-1.0.0'}\n",
        )?;
        fs::write(
            root.join("Cargo.lock"),
            "version=4\n[[package]]\nname='browser'\nversion='1.0.0'\n[[package]]\nname='local'\nversion='1.0.0'\n",
        )?;
        fs::write(
            frontend.join("Cargo.toml"),
            "[package]\nname='browser'\nversion='1.0.0'\n[dependencies]\nlocal.workspace=true\n",
        )?;
        fs::write(frontend.join("src/lib.rs"), "")?;
        fs::write(
            dependency.join("Cargo.toml"),
            "[package]\nname='local'\nversion='1.0.0'\n",
        )?;
        fs::write(dependency.join("src/lib.rs"), "")?;
        fs::write(dependency.join("native/helper.c"), "int answer=1;")?;
        fs::write(
            root.join("docs/dependencies/origins/local-1.0.0/README.md"),
            "original documentation",
        )?;
        let before = snapshot(root)?;
        assert_eq!(before["lockfile"], "Cargo.lock");
        assert!(
            before["patched_inputs"]["src/foundation/local-1.0.0"]["native/helper.c"].is_string()
        );
        assert!(
            before["foundation_inputs"]["docs/dependencies/origins/local-1.0.0/README.md"]
                .is_string()
        );
        let probe = signature(root)?;
        let helper = dependency.join("native/helper.c");
        let modified = fs::metadata(&helper)?.modified()?;
        fs::write(&helper, "int answer=2;")?;
        fs::File::open(&helper)?.set_times(fs::FileTimes::new().set_modified(modified))?;
        let after = snapshot(root)?;
        // The same file occurs in the Cargo package and complete foundation
        // views. Both must contain the new bytes even when size/mtime match.
        assert_eq!(
            after["patched_inputs"]["src/foundation/local-1.0.0"]["native/helper.c"],
            sha(b"int answer=2;"),
        );
        assert_eq!(
            after["foundation_inputs"]["src/foundation/local-1.0.0/native/helper.c"],
            sha(b"int answer=2;"),
        );
        assert_ne!(after, before);
        assert_ne!(signature(root)?, probe);
        let first = foundation_digest(&foundation_inputs(root)?);
        assert!(
            before["patched_inputs"]["src/foundation/local-1.0.0"]["src/target/data.rs"]
                .is_string()
        );
        fs::write(dependency.join("src/target/data.rs"), "const DATA:u8=2;")?;
        assert_ne!(foundation_digest(&foundation_inputs(root)?), first);
        assert_ne!(snapshot(root)?, before);
        let first = foundation_digest(&foundation_inputs(root)?);
        fs::write(
            root.join("docs/dependencies/origins/local-1.0.0/README.md"),
            "edited documentation",
        )?;
        assert_ne!(foundation_digest(&foundation_inputs(root)?), first);
        fs::create_dir_all(root.join("docs/dependencies/guides/compiler"))?;
        fs::write(
            root.join("docs/dependencies/guides/compiler/ast.md"),
            "included compiler docs",
        )?;
        let first = foundation_digest(&foundation_inputs(root)?);
        fs::write(
            root.join("docs/dependencies/guides/compiler/ast.md"),
            "changed compiler docs",
        )?;
        assert_ne!(foundation_digest(&foundation_inputs(root)?), first);
        fs::create_dir_all(root.join("test/foundation/contract"))?;
        fs::write(
            root.join("test/foundation/contract/probe.rs"),
            "fn probe() {}",
        )?;
        let first = foundation_digest(&foundation_inputs(root)?);
        fs::write(
            root.join("test/foundation/contract/probe.rs"),
            "fn changed() {}",
        )?;
        assert_ne!(foundation_digest(&foundation_inputs(root)?), first);
        fs::write(frontend.join("Cargo.lock"), "version=4")?;
        assert!(snapshot(root).is_err());
        fs::remove_file(frontend.join("Cargo.lock"))?;
        let outside = tempfile::tempdir()?;
        fs::write(
            frontend.join("Cargo.toml"),
            format!(
                "[package]\nname='browser'\nversion='1.0.0'\n[dependencies]\noutside={{path='{}'}}\n",
                outside.path().display()
            ),
        )?;
        assert!(snapshot(root).is_err());
        Ok(())
    }
    #[test]
    fn complete_source_receipts_are_compact_in_public_assets_and_bind_every_file() -> Result<()> {
        let mut packages = serde_json::Map::new();
        let mut all = Files::new();
        for package in 0..400 {
            let files: Files = (0..50)
                .map(|file| (format!("src/module{file}.rs"), "1".repeat(64)))
                .collect();
            for (file, digest) in &files {
                all.insert(
                    format!("src/foundation/pkg{package}-1/{file}"),
                    digest.clone(),
                );
            }
            packages.insert(format!("src/foundation/pkg{package}-1"), json!(files));
        }
        let mut snapshot = json!({"patched_inputs":packages,"foundation_inputs":all});
        let runtime = json!({"source_inputs":all,"graph":{"packages":[]}});
        let public = public_provenance(&snapshot, &runtime)?;
        assert!(serde_json::to_vec(&snapshot)?.len() > 512 * 1024);
        assert!(serde_json::to_vec(&public)?.len() < 512 * 1024);
        assert_eq!(public["source_packages"].as_object().unwrap().len(), 400);
        assert_eq!(public["foundation_inventory"]["file_count"], 20000);
        snapshot["patched_inputs"]["src/foundation/pkg0-1"]["src/module0.rs"] =
            json!("2".repeat(64));
        let changed = public_provenance(&snapshot, &runtime)?;
        assert_ne!(
            public["source_snapshot_sha256"],
            changed["source_snapshot_sha256"]
        );
        assert_ne!(
            public["source_packages"]["src/foundation/pkg0-1"],
            changed["source_packages"]["src/foundation/pkg0-1"]
        );
        Ok(())
    }
}
