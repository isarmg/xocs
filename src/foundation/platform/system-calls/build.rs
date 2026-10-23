//! The product uses Rust 1.99 and Linux amd64 raw syscalls.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    assert_eq!(
        std::env::var("TARGET").as_deref(),
        Ok("x86_64-unknown-linux-gnu"),
        "blog-platform-system-calls supports only x86_64-unknown-linux-gnu"
    );
    for cfg in [
        "linux_raw_dep",
        "linux_raw",
        "linux_like",
        "linux_kernel",
        "lower_upper_exp_for_non_zero",
        "rustc_diagnostics",
        "error_in_core",
    ] {
        println!("cargo:rustc-cfg={cfg}");
    }
}
