//! Build script for memory sanitization support

fn main() {
    assert!(
        std::env::var_os("CARGO_CFG_GETRANDOM_BACKEND").is_none(),
        "custom entropy backends are outside the product contract"
    );
    // Automatically detect cfg(sanitize = "memory") even if cfg(sanitize) isn't
    // supported. Build scripts get cfg() info, even if the cfg is unstable.
    println!("cargo:rerun-if-changed=build.rs");
    let sanitizers = std::env::var("CARGO_CFG_SANITIZE").unwrap_or_default();
    if sanitizers.contains("memory") {
        println!("cargo:rustc-cfg=getrandom_msan");
    }
}
