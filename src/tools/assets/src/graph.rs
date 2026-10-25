//! Verify integrated sources and Cargo's conservative resolution graph.
//! Metadata may include features from conditional edges that the current host
//! compilation does not activate. This policy guard is not a compiled-unit
//! inventory; host proc-macros are never counted as browser WASM code.
use crate::{Options, inventory, layout, process};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};

pub fn browser_package(root: &Path) -> Result<String> {
    let manifest: toml::Value = toml::from_str(&String::from_utf8(inventory::read(
        &root.join(layout::BROWSER).join("Cargo.toml"),
    )?)?)?;
    Ok(manifest["package"]["name"]
        .as_str()
        .context("Browser package name")?
        .to_owned())
}

fn browser_root<'a>(metadata: &'a Value, frontend: &Path) -> Result<&'a Value> {
    let manifest = frontend.join("Cargo.toml");
    inventory::physical(&manifest)?;
    let expected = manifest.canonicalize()?;
    let mut found = None;
    for package in metadata["packages"].as_array().context("Cargo packages")? {
        if package["source"].is_null() {
            let path = package["manifest_path"]
                .as_str()
                .context("Cargo manifest")?;
            if Path::new(path).canonicalize()? == expected {
                ensure!(
                    found.is_none(),
                    "Workspace must contain exactly one browser manifest"
                );
                found = Some(package);
            }
        }
    }
    found.context("Workspace must contain exactly one browser manifest")
}

fn local_manifest<'a>(package: &'a Value, repository: &Path) -> Result<&'a Path> {
    ensure!(
        package["source"].is_null(),
        "Every Rust package must be integrated as a local path: {}",
        package["name"]
    );
    let manifest = Path::new(
        package["manifest_path"]
            .as_str()
            .context("Cargo manifest")?,
    );
    inventory::physical(manifest)?;
    ensure!(
        manifest.canonicalize()?.starts_with(repository),
        "Rust manifest escapes repository"
    );
    for target in package["targets"].as_array().context("Cargo targets")? {
        let source = Path::new(target["src_path"].as_str().context("Cargo target source")?);
        inventory::physical(source)?;
        ensure!(
            source.canonicalize()?.starts_with(repository),
            "Rust target source escapes repository"
        );
    }
    Ok(manifest)
}

/// Record Cargo's selected target runtime tree, excluding host macros and all
/// build/dev edges. Portable identities can be independently rechecked by the
/// release producer in a private source export.
pub fn runtime(options: &Options, inputs: &Value) -> Result<Value> {
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
        Ok(process::run(
            &mut command,
            process::Policy::query(32 * 1024 * 1024),
            "Query browser runtime Cargo graph",
        )?)
    };
    let metadata: Value = serde_json::from_slice(&run(&[
        "metadata",
        "--format-version",
        "1",
        "--filter-platform",
        crate::TARGET,
    ])?)?;
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
    ])?)?;
    let graph = runtime_selection(&metadata, &tree, &frontend)?;
    let mut source_inputs = BTreeMap::<String, String>::new();
    let mut append = |prefix: &str, values: &Value| -> Result<()> {
        for (name, digest) in values.as_object().context("Rust source input inventory")? {
            let name = format!("{prefix}{name}");
            inventory::relative(Path::new(&name))?;
            source_inputs.insert(
                name,
                digest.as_str().context("Rust source input hash")?.into(),
            );
        }
        Ok(())
    };
    append(&format!("{}/", layout::BROWSER), &inputs["browser_inputs"])?;
    for (directory, files) in inputs["patched_inputs"]
        .as_object()
        .context("Local Rust dependencies")?
    {
        append(&format!("{directory}/"), files)?;
    }
    for name in ["workspace_inputs", "foundation_inputs"] {
        if inputs.get(name).is_some() {
            append("", &inputs[name])?;
        }
    }
    Ok(
        json!({"target":crate::TARGET,"mode":"csr","package":browser_root(&metadata,&frontend)?["name"],
        "manifest":format!("{}/Cargo.toml",layout::BROWSER),"default_features":true,"features":[],"profile":if options.debug {"debug"} else {"web-release"},"toolchain":{"rust":crate::RUST,"wasm_bindgen":crate::BRIDGE_ABI},
        "lockfile":"Cargo.lock","lock_sha256":inventory::sha(&inventory::read(&options.root.join("Cargo.lock"))?),"source_inputs":source_inputs,"graph":graph}),
    )
}

fn runtime_selection(metadata: &Value, tree: &str, frontend: &Path) -> Result<Value> {
    let repository = frontend
        .ancestors()
        .nth(Path::new(layout::BROWSER).components().count())
        .context("Repository directory")?
        .canonicalize()?;
    let browser = browser_root(metadata, frontend)?;
    let packages = metadata["packages"]
        .as_array()
        .context("Cargo metadata packages")?;
    ensure!(
        packages.len() <= 4096,
        "Cargo runtime package count exceeds bound"
    );
    let mut selected = BTreeMap::<String, (&Value, BTreeSet<String>)>::new();
    let mut stack = Vec::<String>::new();
    let mut root = None;
    for (number, line) in tree.lines().enumerate() {
        ensure!(number < 200000, "Cargo runtime tree exceeds line bound");
        let (depth, display) = line.split_once('\t').context("Cargo runtime tree depth")?;
        let depth: usize = depth.parse()?;
        ensure!(
            depth <= 256 && depth <= stack.len() && (number == 0 || depth > 0),
            "Cargo runtime tree depth changed"
        );
        let matches = packages
            .iter()
            .filter(|p| {
                let prefix = format!(
                    "{} v{}",
                    p["name"].as_str().unwrap_or(""),
                    p["version"].as_str().unwrap_or("")
                );
                if p["source"].is_null() {
                    Path::new(p["manifest_path"].as_str().unwrap_or(""))
                        .parent()
                        .is_some_and(|path| display == format!("{prefix} ({})", path.display()))
                } else if p["source"] == "registry+https://github.com/rust-lang/crates.io-index" {
                    display == prefix
                } else if let Some(source) =
                    p["source"].as_str().and_then(|s| s.strip_prefix("git+"))
                {
                    source.rsplit_once('#').is_some_and(|(url, revision)| {
                        display
                            .strip_prefix(&format!("{prefix} ({url}#"))
                            .and_then(|s| s.strip_suffix(')'))
                            .is_some_and(|short| short.len() >= 7 && revision.starts_with(short))
                    })
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();
        ensure!(
            matches.len() == 1,
            "Cargo runtime tree source is ambiguous: {display}"
        );
        let package = matches[0];
        ensure!(
            !package["targets"]
                .as_array()
                .context("Cargo package targets")?
                .iter()
                .any(|t| t["kind"]
                    .as_array()
                    .is_some_and(|k| k.iter().any(|k| k == "proc-macro"))),
            "Host macro was selected as browser runtime"
        );
        let manifest = local_manifest(package, &repository)?;
        let origin = format!(
            "path:{}",
            inventory::relative(manifest.strip_prefix(&repository)?)?
        );
        let id = format!(
            "{}@{} {origin}",
            package["name"].as_str().context("Cargo name")?,
            package["version"].as_str().context("Cargo version")?
        );
        if number == 0 {
            ensure!(
                package["id"] == browser["id"],
                "Cargo runtime tree has another root"
            );
            root = Some(id.clone());
        }
        if depth > 0 {
            let parent = &stack[depth - 1];
            ensure!(
                !stack[..depth].contains(&id),
                "Cargo runtime dependency cycle"
            );
            let parent_package = selected[parent].0;
            let node = metadata["resolve"]["nodes"]
                .as_array()
                .context("Cargo nodes")?
                .iter()
                .find(|n| n["id"] == parent_package["id"])
                .context("Cargo runtime parent")?;
            ensure!(
                node["deps"]
                    .as_array()
                    .context("Cargo dependency edges")?
                    .iter()
                    .any(|d| {
                        d["pkg"] == package["id"]
                            && d["dep_kinds"].as_array().is_some_and(|k| {
                                k.is_empty() || k.iter().any(|k| k["kind"].is_null())
                            })
                    }),
                "Cargo runtime tree edge differs from normal metadata"
            );
            selected
                .get_mut(parent)
                .context("Cargo runtime parent missing")?
                .1
                .insert(id.clone());
        }
        selected
            .entry(id.clone())
            .or_insert((package, BTreeSet::new()));
        stack.truncate(depth);
        stack.push(id);
    }
    let lock: toml::Value = toml::from_str(&String::from_utf8(inventory::read(
        &repository.join("Cargo.lock"),
    )?)?)?;
    let lock_packages = lock["package"].as_array().context("Cargo lock packages")?;
    let records = selected.into_iter().map(|(id, (p, dependencies))| -> Result<Value> {
        let manifest = inventory::relative(local_manifest(p, &repository)?.strip_prefix(&repository)?)?;
        ensure!(lock_packages.iter().any(|entry| entry["name"].as_str() == p["name"].as_str()
            && entry["version"].as_str() == p["version"].as_str() && entry.get("source").is_none()),
            "Selected path package missing from root Cargo.lock");
        Ok(json!({"id":id,"name":p["name"],"version":p["version"],"source":null,"manifest":manifest,"checksum":null,"dependencies":dependencies}))
    }).collect::<Result<Vec<_>>>()?;
    Ok(json!({"root":root.context("Cargo runtime tree is empty")?,"packages":records}))
}

pub fn resolve(options: &Options) -> Result<Value> {
    let frontend = options.root.join(layout::BROWSER);
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
        .current_dir(&options.root);
    command.arg("--offline");
    let output = process::run(
        &mut command,
        process::Policy::query(32 * 1024 * 1024),
        "Inspect selected CSR dependency graph",
    )?;
    validate(&serde_json::from_slice(&output)?, &frontend)
}

pub fn validate(metadata: &Value, frontend: &Path) -> Result<Value> {
    let packages = metadata["packages"]
        .as_array()
        .context("Cargo metadata packages")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .context("Cargo metadata resolve nodes")?;
    let by_id = packages
        .iter()
        .map(|p| Ok((p["id"].as_str().context("Cargo package id")?, p)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let root = browser_root(metadata, frontend)?["id"]
        .as_str()
        .context("Browser package id")?;
    let node_ids = nodes
        .iter()
        .map(|p| Ok((p["id"].as_str().context("Cargo node id")?, p)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut reachable = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if !reachable.insert(id) {
            continue;
        }
        let node = node_ids
            .get(id)
            .context("Missing resolved dependency node")?;
        for dependency in node["dependencies"]
            .as_array()
            .context("Dependency node edges")?
        {
            pending.push(dependency.as_str().context("Dependency id")?);
        }
    }
    let repository = frontend
        .ancestors()
        .nth(Path::new(layout::BROWSER).components().count())
        .context("Repository directory")?
        .canonicalize()?;
    let integrated = repository.join(layout::INTERFACE);
    let facade = reachable
        .iter()
        .filter_map(|id| {
            by_id
                .get(id)
                .filter(|p| p["name"] == "blog-interface")
                .copied()
        })
        .collect::<Vec<_>>();
    ensure!(
        facade.len() == 1,
        "CSR requires exactly one integrated blog-interface facade"
    );
    ensure!(
        facade[0]["source"].is_null()
            && Path::new(
                facade[0]["manifest_path"]
                    .as_str()
                    .context("Facade manifest")?
            )
            .canonicalize()?
                == integrated.join("Cargo.toml").canonicalize()?,
        "blog-interface must resolve to the integrated facade source"
    );
    let mut selected = Vec::new();
    for (name, directory) in [("blog-dom", "dom"), ("blog-component-macros", "macros")] {
        let candidates = reachable
            .iter()
            .filter_map(|id| {
                by_id
                    .get(id)
                    .filter(|p| p["name"] == name)
                    .map(|p| (*id, *p))
            })
            .collect::<Vec<_>>();
        ensure!(
            candidates.len() == 1,
            "CSR requires exactly one integrated {name}; found {}",
            candidates.len()
        );
        let (id, package) = candidates[0];
        let expected = integrated.join(directory).join("Cargo.toml");
        let actual = Path::new(
            package["manifest_path"]
                .as_str()
                .context("Selected package manifest")?,
        );
        ensure!(
            package["source"].is_null() && actual.canonicalize()? == expected.canonicalize()?,
            "{name} must resolve to the project-maintained CSR source"
        );
        let features = node_ids[id]["features"]
            .as_array()
            .context("Resolved package features")?
            .iter()
            .map(|v| v.as_str().context("Feature name"))
            .collect::<Result<BTreeSet<_>>>()?;
        let allowed = BTreeSet::from(["csr", "default"]);
        ensure!(
            features.contains("csr") && features.is_subset(&allowed),
            "{name} only permits the supported csr feature; selected {features:?}"
        );
        if features.contains("default") {
            let manifest: toml::Value =
                toml::from_str(&String::from_utf8(inventory::read(actual)?)?)?;
            let defaults = manifest
                .get("features")
                .and_then(|value| value.get("default"))
                .and_then(toml::Value::as_array);
            ensure!(
                defaults
                    .is_some_and(|values| values.len() == 1 && values[0].as_str() == Some("csr")),
                "{name} default must be an alias for csr alone"
            );
        }
        let host_macro = package["targets"]
            .as_array()
            .context("Package targets")?
            .iter()
            .any(|target| {
                target["kind"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|kind| kind == "proc-macro"))
            });
        ensure!(
            host_macro == (name == "blog-component-macros"),
            "Integrated runtime/proc-macro target roles changed"
        );
        selected.push(json!({"name":name,"version":package["version"],"path":inventory::relative(expected.strip_prefix(&repository)?)?,"features":features,"role":if host_macro {"host-proc-macro"} else {"browser"}}));
    }
    for id in reachable {
        let package = by_id[id];
        local_manifest(package, &repository)?;
        let name = package["name"].as_str().context("Package name")?;
        ensure!(
            !matches!(
                name,
                "leptos"
                    | "leptos_macro"
                    | "server_fn"
                    | "server_fn_macro"
                    | "leptos_server"
                    | "hydration_context"
                    | "leptos_config"
            ),
            "Server/hydration package unexpectedly selected in CSR graph: {name}"
        );
        if matches!(
            name,
            "blog-interface-reactivity" | "blog-interface-view-tree"
        ) {
            let features = node_ids[id]["features"]
                .as_array()
                .context("Dependency features")?;
            ensure!(
                !features.iter().any(|feature| matches!(
                    feature.as_str(),
                    Some("ssr" | "hydrate" | "hydration" | "islands")
                )),
                "{name} selected a server/hydration feature"
            );
        }
    }
    Ok(
        json!({"target":crate::TARGET,"mode":"csr","feature_scope":"cargo-resolution-union","integrated":selected}),
    )
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/graph.rs"]
mod tests;
