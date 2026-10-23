//! Raw FFI bindings for Linux AMD64 with the GNU/glibc ABI.
//!
//! The maintained native target is `x86_64-unknown-linux-gnu` with 64-bit pointers. The browser
//! `wasm32-unknown-unknown` target exposes an empty ABI surface. These bindings use the repository's
//! pinned Rust toolchain and follow current consumers; upstream libc's platform matrix and release
//! compatibility policy are not support promises for this package.
//!
//! # API documentation
//!
//! This crate defines types, constants, and raw function declarations. Calling an unsafe function
//! requires satisfying the underlying C API's pointer, lifetime, ownership, and threading rules.
//! Refer to the [Linux manual pages](https://man7.org/linux/man-pages/index.html) and the
//! [glibc manual](https://sourceware.org/glibc/manual/latest/html_mono/libc.html), or use
//! `man 2 NAME` for kernel interfaces and `man 3 NAME` for C library interfaces.
//!
//! # Usage guidelines
//!
//! - Use the declared ABI types and constants in FFI calls. Pointer width, alignment, field
//!   offsets, and argument types must match the Linux AMD64 ABI.
//! - Do not assume a C function initializes every byte or field of an output struct. Keep
//!   partially initialized outputs behind `MaybeUninit` and read only fields the API guarantees
//!   to initialize. Zero-initialization is appropriate only where the C API accepts it.
//! - Avoid accessing reserved and padding fields, or naming generated `__c_anonymous_*` helper
//!   types in consumer APIs.
//! - Native ABI regression tests compare maintained layouts and constants with the host headers;
//!   they do not make unsafe calls safe or guarantee the availability of every declared symbol.
//!
//! # Cargo features
//!
//! `std` is enabled by default and relies on the standard library's system link directives.
//! Without it, the native bindings provide their link directives. `extra_traits` adds equality,
//! hashing, and additional debug implementations for supported structures. The remaining `align`,
//! `const-extern-fn`, and `use_std` feature names are inherited manifest entries, not additional
//! supported platforms or a compatibility policy.
//!
//! Original libc source, documentation, and licensing are retained separately under
//! `docs/upstream` and `compliance`.

// Make it a bit easier to build without Cargo
#![crate_name = "blog_platform_system_abi"]
#![crate_type = "rlib"]
// Pretty much all C API doesn't match Rust conventions.
#![allow(nonstandard_style)]
// Not all macros and all patterns are used on all targets.
#![allow(unused_macros)]
#![allow(unused_macro_rules)]
// All traits should be `Copy` and `Debug`.
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
// Downgrade deny to a warning.
#![warn(overflowing_literals)]
// Prepare for a future upgrade.
#![warn(rust_2024_compatibility)]
// Things missing for 2024 that are blocked on MSRV or breakage.
#![allow(missing_unsafe_on_extern)]
#![allow(edition_2024_expr_fragment_specifier)]
// Allowed globally, the warning is enabled in individual modules as we work through them
#![allow(unsafe_op_in_unsafe_fn)]
#![cfg_attr(libc_deny_warnings, deny(warnings))]
// Attributes needed when building as part of the standard library
#![cfg_attr(feature = "rustc-dep-of-std", feature(link_cfg, no_core))]
#![cfg_attr(feature = "rustc-dep-of-std", allow(internal_features))]
// Some targets don't need `link_cfg` and emit a warning.
#![cfg_attr(feature = "rustc-dep-of-std", allow(unused_features))]
// DIFF(1.0): The thread local references that raise this lint were removed in 1.0
#![cfg_attr(feature = "rustc-dep-of-std", allow(static_mut_refs))]
#![cfg_attr(not(feature = "rustc-dep-of-std"), no_std)]
#![cfg_attr(feature = "rustc-dep-of-std", no_core)]

#[macro_use]
mod macros;
#[cfg(target_os = "linux")]
mod new;

cfg_if! {
    if #[cfg(feature = "rustc-dep-of-std")] {
        extern crate rustc_std_workspace_core as core;
    }
}

pub use core::ffi::c_void;

#[allow(unused_imports)] // needed while the module is empty on some platforms
#[cfg(target_os = "linux")]
pub use new::*;

#[cfg(not(any(
    all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu",
        target_pointer_width = "64"
    ),
    all(target_arch = "wasm32", target_os = "unknown")
)))]
compile_error!("system ABI supports Linux amd64 GNU and the browser wasm32 empty ABI only");

#[cfg(target_os = "linux")]
mod primitives;
#[cfg(target_os = "linux")]
pub use crate::primitives::*;
#[cfg(target_os = "linux")]
mod unix;
#[cfg(target_os = "linux")]
pub use crate::unix::*;
#[cfg(target_os = "linux")]
prelude!();
