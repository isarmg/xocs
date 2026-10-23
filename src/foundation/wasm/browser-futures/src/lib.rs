//! Bridging the gap between Rust Futures and JavaScript Promises.
//!
//! Browser executor and Promise bridge used by the CSR runtime.
//! The implementation lives in [`js_sys::futures`] so [`js_sys::Promise`]
//! can implement [`core::future::IntoFuture`] directly.

#![cfg_attr(not(feature = "std"), no_std)]

pub use js_sys::futures::{future_to_promise, future_to_promise_typed, spawn_local, JsFuture};

#[cfg(not(target_feature = "atomics"))]
#[allow(deprecated)]
pub use js_sys::futures::jspi_block_on_promise;

pub use js_sys;
pub use wasm_bindgen;
