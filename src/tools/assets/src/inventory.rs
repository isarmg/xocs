use crate::{
    layout,
    source_inputs::{self, SnapshotFiles},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

mod error;
pub use error::InventoryError;

type Result<T> = std::result::Result<T, InventoryError>;

pub type Files = BTreeMap<String, String>;
pub const MAX_FILE: u64 = source_inputs::MAX_FILE;

pub fn sha(bytes: &[u8]) -> String {
    source_inputs::sha(bytes)
}

pub fn relative(path: &Path) -> Result<String> {
    source_inputs::relative(path).map_err(|source| InventoryError::RelativePath {
        path: path.to_owned(),
        source,
    })
}

pub fn physical(path: &Path) -> Result<()> {
    source_inputs::physical(path).map_err(|source| InventoryError::InspectPath {
        path: path.to_owned(),
        source,
    })
}

pub fn read(path: &Path) -> Result<Vec<u8>> {
    source_inputs::read(path).map_err(|source| InventoryError::Read {
        path: path.to_owned(),
        source,
    })
}

pub fn walk(root: &Path) -> Result<Vec<PathBuf>> {
    source_inputs::walk(root, source_inputs::MAX_PACKAGE_FILES).map_err(|source| {
        InventoryError::EnumerateSource {
            path: root.to_owned(),
            source,
        }
    })
}

pub fn files(root: &Path) -> Result<Files> {
    walk(root)?
        .into_iter()
        .map(|path| {
            Ok((
                relative(
                    path.strip_prefix(root)
                        .expect("invariant: walked source remains under its root"),
                )?,
                sha(&read(&path)?),
            ))
        })
        .collect()
}

pub fn validate_vendor(root: &Path) -> Result<()> {
    let provenance = root.join("SOURCE.json");
    let value = receipt(&provenance)?;
    validate_vendor_receipt(root, &provenance, value)
}

fn validate_vendor_receipt(root: &Path, provenance: &Path, value: Value) -> Result<()> {
    let expected = Files::deserialize(
        value
            .get("files")
            .ok_or(InventoryError::MissingEvidence("SOURCE.json files"))?,
    )
    .map_err(|source| InventoryError::FileInventory {
        path: provenance.to_owned(),
        source,
    })?;
    if expected.is_empty() {
        return Err(InventoryError::EmptyVendorInventory);
    }
    for (name, digest) in &expected {
        relative(Path::new(name))?;
        if digest.len() != 64 || !digest.bytes().all(|v| v.is_ascii_hexdigit()) {
            return Err(InventoryError::InvalidDigest(name.clone()));
        }
    }
    let mut actual = files(root)?;
    actual.remove("SOURCE.json");
    if actual != expected {
        return Err(InventoryError::VendorChanged(root.to_owned()));
    }
    Ok(())
}

pub fn validate_vendors(root: &Path) -> Result<()> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(InventoryError::EnumerateSource {
                path: root.to_owned(),
                source,
            });
        }
    };
    for entry in entries {
        let entry = entry.map_err(|source| InventoryError::DirectoryEntry {
            path: root.to_owned(),
            source,
        })?;
        let path = entry.path();
        if !entry
            .file_type()
            .map_err(|source| InventoryError::InspectPath {
                path: path.clone(),
                source,
            })?
            .is_dir()
        {
            return Err(InventoryError::VendorEntry);
        }
        validate_vendor(&path)?;
    }
    Ok(())
}

/// Validate immutable historical evidence while allowing the maintained source
/// to evolve. Current implementation bytes are recorded separately by snapshot.
pub fn maintained_source(repository: &Path, directory: &Path) -> Result<Value> {
    let source = receipt(&directory.join("ORIGIN.json"))?;
    validate_maintained_source(repository, directory, source)
}

fn validate_maintained_source(repository: &Path, directory: &Path, source: Value) -> Result<Value> {
    if !(source["format"] == 1
        && source["ownership"] == "repository-maintained"
        && source["changes"]
            .as_array()
            .is_some_and(|changes| !changes.is_empty()))
    {
        return Err(InventoryError::Ownership);
    }
    let upstream = &source["upstream"];
    let mut original = None;
    for kind in ["receipt", "archive"] {
        let name = upstream[kind]
            .as_str()
            .ok_or(InventoryError::MissingEvidence("original source evidence"))?;
        relative(Path::new(name))?;
        let bytes = read(&repository.join(name))?;
        if upstream[format!("{kind}_sha256")].as_str() != Some(sha(&bytes).as_str()) {
            return Err(InventoryError::OriginalEvidenceChanged(name.into()));
        }
        if kind == "receipt" {
            original = Some(serde_json::from_slice::<Value>(&bytes).map_err(|source| {
                InventoryError::Receipt {
                    path: repository.join(name),
                    source,
                }
            })?);
        }
    }
    let original = original.expect("invariant: original receipt is read before its archive");
    if !(original["package"] == upstream["package"]
        && original["version"] == upstream["version"]
        && original["files"]
            .as_object()
            .is_some_and(|files| !files.is_empty()))
    {
        return Err(InventoryError::OriginalIdentityChanged);
    }
    if let Some(license) = original["files"]["LICENSE"].as_str()
        && sha(&read(&directory.join("LICENSE"))?) != license
    {
        return Err(InventoryError::LicenseChanged);
    }
    Ok(source)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OriginRequirement {
    Required,
    Optional,
}

pub fn builder_source(directory: &Path, requirement: OriginRequirement) -> Result<Option<Value>> {
    let Some(source) = optional_receipt(&directory.join("ORIGIN.json"))? else {
        return match requirement {
            OriginRequirement::Required => Err(InventoryError::MissingBuilderOrigin),
            OriginRequirement::Optional => Ok(None),
        };
    };
    let repository = directory
        .ancestors()
        .nth(Path::new(layout::BUILDER).components().count())
        .ok_or(InventoryError::BuilderLayout)?;
    let source = validate_maintained_source(repository, directory, source)?;
    let manifest = manifest(&directory.join("Cargo.toml"))?;
    if source["package"] != "blog-assets"
        || manifest
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str)
            != Some("blog-assets")
    {
        return Err(InventoryError::BuilderIdentity);
    }
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
    let base = base
        .canonicalize()
        .map_err(|source| InventoryError::ResolvePath {
            path: base.to_owned(),
            source,
        })?;
    if !base.starts_with(repository) {
        return Err(InventoryError::DependencyOutsideRepository(base));
    }
    if !seen.insert(base.clone()) {
        return Ok(());
    }
    if base
        .join("Cargo.lock")
        .try_exists()
        .map_err(|source| InventoryError::InspectPath {
            path: base.join("Cargo.lock"),
            source,
        })?
    {
        return Err(InventoryError::NestedLockfile(base));
    }
    let manifest_path = base.join("Cargo.toml");
    let bytes = hashes
        .read(&manifest_path)
        .map_err(|source| InventoryError::Read {
            path: manifest_path.clone(),
            source,
        })?;
    let manifest = parse_manifest(&manifest_path, bytes)?;
    if let Some(source) = optional_receipt(&base.join("ORIGIN.json"))? {
        validate_maintained_source(repository, &base, source)?;
    }
    let provenance = base.join("SOURCE.json");
    if let Some(value) = optional_receipt(&provenance)? {
        validate_vendor_receipt(&base, &provenance, value)?;
    }
    let input = hashes
        .files(&base)
        .map_err(|source| InventoryError::SnapshotSource {
            path: base.clone(),
            source,
        })?;
    records.insert(
        relative(
            base.strip_prefix(repository)
                .expect("invariant: accepted dependency is inside the repository"),
        )?,
        input,
    );
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
                            .ok_or_else(|| {
                                InventoryError::MissingWorkspaceDependency(name.clone())
                            })?,
                        repository,
                    )
                } else {
                    (declaration, base)
                };
            let path = declaration
                .get("path")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| InventoryError::NonLocalDependency {
                    name: name.clone(),
                    directory: base.to_owned(),
                })?;
            if declaration.get("git").is_some() || declaration.get("registry").is_some() {
                return Err(InventoryError::ExternalSourceSelector(name.clone()));
            }
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
    hashes
        .verification_inputs(root)
        .map_err(|source| InventoryError::SnapshotSource {
            path: root.to_owned(),
            source,
        })
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
    let workspace_path = root.join("Cargo.toml");
    let workspace_bytes = hashes
        .read(&workspace_path)
        .map_err(|source| InventoryError::Read {
            path: workspace_path.clone(),
            source,
        })?;
    let workspace = parse_manifest(&workspace_path, workspace_bytes)?;
    cargo_sources(
        root,
        &frontend,
        &workspace,
        &mut seen,
        &mut hashes,
        &mut rust,
    )?;
    let builder = root.join(layout::BUILDER);
    let has_builder = optional_hash(&mut hashes, &builder.join("Cargo.toml"))?.is_some();
    if has_builder {
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
    for path in
        source_inputs::web_paths(&web, &web).map_err(|source| InventoryError::EnumerateSource {
            path: web.clone(),
            source,
        })?
    {
        assets.insert(
            relative(
                path.strip_prefix(&web)
                    .expect("invariant: Web source remains inside its root"),
            )?,
            hashes
                .hash(&path)
                .map_err(|source| InventoryError::SnapshotSource {
                    path: path.clone(),
                    source,
                })?,
        );
    }
    let styles = web.join("styles");
    if let Some(source) = optional_receipt(&styles.join("ORIGIN.json"))? {
        validate_maintained_source(root, &styles, source)?;
    }
    let origin_root = root.join("compliance/web-origins");
    let origin_inputs =
        hashes
            .files(&origin_root)
            .map_err(|source| InventoryError::SnapshotSource {
                path: origin_root,
                source,
            })?;
    let builder_source = builder_source(
        &builder,
        if has_builder && optional_hash(&mut hashes, &root.join("blog-assets.json"))?.is_some() {
            OriginRequirement::Required
        } else {
            OriginRequirement::Optional
        },
    )?;
    let foundation_inputs = foundation_inputs_with(root, &mut hashes)?;
    let mut workspace_inputs = Files::new();
    for name in ["Cargo.toml", "Cargo.lock", ".cargo/config.toml"] {
        let path = root.join(name);
        if let Some(digest) = optional_hash(&mut hashes, &path)? {
            workspace_inputs.insert(name.into(), digest);
        }
    }
    let mut tool_inputs = Files::new();
    if has_builder {
        tool_inputs.extend(workspace_inputs.clone());
        tool_inputs.insert(
            layout::FOUNDATION.into(),
            foundation_digest(&foundation_inputs),
        );
        for name in ["Cargo.toml", "build.rs", "ORIGIN.json"] {
            let path = builder.join(name);
            if let Some(digest) = optional_hash(&mut hashes, &path)? {
                tool_inputs.insert(
                    relative(
                        path.strip_prefix(root)
                            .expect("invariant: builder is inside its repository"),
                    )?,
                    digest,
                );
            }
        }
        for path in walk(&builder.join("src"))? {
            tool_inputs.insert(
                relative(
                    path.strip_prefix(root)
                        .expect("invariant: walked source remains under its root"),
                )?,
                hashes
                    .hash(&path)
                    .map_err(|source| InventoryError::SnapshotSource {
                        path: path.clone(),
                        source,
                    })?,
            );
        }
    }
    if !tool_inputs.is_empty() {
        for (name, digest) in crate::compiled_inputs() {
            if !tool_inputs
                .get(name)
                .is_some_and(|actual| actual == &digest)
            {
                return Err(InventoryError::StaleBuilder(name.into()));
            }
        }
    }
    let lock_path = root.join("Cargo.lock");
    let lock_bytes = hashes
        .read(&lock_path)
        .map_err(|source| InventoryError::Read {
            path: lock_path.clone(),
            source,
        })?;
    let lock = parse_manifest(&lock_path, lock_bytes)?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or(InventoryError::LockPackages)?;
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
        "builder_sha256":sha(&serde_json::to_vec(&tool_inputs).expect("invariant: string maps serialize to JSON")),"builder_inputs":tool_inputs,"builder_source":builder_source,"cargo_packages":cargo_packages,"origin_inputs":origin_inputs,"workspace_inputs":workspace_inputs,"foundation_inputs":foundation_inputs,"lockfile":"Cargo.lock"}),
    )
}

/// Public assets retain the existing 512 KiB budget. Full per-file inputs are
/// checked internally and exported in the private frontend receipt; public
/// provenance binds those exact inventories by SHA and per-package summaries.
pub(crate) fn public_provenance(snapshot: &Value, runtime: &Value) -> Value {
    let mut public = snapshot.clone();
    let object = public
        .as_object_mut()
        .expect("invariant: snapshot is a JSON object");
    let dependencies = object
        .remove("patched_inputs")
        .expect("invariant: snapshot includes local source packages");
    let summaries = dependencies.as_object().expect("invariant: local source packages are a JSON object").iter().map(|(name, files)| {
        (name.clone(), json!({"sha256":sha(&serde_json::to_vec(files).expect("invariant: JSON values serialize")),"file_count":files.as_object().expect("invariant: package file inventories are JSON objects").len()}))
    }).collect::<BTreeMap<_,_>>();
    let foundation = object
        .remove("foundation_inputs")
        .expect("invariant: snapshot includes foundation inputs");
    object.insert("source_packages".into(), json!(summaries));
    object.insert(
        "source_snapshot_sha256".into(),
        json!(sha(
            &serde_json::to_vec(snapshot).expect("invariant: JSON values serialize")
        )),
    );
    object.insert("foundation_inventory".into(), json!({"sha256":sha(&serde_json::to_vec(&foundation).expect("invariant: JSON values serialize")),"file_count":foundation.as_object().expect("invariant: foundation inputs are a JSON object").len()}));
    let mut runtime = runtime.clone();
    let runtime_object = runtime
        .as_object_mut()
        .expect("invariant: runtime receipt is a JSON object");
    let sources = runtime_object
        .remove("source_inputs")
        .expect("invariant: runtime receipt includes source inputs");
    runtime_object.insert("source_inputs_summary".into(), json!({"sha256":sha(&serde_json::to_vec(&sources).expect("invariant: JSON values serialize")),"file_count":sources.as_object().expect("invariant: runtime source inputs are a JSON object").len()}));
    object.insert("rust_runtime".into(), runtime);
    public
}

/// Cheap watcher probe; the release build still hashes all inputs before and
/// after compilation/linking. Include inode/ctime on Unix to notice replacement
/// even when an editor restores the previous modification timestamp.
pub fn signature(root: &Path) -> Result<String> {
    let web = root.join(layout::WEB);
    let mut result = String::new();
    let mut paths = Vec::new();
    paths.extend(source_inputs::web_paths(&web, &web).map_err(|source| {
        InventoryError::EnumerateSource {
            path: web.clone(),
            source,
        }
    })?);
    for name in [layout::BROWSER, layout::INTERFACE, layout::BUILDER] {
        paths.extend(walk(&root.join(name))?);
    }
    paths.extend(source_inputs::verification_paths(root).map_err(|source| {
        InventoryError::EnumerateSource {
            path: root.to_owned(),
            source,
        }
    })?);
    for path in paths {
        let info = fs::symlink_metadata(&path).map_err(|source| InventoryError::InspectPath {
            path: path.clone(),
            source,
        })?;
        append_signature(&mut result, &path, &info)?;
    }
    for name in ["Cargo.toml", "Cargo.lock", ".cargo/config.toml"] {
        let path = root.join(name);
        match fs::symlink_metadata(&path) {
            Ok(info) => append_signature(&mut result, &path, &info)?,
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => return Err(InventoryError::InspectPath { path, source }),
        }
    }
    Ok(sha(result.as_bytes()))
}

fn append_signature(result: &mut String, path: &Path, info: &fs::Metadata) -> Result<()> {
    use std::fmt::Write;
    let modified = info
        .modified()
        .map_err(|source| InventoryError::ModificationTime {
            path: path.to_owned(),
            source,
        })?;
    write!(result, "{}:{}:{modified:?}", path.display(), info.len())
        .expect("invariant: writing metadata into a String cannot fail");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        write!(
            result,
            ":{}:{}:{}",
            info.ino(),
            info.ctime(),
            info.ctime_nsec()
        )
        .expect("invariant: writing metadata into a String cannot fail");
    }
    result.push('\n');
    Ok(())
}

fn receipt(path: &Path) -> Result<Value> {
    parse_receipt(path, &read(path)?)
}

fn parse_receipt(path: &Path, bytes: &[u8]) -> Result<Value> {
    serde_json::from_slice(bytes).map_err(|source| InventoryError::Receipt {
        path: path.to_owned(),
        source,
    })
}

fn optional_receipt(path: &Path) -> Result<Option<Value>> {
    match read(path) {
        Ok(bytes) => parse_receipt(path, &bytes).map(Some),
        Err(InventoryError::Read { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn optional_hash(hashes: &mut SnapshotFiles, path: &Path) -> Result<Option<String>> {
    match hashes.hash(path) {
        Ok(digest) => Ok(Some(digest)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(InventoryError::SnapshotSource {
            path: path.to_owned(),
            source,
        }),
    }
}

pub(crate) fn manifest(path: &Path) -> Result<toml::Value> {
    parse_manifest(path, read(path)?)
}

fn parse_manifest(path: &Path, bytes: Vec<u8>) -> Result<toml::Value> {
    let text = String::from_utf8(bytes).map_err(|source| InventoryError::ManifestEncoding {
        path: path.to_owned(),
        source,
    })?;
    toml::from_str(&text).map_err(|source| InventoryError::Manifest {
        path: path.to_owned(),
        source: Box::new(source),
    })
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/inventory.rs"]
mod tests;
