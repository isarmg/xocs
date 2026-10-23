#[path = "src/platform_contract.rs"]
mod platform_contract;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn physical(path: &Path) {
    for ancestor in path.ancestors() {
        assert!(
            !fs::symlink_metadata(ancestor)
                .unwrap()
                .file_type()
                .is_symlink(),
            "Symlink build input: {}",
            ancestor.display()
        );
    }
}

fn collect(repository: &Path, directory: &Path, files: &mut BTreeMap<String, String>) {
    if !directory.exists() {
        return;
    }
    physical(directory);
    let mut entries: Vec<_> = fs::read_dir(directory)
        .unwrap()
        .map(Result::unwrap)
        .collect();
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
        let kind = entry.file_type().unwrap();
        assert!(
            !kind.is_symlink(),
            "Symlink build input: {}",
            path.display()
        );
        if kind.is_dir() {
            collect(repository, &path, files);
        } else {
            let info = entry.metadata().unwrap();
            assert!(
                kind.is_file() && info.len() <= 32 * 1024 * 1024,
                "Unbounded/non-regular build input: {}",
                path.display()
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                assert_eq!(
                    info.nlink(),
                    1,
                    "Hard linked build input: {}",
                    path.display()
                );
            }
            let bytes = fs::read(&path).unwrap();
            assert!(
                bytes.len() <= 32 * 1024 * 1024,
                "Build input grew beyond bound"
            );
            let name = path
                .strip_prefix(repository)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            files.insert(
                name,
                Sha256::digest(bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
            );
            assert!(files.len() <= 131072, "Foundation file count exceeds bound");
        }
    }
}

fn main() {
    platform_contract::check();
    let package = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let repository = package.parent().unwrap().parent().unwrap();
    let mut files = BTreeMap::new();
    // Also bind relocated include_str! documentation and preserved notices.
    for name in [
        "src/foundation",
        "docs/dependencies",
        "test/foundation",
        "compliance/licenses/upstream",
    ] {
        let path = repository.join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        collect(repository, &path, &mut files);
    }
    let mut digest = Sha256::new();
    for (name, hash) in files {
        digest.update(name.as_bytes());
        digest.update([0]);
        digest.update(hash.as_bytes());
        digest.update(b"\n");
    }
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("foundation-inputs.rs");
    fs::write(
        output,
        format!(
            "\"{}\"\n",
            digest
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        ),
    )
    .unwrap();
}
