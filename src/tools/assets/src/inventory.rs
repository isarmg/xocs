use crate::{
    layout,
    source_inputs::{self, SnapshotFiles},
};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub type Files = BTreeMap<String, String>;
pub const MAX_FILE: u64 = source_inputs::MAX_FILE;

pub fn sha(bytes: &[u8]) -> String {
    source_inputs::sha(bytes)
}

pub fn relative(path: &Path) -> Result<String> {
    Ok(source_inputs::relative(path)?)
}

pub fn physical(path: &Path) -> Result<()> {
    Ok(source_inputs::physical(path)?)
}

pub fn read(path: &Path) -> Result<Vec<u8>> {
    Ok(source_inputs::read(path)?)
}

pub fn walk(root: &Path) -> Result<Vec<PathBuf>> {
    Ok(source_inputs::walk(root, source_inputs::MAX_PACKAGE_FILES)?)
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
        .ancestors()
        .nth(Path::new(layout::BUILDER).components().count())
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

/// Full source verification includes documentation, legal records and test
/// inputs outside package directories. This is deliberately the same broad
/// coverage used by the compiled implementation basis; it is not a list of
/// Cargo compilation units.
pub fn foundation_inputs(root: &Path) -> Result<Files> {
    foundation_inputs_with(root, &mut SnapshotFiles::default())
}

fn foundation_inputs_with(root: &Path, hashes: &mut SnapshotFiles) -> Result<Files> {
    Ok(hashes.verification_inputs(root)?)
}

pub fn foundation_digest(files: &Files) -> String {
    source_inputs::digest(files)
}

pub fn snapshot(root: &Path) -> Result<Value> {
    let web = root.join(layout::WEB);
    let frontend = root.join(layout::BROWSER);
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
    let builder = root.join(layout::BUILDER);
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
    for path in source_inputs::web_paths(&web, &web)? {
        assets.insert(relative(path.strip_prefix(&web)?)?, hashes.hash(&path)?);
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
            layout::FOUNDATION.into(),
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
                "Builder executable is stale or differs from vendored source ({name}); rebuild src/tools/assets with --locked"
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
    let browser_inputs = rust
        .remove(layout::BROWSER)
        .expect("invariant: cargo_sources records the browser root");
    let shared_inputs = rust.get(layout::INTERFACE).cloned().unwrap_or_default();
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
    let web = root.join(layout::WEB);
    let mut result = String::new();
    let mut paths = Vec::new();
    paths.extend(source_inputs::web_paths(&web, &web)?);
    for name in [layout::BROWSER, layout::INTERFACE, layout::BUILDER] {
        paths.extend(walk(&root.join(name))?);
    }
    paths.extend(source_inputs::verification_paths(root)?);
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
#[path = "../../../../test/tools/assets/unit/inventory.rs"]
mod tests;
