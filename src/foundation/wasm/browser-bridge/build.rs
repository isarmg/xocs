use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // The repository uses Rust 1.99; diagnostic attributes are always available.
    println!("cargo:rustc-check-cfg=cfg(wbg_diagnostic)");
    println!("cargo:rustc-cfg=wbg_diagnostic");

    let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    println!("cargo:rustc-check-cfg=cfg(wbg_reference_types)");
    if target_features
        .split(',')
        .any(|feature| feature.trim() == "reference-types")
    {
        println!("cargo:rustc-cfg=wbg_reference_types");
    }
}
