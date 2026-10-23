#![deny(warnings)]

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH was not set");
    println!("cargo:rustc-check-cfg=cfg(folded_multiply)");
    if arch.eq_ignore_ascii_case("x86_64") {
        println!("cargo:rustc-cfg=folded_multiply");
    }
}
