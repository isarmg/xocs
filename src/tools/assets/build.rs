#![forbid(unsafe_code)]
#[path = "src/layout.rs"]
#[allow(dead_code)]
mod layout;
#[path = "src/platform_contract.rs"]
mod platform_contract;
#[path = "src/source_inputs.rs"]
#[allow(dead_code)]
mod source_inputs;

use std::{
    fs,
    path::{Path, PathBuf},
};

fn main() -> std::io::Result<()> {
    platform_contract::check();
    let package = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("invariant: Cargo supplies the build-script manifest directory"),
    );
    let repository = package
        .ancestors()
        .nth(Path::new(layout::BUILDER).components().count())
        .expect("Builder package must be in the repository layout");
    assert_eq!(repository.join(layout::BUILDER), package);
    let mut inputs = source_inputs::SnapshotFiles::default();
    let sources = package.join("src");
    println!("cargo:rerun-if-changed={}", sources.display());
    let mut builder = inputs.collect(
        repository,
        source_inputs::walk(&sources, source_inputs::MAX_VERIFICATION_FILES)?,
    )?;
    for name in ["Cargo.toml", "build.rs"] {
        let path = package.join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        builder.insert(
            source_inputs::relative(
                path.strip_prefix(repository)
                    .expect("invariant: builder inputs are inside the repository"),
            )?,
            inputs.hash(&path)?,
        );
    }
    let output = PathBuf::from(
        std::env::var_os("OUT_DIR")
            .expect("invariant: Cargo supplies the build-script output directory"),
    );
    let entries = builder
        .iter()
        .map(|(name, hash)| format!("({name:?}, {hash:?})"))
        .collect::<Vec<_>>()
        .join(",\n");
    fs::write(output.join("builder-inputs.rs"), format!("[{entries}]\n"))?;

    // Keep the existing conservative compiled implementation basis, including
    // relocated test/include inputs and documentation. Full source verification
    // uses the same rules; no source category is silently dropped here.
    for name in source_inputs::VERIFICATION_ROOTS {
        println!("cargo:rerun-if-changed={}", repository.join(name).display());
    }
    let compiled_implementation_basis = inputs.verification_inputs(repository)?;
    fs::write(
        output.join("foundation-inputs.rs"),
        format!(
            "\"{}\"\n",
            source_inputs::digest(&compiled_implementation_basis)
        ),
    )?;
    Ok(())
}
