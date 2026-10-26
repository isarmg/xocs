use super::*;
use anyhow::Result;

#[test]
fn source_read_and_receipt_errors_preserve_their_operation_and_source() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let missing = directory.path().join("missing.json");
    let error = read(&missing).unwrap_err();
    let InventoryError::Read { path, source } = error else {
        panic!("missing source should report the read operation");
    };
    assert_eq!(path, missing);
    assert_eq!(source.kind(), io::ErrorKind::NotFound);
    let malformed = directory.path().join("ORIGIN.json");
    fs::write(&malformed, b"{")?;
    assert!(matches!(
        maintained_source(directory.path(), directory.path()),
        Err(InventoryError::Receipt { path, source }) if path == malformed && source.is_eof()
    ));
    Ok(())
}

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
    for name in ["", "/etc/passwd", "../file", "a/../file", "a\\b"] {
        assert!(relative(Path::new(name)).is_err());
    }
}

#[test]
fn source_names_cannot_alias_directory_separators_in_receipts() -> Result<()> {
    let directory = tempfile::tempdir()?;
    fs::create_dir(directory.path().join("a"))?;
    fs::write(directory.path().join("a/b"), "nested input")?;
    fs::write(directory.path().join("a\\b"), "different input")?;
    assert!(files(directory.path()).is_err());
    Ok(())
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
    let vendor = dir.path().join("src/web/vendor/example");
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
    let web = dir.path().join("src/web");
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
    fs::create_dir_all(root.join("src/web"))?;
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
    assert!(before["patched_inputs"]["src/foundation/local-1.0.0"]["native/helper.c"].is_string());
    assert!(
        before["foundation_inputs"]["docs/dependencies/origins/local-1.0.0/README.md"].is_string()
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
        before["patched_inputs"]["src/foundation/local-1.0.0"]["src/target/data.rs"].is_string()
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
    let public = public_provenance(&snapshot, &runtime);
    assert!(serde_json::to_vec(&snapshot)?.len() > 512 * 1024);
    assert!(serde_json::to_vec(&public)?.len() < 512 * 1024);
    assert_eq!(public["source_packages"].as_object().unwrap().len(), 400);
    assert_eq!(public["foundation_inventory"]["file_count"], 20000);
    snapshot["patched_inputs"]["src/foundation/pkg0-1"]["src/module0.rs"] = json!("2".repeat(64));
    let changed = public_provenance(&snapshot, &runtime);
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

#[test]
fn accepted_custom_web_sources_are_bound_to_snapshots_and_watcher_signatures() -> Result<()> {
    let repository = tempfile::tempdir()?;
    let root = repository.path();
    let web = root.join("src/web");
    for directory in [
        web.join("src"),
        web.join("pkg"),
        web.join("custom"),
        root.join("src/browser/src"),
    ] {
        fs::create_dir_all(directory)?;
    }
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers=['src/browser']\nresolver='3'\n",
    )?;
    fs::write(
        root.join("Cargo.lock"),
        "version=4\n[[package]]\nname='browser-fixture'\nversion='1.0.0'\n",
    )?;
    fs::write(
        root.join("src/browser/Cargo.toml"),
        "[package]\nname='browser-fixture'\nversion='1.0.0'\nedition='2024'\n",
    )?;
    fs::write(root.join("src/browser/src/lib.rs"), "")?;
    fs::write(
        web.join("src/main.js"),
        "export {value} from '../custom/module.js';export default function init(){return new URL('../pkg/app.wasm',import.meta.url)}",
    )?;
    fs::write(web.join("pkg/app.wasm"), b"\0asm\x01\0\0\0")?;
    fs::write(web.join("custom/module.js"), "export const value='first';")?;
    fs::write(web.join("custom/notice.txt"), "Custom notice")?;
    fs::write(web.join("custom/static.txt"), "Custom static data")?;
    fs::write(
        web.join("index.html"),
        "<html><head></head><body><script type=\"module\" src=\"/src/main.js\"></script></body></html>",
    )?;
    fs::write(
        web.join("rust-web.toml"),
        "[web]\nentry='src/main.js'\nlicenses=['custom/notice.txt']\n[web.copies]\n'assets/static.txt'='custom/static.txt'\n",
    )?;
    let before = snapshot(root)?;
    for name in ["custom/module.js", "custom/notice.txt", "custom/static.txt"] {
        assert!(before["asset_inputs"][name].is_string(), "{name}");
    }
    let first_signature = signature(root)?;
    fs::write(web.join("custom/module.js"), "export const value='second';")?;
    let after = snapshot(root)?;
    assert_ne!(
        before["asset_inputs"]["custom/module.js"],
        after["asset_inputs"]["custom/module.js"]
    );
    assert_ne!(first_signature, signature(root)?);
    let config = crate::assets::Config::read(&web)?;
    let (outputs, _) = crate::assets::Linker::new(&web, &web.join("pkg"), config).finish(b"{}")?;
    assert!(
        outputs
            .values()
            .any(|bytes| String::from_utf8_lossy(bytes).contains("second"))
    );
    assert_eq!(outputs["assets/notice.txt"], b"Custom notice");
    assert_eq!(outputs["assets/static.txt"], b"Custom static data");
    let bound = snapshot(root)?;
    fs::write(web.join("pkg/generated-cache"), "Generated package data")?;
    fs::create_dir_all(web.join("dist"))?;
    fs::write(
        web.join("dist/generated-cache"),
        "Generated distribution data",
    )?;
    assert_eq!(bound, snapshot(root)?);
    Ok(())
}
