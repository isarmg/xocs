//! Backends for `cmov`, selected at compile time.

// Native predication on the supported Linux amd64 target.
#[cfg(all(target_arch = "x86_64", not(miri)))]
mod x86;

// Portable arithmetic for browser WebAssembly and Miri.
#[cfg(any(not(target_arch = "x86_64"), miri))]
mod soft;
