//! Build-time platform contract shared by the product entry points.
//!
//! This file is also a standalone Cargo build script. Keep it std-only so that
//! checking the platform never depends on compiling the asset builder itself.

use std::{env, path::Path};

pub const NATIVE_TARGET: &str = "x86_64-unknown-linux-gnu";
pub const BROWSER_TARGET: &str = "wasm32-unknown-unknown";

fn validate(host: &str, target: &str, package: &str) -> Result<(), String> {
    if host != NATIVE_TARGET {
        return Err(format!(
            "unsupported build host {host:?}; {package} requires {NATIVE_TARGET}"
        ));
    }
    // These libraries also compile natively for domain and kernel tests. All
    // other entry points and build tools are native-only; proc macros are built
    // for HOST even when their consumers target the browser.
    let browser = matches!(package, "blog-browser" | "blog-interface" | "blog-dom");
    if target != NATIVE_TARGET && !(browser && target == BROWSER_TARGET) {
        let supported = if browser {
            format!("{NATIVE_TARGET} (native tests) or {BROWSER_TARGET} (browser)")
        } else {
            NATIVE_TARGET.to_owned()
        };
        return Err(format!(
            "unsupported build target {target:?}; {package} supports only {supported}"
        ));
    }
    Ok(())
}

pub fn check() {
    let manifest = env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory");
    let source = Path::new(file!());
    let source = if source.is_absolute() {
        source.to_owned()
    } else {
        Path::new(&manifest).join(source)
    };
    println!("cargo:rerun-if-changed={}", source.display());
    for variable in ["HOST", "TARGET", "CARGO_PKG_NAME"] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    let host = env::var("HOST").expect("Cargo build host");
    let target = env::var("TARGET").expect("Cargo build target");
    let package = env::var("CARGO_PKG_NAME").expect("Cargo package name");
    validate(&host, &target, &package).unwrap_or_else(|error| panic!("{error}"));
}

#[allow(dead_code)] // Used directly as build.rs or included by an existing build script.
fn main() {
    check();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_entry_points_and_browser_test_libraries_use_linux_amd64_gnu() {
        for package in [
            "blog",
            "blog-application",
            "blog-setup",
            "blog-assets",
            "blog-browser",
            "blog-interface",
            "blog-dom",
            "blog-component-macros",
            "blog-browser-domain-tests",
        ] {
            assert_eq!(validate(NATIVE_TARGET, NATIVE_TARGET, package), Ok(()));
        }
    }

    #[test]
    fn wasm_is_an_output_for_browser_libraries_only() {
        for package in ["blog-browser", "blog-interface", "blog-dom"] {
            assert_eq!(validate(NATIVE_TARGET, BROWSER_TARGET, package), Ok(()));
        }
        for package in [
            "blog",
            "blog-application",
            "blog-setup",
            "blog-assets",
            "blog-component-macros",
        ] {
            assert!(validate(NATIVE_TARGET, BROWSER_TARGET, package).is_err());
        }
    }

    #[test]
    fn other_architectures_operating_systems_and_abis_are_rejected() {
        for target in [
            "aarch64-unknown-linux-gnu",
            "i686-unknown-linux-gnu",
            "x86_64-unknown-linux-gnux32",
            "x86_64-unknown-linux-musl",
            "x86_64-pc-windows-gnu",
            "x86_64-apple-darwin",
            "wasm32-wasip1",
        ] {
            assert!(
                validate(NATIVE_TARGET, target, "blog-browser").is_err(),
                "{target}"
            );
        }
    }

    #[test]
    fn a_supported_target_does_not_hide_an_unsupported_host() {
        for host in [
            "aarch64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "x86_64-unknown-linux-gnux32",
        ] {
            assert!(validate(host, NATIVE_TARGET, "blog").is_err());
            assert!(validate(host, BROWSER_TARGET, "blog-browser").is_err());
        }
    }
}
