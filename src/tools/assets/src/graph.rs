//! Accept Cargo's external metadata once, then apply the browser source policy.
//! Runtime receipts contain target normal dependencies and exclude host macros.
use crate::{Options, inventory, layout, process};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::Command,
};

mod error;
pub use error::GraphError;

type Result<T> = std::result::Result<T, GraphError>;

// Cargo owns the complete schema; this projection borrows only fields used by the policy.
#[derive(Debug, Deserialize)]
struct Metadata<'a> {
    #[serde(borrow)]
    packages: Vec<Package<'a>>,
    #[serde(borrow)]
    resolve: Resolve<'a>,
}

#[derive(Debug, Deserialize)]
struct Package<'a> {
    id: &'a str,
    name: &'a str,
    version: &'a str,
    source: Option<&'a str>,
    manifest_path: &'a str,
    #[serde(borrow)]
    targets: Vec<Target<'a>>,
}

impl Package<'_> {
    fn host_macro(&self) -> bool {
        self.targets
            .iter()
            .any(|target| target.kind.contains(&"proc-macro"))
    }
}

#[derive(Debug, Deserialize)]
struct Target<'a> {
    #[serde(borrow)]
    kind: Vec<&'a str>,
    src_path: &'a str,
}

#[derive(Debug, Deserialize)]
struct Resolve<'a> {
    #[serde(borrow)]
    nodes: Vec<Node<'a>>,
}

#[derive(Debug, Deserialize)]
struct Node<'a> {
    id: &'a str,
    #[serde(borrow)]
    dependencies: Vec<&'a str>,
    #[serde(borrow)]
    features: Vec<&'a str>,
    #[serde(borrow)]
    deps: Vec<Dependency<'a>>,
}

#[derive(Debug, Deserialize)]
struct Dependency<'a> {
    pkg: &'a str,
    dep_kinds: Vec<DependencyUsage>,
}

#[derive(Debug, Deserialize)]
struct DependencyUsage {
    kind: Option<DependencyKind>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DependencyKind {
    Dev,
    Build,
}

fn metadata(value: &Value) -> Result<Metadata<'_>> {
    let metadata = Metadata::deserialize(value).map_err(GraphError::Metadata)?;
    if metadata.packages.len() > 4096 {
        return Err(GraphError::PackageBudget);
    }
    let mut ids = BTreeSet::new();
    for package in &metadata.packages {
        if !ids.insert(package.id) {
            return Err(GraphError::DuplicateId(package.id.into()));
        }
    }
    ids.clear();
    for node in &metadata.resolve.nodes {
        if !ids.insert(node.id) {
            return Err(GraphError::DuplicateId(node.id.into()));
        }
    }
    Ok(metadata)
}

fn canonicalize(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .map_err(|source| GraphError::Canonicalize {
            path: path.to_owned(),
            source,
        })
}

pub fn browser_package(root: &Path) -> Result<String> {
    let manifest = inventory::manifest(&root.join(layout::BROWSER).join("Cargo.toml"))?;
    manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or(GraphError::BrowserPackageName)
}

fn browser_root<'m, 'v>(metadata: &'m Metadata<'v>, frontend: &Path) -> Result<&'m Package<'v>> {
    let manifest = frontend.join("Cargo.toml");
    inventory::physical(&manifest)?;
    let expected = canonicalize(&manifest)?;
    let mut found = None;
    for package in &metadata.packages {
        if package.source.is_none() && canonicalize(Path::new(package.manifest_path))? == expected {
            if found.is_some() {
                return Err(GraphError::BrowserManifest);
            }
            found = Some(package);
        }
    }
    found.ok_or(GraphError::BrowserManifest)
}

fn local_manifest<'a>(package: &'a Package<'_>, repository: &Path) -> Result<&'a Path> {
    if package.source.is_some() {
        return Err(GraphError::NonLocalPackage(package.name.into()));
    }
    let manifest = Path::new(package.manifest_path);
    inventory::physical(manifest)?;
    if !canonicalize(manifest)?.starts_with(repository) {
        return Err(GraphError::ManifestOutsideRepository);
    }
    for target in &package.targets {
        let source = Path::new(target.src_path);
        inventory::physical(source)?;
        if !canonicalize(source)?.starts_with(repository) {
            return Err(GraphError::TargetOutsideRepository);
        }
    }
    Ok(manifest)
}

/// Record target normal dependencies, excluding host macros and build/dev edges.
pub(crate) fn runtime(options: &Options, inputs: &Value) -> Result<Value> {
    let frontend = options.root.join(layout::BROWSER);
    let manifest = options.root.join("Cargo.toml");
    let package = browser_package(&options.root)?;
    let run = |arguments: &[&str]| -> Result<Vec<u8>> {
        let mut command = Command::new("cargo");
        command
            .arg(format!("+{}", crate::RUST))
            .args(arguments)
            .arg("--manifest-path")
            .arg(&manifest)
            .args(["--locked", "--offline"])
            .current_dir(&options.root);
        process::run(
            &mut command,
            process::Policy::query(32 * 1024 * 1024),
            "Query browser runtime Cargo graph",
        )
        .map_err(GraphError::Command)
    };
    let value: Value = serde_json::from_slice(&run(&[
        "metadata",
        "--format-version",
        "1",
        "--filter-platform",
        crate::TARGET,
    ])?)
    .map_err(GraphError::Metadata)?;
    let metadata = metadata(&value)?;
    let tree = String::from_utf8(run(&[
        "tree",
        "--package",
        &package,
        "--target",
        crate::TARGET,
        "--edges",
        "normal,no-proc-macro",
        "--no-dedupe",
        "--prefix",
        "depth",
        "--format",
        "\t{p}",
        "--color",
        "never",
    ])?)
    .map_err(GraphError::TreeEncoding)?;
    let graph = runtime_selection(&metadata, &tree, &frontend)?;
    let mut source_inputs = BTreeMap::<String, String>::new();
    let mut append = |prefix: &str, values: &Value| {
        for (name, digest) in values
            .as_object()
            .expect("invariant: source snapshot inventories are JSON objects")
        {
            source_inputs.insert(
                format!("{prefix}{name}"),
                digest
                    .as_str()
                    .expect("invariant: source snapshot hashes are strings")
                    .into(),
            );
        }
    };
    append(&format!("{}/", layout::BROWSER), &inputs["browser_inputs"]);
    for (directory, files) in inputs["patched_inputs"]
        .as_object()
        .expect("invariant: source snapshot contains local dependencies")
    {
        append(&format!("{directory}/"), files);
    }
    for name in ["workspace_inputs", "foundation_inputs"] {
        if let Some(values) = inputs.get(name) {
            append("", values);
        }
    }
    Ok(
        json!({"target":crate::TARGET,"mode":"csr","package":browser_root(&metadata,&frontend)?.name,
        "manifest":format!("{}/Cargo.toml",layout::BROWSER),"default_features":true,"features":[],"profile":if options.debug {"debug"} else {"web-release"},"toolchain":{"rust":crate::RUST,"wasm_bindgen":crate::BRIDGE_ABI},
        "lockfile":"Cargo.lock","lock_sha256":inventory::sha(&inventory::read(&options.root.join("Cargo.lock"))?),"source_inputs":source_inputs,"graph":graph}),
    )
}

fn runtime_selection(metadata: &Metadata<'_>, tree: &str, frontend: &Path) -> Result<Value> {
    let repository = canonicalize(
        frontend
            .ancestors()
            .nth(Path::new(layout::BROWSER).components().count())
            .ok_or(GraphError::RepositoryLayout)?,
    )?;
    let browser = browser_root(metadata, frontend)?;
    let nodes: BTreeMap<_, _> = metadata
        .resolve
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect();
    let mut selected = BTreeMap::<String, (&Package<'_>, BTreeSet<String>)>::new();
    let mut stack = Vec::<String>::new();
    let mut root = None;
    for (number, line) in tree.lines().enumerate() {
        if number >= 200000 {
            return Err(GraphError::TreeBudget);
        }
        let (depth, display) = line.split_once('\t').ok_or(GraphError::MissingTreeDepth)?;
        let depth: usize = depth.parse().map_err(GraphError::TreeDepth)?;
        if depth > 256 || depth > stack.len() || (number != 0 && depth == 0) {
            return Err(GraphError::InvalidTreeDepth);
        }
        let matches = metadata
            .packages
            .iter()
            .filter(|package| {
                let prefix = format!("{} v{}", package.name, package.version);
                match package.source {
                    None => Path::new(package.manifest_path)
                        .parent()
                        .is_some_and(|path| display == format!("{prefix} ({})", path.display())),
                    Some("registry+https://github.com/rust-lang/crates.io-index") => {
                        display == prefix
                    }
                    Some(source) => source
                        .strip_prefix("git+")
                        .and_then(|source| source.rsplit_once('#'))
                        .is_some_and(|(url, revision)| {
                            display
                                .strip_prefix(&format!("{prefix} ({url}#"))
                                .and_then(|display| display.strip_suffix(')'))
                                .is_some_and(|short| {
                                    short.len() >= 7 && revision.starts_with(short)
                                })
                        }),
                }
            })
            .collect::<Vec<_>>();
        let [package] = matches.as_slice() else {
            return Err(GraphError::AmbiguousTreeSource(display.into()));
        };
        if package.host_macro() {
            return Err(GraphError::HostMacroSelected);
        }
        let manifest = local_manifest(package, &repository)?;
        let origin = format!(
            "path:{}",
            inventory::relative(
                manifest
                    .strip_prefix(&repository)
                    .expect("invariant: local manifest remains inside repository")
            )?
        );
        let id = format!("{}@{} {origin}", package.name, package.version);
        if number == 0 {
            if package.id != browser.id {
                return Err(GraphError::RuntimeRoot);
            }
            root = Some(id.clone());
        }
        if depth > 0 {
            let parent = &stack[depth - 1];
            if stack[..depth].contains(&id) {
                return Err(GraphError::RuntimeCycle);
            }
            let parent_package = selected[parent].0;
            let node = nodes
                .get(parent_package.id)
                .ok_or_else(|| GraphError::MissingRuntimeNode(parent_package.id.into()))?;
            if !node.deps.iter().any(|dependency| {
                dependency.pkg == package.id
                    && (dependency.dep_kinds.is_empty()
                        || dependency.dep_kinds.iter().any(|kind| kind.kind.is_none()))
            }) {
                return Err(GraphError::RuntimeEdge);
            }
            selected
                .get_mut(parent)
                .expect("invariant: the parent stack references a selected package")
                .1
                .insert(id.clone());
        }
        selected
            .entry(id.clone())
            .or_insert((package, BTreeSet::new()));
        stack.truncate(depth);
        stack.push(id);
    }
    let lock = inventory::manifest(&repository.join("Cargo.lock"))?;
    let lock_packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or(GraphError::LockPackages)?;
    let records = selected.into_iter().map(|(id, (package, dependencies))| -> Result<Value> {
        let manifest = inventory::relative(local_manifest(package, &repository)?.strip_prefix(&repository).expect("invariant: local manifest remains inside repository"))?;
        if !lock_packages.iter().any(|entry| entry.get("name").and_then(toml::Value::as_str) == Some(package.name) && entry.get("version").and_then(toml::Value::as_str) == Some(package.version) && entry.get("source").is_none()) {
            return Err(GraphError::PackageMissingFromLock(package.name.into()));
        }
        Ok(json!({"id":id,"name":package.name,"version":package.version,"source":null,"manifest":manifest,"checksum":null,"dependencies":dependencies}))
    }).collect::<Result<Vec<_>>>()?;
    Ok(json!({"root":root.ok_or(GraphError::EmptyTree)?,"packages":records}))
}

/// Resolve the selected CSR dependencies through bounded, offline Cargo metadata.
pub fn resolve(options: &Options) -> Result<Value> {
    let mut command = Command::new("cargo");
    command
        .arg(format!("+{}", crate::RUST))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--filter-platform",
            crate::TARGET,
            "--manifest-path",
        ])
        .arg(options.root.join("Cargo.toml"))
        .current_dir(&options.root)
        .arg("--offline");
    let output = process::run(
        &mut command,
        process::Policy::query(32 * 1024 * 1024),
        "Inspect selected CSR dependency graph",
    )
    .map_err(GraphError::Command)?;
    let value: Value = serde_json::from_slice(&output).map_err(GraphError::Metadata)?;
    validate(&value, &options.root.join(layout::BROWSER))
}

/// Validate an external Cargo metadata value against the integrated CSR policy.
pub fn validate(value: &Value, frontend: &Path) -> Result<Value> {
    let metadata = metadata(value)?;
    let by_id: BTreeMap<_, _> = metadata
        .packages
        .iter()
        .map(|package| (package.id, package))
        .collect();
    let nodes: BTreeMap<_, _> = metadata
        .resolve
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect();
    let root = browser_root(&metadata, frontend)?.id;
    let mut reachable = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if !reachable.insert(id) {
            continue;
        }
        let node = nodes
            .get(id)
            .ok_or_else(|| GraphError::MissingNode(id.into()))?;
        pending.extend(node.dependencies.iter().copied());
    }
    let repository = canonicalize(
        frontend
            .ancestors()
            .nth(Path::new(layout::BROWSER).components().count())
            .ok_or(GraphError::RepositoryLayout)?,
    )?;
    let integrated = repository.join(layout::INTERFACE);
    let mut facade = Vec::new();
    for id in &reachable {
        let package = by_id
            .get(id)
            .ok_or_else(|| GraphError::MissingPackage((*id).into()))?;
        if package.name == "blog-interface" {
            facade.push(*package);
        }
    }
    let [facade] = facade.as_slice() else {
        return Err(GraphError::FacadeCount);
    };
    if facade.source.is_some()
        || canonicalize(Path::new(facade.manifest_path))?
            != canonicalize(&integrated.join("Cargo.toml"))?
    {
        return Err(GraphError::FacadeSource);
    }
    let mut selected = Vec::new();
    for (name, directory) in [("blog-dom", "dom"), ("blog-component-macros", "macros")] {
        let candidates = reachable
            .iter()
            .filter_map(|id| {
                by_id
                    .get(id)
                    .filter(|package| package.name == name)
                    .map(|package| (*id, *package))
            })
            .collect::<Vec<_>>();
        let [(id, package)] = candidates.as_slice() else {
            return Err(GraphError::IntegratedCount {
                name,
                actual: candidates.len(),
            });
        };
        let expected = integrated.join(directory).join("Cargo.toml");
        let actual = Path::new(package.manifest_path);
        if package.source.is_some() || canonicalize(actual)? != canonicalize(&expected)? {
            return Err(GraphError::IntegratedSource(name));
        }
        let features: BTreeSet<_> = nodes[id].features.iter().copied().collect();
        let allowed = BTreeSet::from(["csr", "default"]);
        if !features.contains("csr") || !features.is_subset(&allowed) {
            return Err(GraphError::IntegratedFeatures {
                name,
                features: features.into_iter().map(str::to_owned).collect(),
            });
        }
        if features.contains("default") {
            let manifest = inventory::manifest(actual)?;
            let defaults = manifest
                .get("features")
                .and_then(|value| value.get("default"))
                .and_then(toml::Value::as_array);
            if !defaults
                .is_some_and(|values| values.len() == 1 && values[0].as_str() == Some("csr"))
            {
                return Err(GraphError::DefaultFeatures(name));
            }
        }
        let host_macro = package.host_macro();
        if host_macro != (name == "blog-component-macros") {
            return Err(GraphError::TargetRole);
        }
        selected.push(json!({"name":name,"version":package.version,"path":inventory::relative(expected.strip_prefix(&repository).expect("invariant: integrated package is inside repository"))?,"features":features,"role":if host_macro {"host-proc-macro"} else {"browser"}}));
    }
    for id in reachable {
        let package = by_id
            .get(id)
            .ok_or_else(|| GraphError::MissingPackage(id.into()))?;
        local_manifest(package, &repository)?;
        let name = package.name;
        if matches!(
            name,
            "leptos"
                | "leptos_macro"
                | "server_fn"
                | "server_fn_macro"
                | "leptos_server"
                | "hydration_context"
                | "leptos_config"
        ) {
            return Err(GraphError::UnsupportedPackage(name.into()));
        }
        if matches!(
            name,
            "blog-interface-reactivity" | "blog-interface-view-tree"
        ) && nodes[id]
            .features
            .iter()
            .any(|feature| matches!(*feature, "ssr" | "hydrate" | "hydration" | "islands"))
        {
            return Err(GraphError::ServerFeature(name.into()));
        }
    }
    Ok(
        json!({"target":crate::TARGET,"mode":"csr","feature_scope":"cargo-resolution-union","integrated":selected}),
    )
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/graph.rs"]
mod tests;
