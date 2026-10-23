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
