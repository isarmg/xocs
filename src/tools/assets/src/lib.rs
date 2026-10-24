pub mod assets;
mod browser_module;
pub mod dev;
pub mod graph;
pub mod inventory;
pub mod javascript;
mod layout;
pub mod server_build;
mod source_inputs;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub const RUST: &str = "1.99.0";
pub const BRIDGE_ABI: &str = "blog-browser-bridge/1";
pub const TARGET: &str = "wasm32-unknown-unknown";

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

fn validate_distribution(root: &Path, output: &Path) -> Result<()> {
    ensure!(
        !root.starts_with(output)
            && output != root.join(layout::WEB)
            && (!output.starts_with(root.join("src"))
                || output.starts_with(root.join(layout::WEB).join("dist")))
            && !["test", "crates", "tools", "compliance", "docs"]
                .iter()
                .any(|directory| output.starts_with(root.join(directory)))
            && output != root.join(layout::WEB).join("pkg"),
        "Distribution must be a generated directory separate from authored source and source ancestors"
    );
    Ok(())
}

pub fn distribution(options: &Options) -> Result<PathBuf> {
    let output = options.dist.clone().unwrap_or(PathBuf::from(
        assets::Config::read(&options.root.join(layout::WEB))?
            .web
            .dist,
    ));
    let output = if output.is_absolute() {
        output
    } else {
        options.root.join(output)
    };
    validate_distribution(&options.root, &output)?;
    let parent = output
        .parent()
        .context("Distribution needs a parent directory")?;
    fs::create_dir_all(parent)?;
    inventory::physical(parent)?;
    let output = parent
        .canonicalize()?
        .join(output.file_name().context("Distribution directory name")?);
    validate_distribution(&options.root, &output)?;
    Ok(output)
}

pub fn cargo_action(options: &Options, action: &str) -> Result<()> {
    graph::resolve(options)?;
    let mut command = Command::new("cargo");
    command
        .arg(format!("+{RUST}"))
        .arg(action)
        .args(["--manifest-path"])
        .arg(options.root.join("Cargo.toml"))
        .args([
            "--package",
            &graph::browser_package(&options.root)?,
            "--locked",
            "--offline",
        ])
        .current_dir(&options.root);
    if action != "test" {
        command.args(["--target", TARGET]);
    }
    if action == "build" && !options.debug {
        command.args(["--profile", "web-release"]);
    }
    command.env("CARGO_TARGET_DIR", target_directory(options)?);
    if env::var_os("CARGO_BUILD_JOBS").is_none() {
        command.env("CARGO_BUILD_JOBS", "2");
    }
    if env::var_os("CARGO_PROFILE_DEV_DEBUG").is_none() {
        command.env("CARGO_PROFILE_DEV_DEBUG", "0");
    }
    if env::var_os("CARGO_PROFILE_TEST_DEBUG").is_none() {
        command.env("CARGO_PROFILE_TEST_DEBUG", "0");
    }
    ensure!(
        command
            .status()
            .context("Run Cargo browser build")?
            .success(),
        "Browser cargo {action} failed"
    );
    Ok(())
}

fn target_directory(options: &Options) -> Result<PathBuf> {
    let path = env::var_os("BLOG_WASM_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| options.root.join("target/blog-browser"));
    Ok(if path.is_absolute() {
        path
    } else {
        options.root.join(path)
    })
}

fn validate_generated(directory: &Path) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    inventory::physical(directory)?;
    ensure!(
        directory.is_dir(),
        "Generated destination must be a physical directory"
    );
    inventory::files(directory)?;
    ensure!(
        fs::read_dir(directory)?.next().is_none()
            || directory.join("blog-assets.json").is_file()
            || directory.join("assets/blog-assets.json").is_file(),
        "Refusing to replace a nonempty directory without blog-browser generated-output provenance: {}",
        directory.display()
    );
    Ok(())
}

fn receipt_destination(path: &Path, pkg: &Path, dist: &Path) -> Result<PathBuf> {
    ensure!(
        path.is_absolute(),
        "Runtime inventory needs an absolute path"
    );
    let parent = path.parent().context("Runtime inventory parent")?;
    inventory::physical(parent)?;
    let path = parent
        .canonicalize()?
        .join(path.file_name().context("Runtime inventory filename")?);
    for output in [pkg, dist] {
        let output = output
            .parent()
            .context("Generated output parent")?
            .canonicalize()?
            .join(output.file_name().context("Generated output filename")?);
        ensure!(
            !path.starts_with(output),
            "Runtime inventory must be separate from generated output directories"
        );
    }
    Ok(path)
}

fn replace_pair(
    pkg: &Path,
    stage_pkg: &Path,
    dist: &Path,
    stage_dist: &Path,
    backup: &Path,
    receipt: Option<(&Path, &[u8])>,
) -> Result<()> {
    validate_generated(pkg)?;
    validate_generated(dist)?;
    let receipt = receipt
        .map(|(path, bytes)| -> Result<_> {
            use std::io::Write;
            let path = receipt_destination(path, pkg, dist)?;
            let parent = path.parent().expect("Normalized absolute receipt path");
            let mut builder = tempfile::Builder::new();
            builder.prefix(".blog-frontend-receipt-");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                builder.permissions(fs::Permissions::from_mode(0o600));
            }
            let mut temporary = builder.tempfile_in(parent)?;
            temporary.write_all(bytes)?;
            temporary.as_file().sync_all()?;
            Ok((temporary, path))
        })
        .transpose()?;
    let old_pkg = backup.join("previous-pkg");
    let old_dist = backup.join("previous-dist");
    let had_pkg = pkg.exists();
    let had_dist = dist.exists();
    if had_pkg {
        fs::rename(pkg, &old_pkg)?;
    }
    if had_dist && let Err(error) = fs::rename(dist, &old_dist) {
        if had_pkg {
            fs::rename(&old_pkg, pkg)?;
        }
        return Err(error.into());
    }
    if let Err(error) = fs::rename(stage_pkg, pkg) {
        if had_pkg {
            fs::rename(&old_pkg, pkg)?;
        }
        if had_dist {
            fs::rename(&old_dist, dist)?;
        }
        return Err(error.into());
    }
    let mut published_dist = false;
    let publish = (|| -> Result<()> {
        fs::rename(stage_dist, dist)?;
        published_dist = true;
        if let Some((temporary, path)) = receipt {
            temporary.persist_noclobber(path)?;
        }
        Ok(())
    })();
    if let Err(error) = publish {
        if published_dist {
            fs::rename(dist, stage_dist)?;
        }
        fs::rename(pkg, stage_pkg)?;
        if had_pkg {
            fs::rename(&old_pkg, pkg)?;
        }
        if had_dist {
            fs::rename(&old_dist, dist)?;
        }
        return Err(error);
    }
    Ok(())
}

pub fn build(options: &Options) -> Result<Value> {
    inventory::physical(&options.root)?;
    let web = options.root.join(layout::WEB);
    let manifest = options.root.join(layout::BROWSER).join("Cargo.toml");
    let declaration: toml::Value =
        toml::from_str(&String::from_utf8(inventory::read(&manifest)?)?)?;
    let package = declaration["package"]["name"]
        .as_str()
        .context("Browser package name")?
        .replace('-', "_");
    let config = assets::Config::read(&web)?;
    let runtime_root = if config.web.runtime_root == "product" {
        options.root.clone()
    } else {
        web.clone()
    };
    let dist = distribution(options)?;
    validate_generated(&web.join("pkg"))?;
    validate_generated(&dist)?;
    let inventory_path = env::var_os("BLOG_FRONTEND_METADATA_FILE")
        .map(|path| receipt_destination(Path::new(&path), &web.join("pkg"), &dist))
        .transpose()?;
    if let Some(path) = &inventory_path {
        ensure!(
            path.is_absolute() && !path.exists(),
            "Runtime inventory needs absolute paths and a new output file"
        );
        inventory::physical(path.parent().context("Runtime inventory parent")?)?;
        ensure!(
            dist.starts_with(&runtime_root),
            "Blog runtime receipt requires distribution inside its configured source root"
        );
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
        "source_snapshot_sha256": inventory::sha(&serde_json::to_vec(&before)?),
    });
    cargo_action(options, "build")?;
    ensure!(
        inventory::snapshot(&options.root)? == before,
        "Source changed during browser compilation; previous outputs were preserved"
    );
    let stage = tempfile::Builder::new()
        .prefix(".rust-web-stage-")
        .tempdir_in(&web)?;
    let stage_pkg = stage.path().join("pkg");
    let stage_dist = stage.path().join("dist");
    fs::create_dir(&stage_pkg)?;
    fs::create_dir(&stage_dist)?;
    let profile = if options.debug {
        "debug"
    } else {
        "web-release"
    };
    let binary = target_directory(options)?
        .join(TARGET)
        .join(profile)
        .join(format!("{package}.wasm"));
    browser_module::generate(&binary, &stage_pkg, &package)?;
    let generated = inventory::files(&stage_pkg)?;
    let mut metadata = inventory::public_provenance(&before, &rust_runtime)?;
    let object = metadata
        .as_object_mut()
        .context("Source inventory object")?;
    object.extend(json!({"format":2,"rust":RUST,"browser_bridge":BRIDGE_ABI,"package":package,"profile":profile,"files":generated,"asset_builder":"blog-assets","csr_graph":csr_graph,"bindings_tool":bindings_tool}).as_object().expect("Constant object").clone());
    let mut provenance = serde_json::to_vec(&metadata)?;
    provenance.push(b'\n');
    fs::write(stage_pkg.join("blog-assets.json"), &provenance)?;
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
    let final_files = assets::write_outputs(&stage_dist, &outputs)?;
    ensure!(
        inventory::snapshot(&options.root)? == before,
        "Source changed during asset linking; previous outputs were preserved"
    );
    runtime["distribution"] = json!(if let Ok(relative) = dist.strip_prefix(&runtime_root) {
        inventory::relative(relative)?
    } else {
        dist.to_string_lossy().into_owned()
    });
    let mut runtime_bytes = serde_json::to_vec_pretty(&runtime)?;
    runtime_bytes.push(b'\n');
    ensure!(
        runtime_bytes.len() <= 32 * 1024 * 1024,
        "Runtime inventory exceeds byte bound"
    );
    // Metadata is staged with the output and published as a create-new file only
    // after every source/hash/license/budget gate has passed.
    replace_pair(
        &web.join("pkg"),
        &stage_pkg,
        &dist,
        &stage_dist,
        stage.path(),
        inventory_path
            .as_deref()
            .map(|path| (path, runtime_bytes.as_slice())),
    )?;
    let summary = json!({"distribution":dist,"files":final_files.len(),"bytes":outputs.values().map(Vec::len).sum::<usize>(),"runtime":runtime});
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
