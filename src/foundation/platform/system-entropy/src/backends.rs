//! Product entropy sources: Linux kernel RNG and browser Web Crypto.

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
mod linux;
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
mod use_file;
#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
pub use linux::*;

#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "wasm_js"))]
mod wasm_js;
#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "wasm_js"))]
pub use wasm_js::*;

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    not(feature = "wasm_js")
))]
compile_error!("browser entropy requires the wasm_js feature");
#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"),
    all(target_arch = "wasm32", target_os = "unknown")
)))]
compile_error!("entropy supports Linux amd64 GNU and browser wasm32 only");
