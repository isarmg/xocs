#![allow(
    clippy::cognitive_complexity,
    clippy::large_enum_variant,
    clippy::module_inception,
    clippy::needless_doctest_main
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![deny(unused_must_use, unsafe_op_in_unsafe_fn)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
// loom is an internal implementation detail.
// Do not show "Available on non-loom only" label
#![cfg_attr(docsrs, doc(auto_cfg(hide(loom))))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![cfg_attr(loom, allow(dead_code, unreachable_pub))]
#![cfg_attr(windows, allow(rustdoc::broken_intra_doc_links))]

//! Async task scheduling, I/O, timers and synchronization for the local product.
//!
//! Native consumers run on AMD64 Linux GNU/glibc. The I/O driver uses Linux
//! `epoll` through the local poller; no registry download or external runtime
//! implementation is selected at build time. Browser consumers use only the
//! features enabled by the WASM application graph.
//!
//! # Feature selection
//!
//! The default feature set is empty. Consumers enable the capabilities they use:
//!
//! | Feature | Capability |
//! | --- | --- |
//! | `rt` | Task spawning and the current-thread runtime |
//! | `rt-multi-thread` | Multi-threaded, work-stealing scheduler; includes `rt` |
//! | `time` | Timers, sleeps and timeouts |
//! | `net` | TCP, UDP, Unix sockets and readiness registration |
//! | `fs` | Asynchronous filesystem operations |
//! | `io-util` | Buffered I/O and extension traits |
//! | `io-std` | Asynchronous standard input and output |
//! | `sync` | Synchronization primitives and channels |
//! | `signal` | Linux signal handling |
//! | `macros` | Runtime entry and test attributes |
//! | `test-util` | Runtime testing helpers; includes `rt`, `sync` and `time` |
//!
//! `AsyncRead` and `AsyncWrite` are always available in the `io` module. Runtime
//! and operating-system services are available only with their corresponding
//! features; the product's Cargo manifests define the actual combinations.
//!
//! # Blocking work
//!
//! Polling a future must not block the worker thread. Move synchronous file,
//! password and CPU work to a blocking worker with an explicit application
//! concurrency limit. The runtime's blocking queue does not provide that limit.
//! Resources tied to I/O or timers require a running runtime with the relevant
//! driver enabled.
//!
//! # Source and API identity
//!
//! This package is `blog-runtime-task-scheduler`; consumers may select a local
//! dependency alias such as `tokio`. Source, dependency mapping and licenses are
//! stored in this repository. The API pages below describe the selected features.

// Test that pointer width is compatible. This asserts that e.g. usize is at
// least 32 bits, which a lot of components in Tokio currently assumes.
//
// TODO: improve once we have MSRV access to const eval to make more flexible.
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error! {
    "Tokio requires the platform pointer width to be at least 32 bits"
}

#[cfg(all(
    not(tokio_unstable),
    target_family = "wasm",
    any(
        feature = "fs",
        feature = "io-std",
        feature = "net",
        feature = "rt-multi-thread",
        feature = "signal"
    )
))]
compile_error!("Only features sync,macros,io-util,rt,time are supported on wasm.");

#[cfg(all(not(tokio_unstable), feature = "schedule-latency"))]
compile_error!("The `schedule-latency` feature requires `--cfg tokio_unstable`.");

#[cfg(all(
    feature = "schedule-latency",
    not(all(target_pointer_width = "64", target_has_atomic = "64"))
))]
compile_error!("The `schedule-latency` feature is only currently supported on 64-bit targets.");

// Includes re-exports used by macros.
//
// This module is not intended to be part of the public API. In general, any
// `doc(hidden)` code is not part of Tokio's public and stable API.
#[macro_use]
#[doc(hidden)]
pub mod macros;

cfg_fs! {
    pub mod fs;
}

mod future;

pub mod io;
pub mod net;

mod loom;

#[cfg(any(feature = "fs", feature = "io-std", feature = "net"))]
mod blocking;

cfg_rt! {
    pub mod runtime;
}
cfg_not_rt! {
    pub(crate) mod runtime;
}

cfg_signal! {
    pub mod signal;
}

cfg_signal_internal! {
    #[cfg(not(feature = "signal"))]
    #[allow(dead_code)]
    #[allow(unreachable_pub)]
    pub(crate) mod signal;
}

cfg_sync! {
    pub mod sync;
}
cfg_not_sync! {
    mod sync;
}

// Currently, task module does not expose any public API outside `rt`
// feature, so we mark it in the docs. This happens only to docs to
// avoid introducing breaking changes by restricting the visibility
// of the task module.
#[cfg_attr(docsrs, doc(cfg(feature = "rt")))]
pub mod task;
cfg_rt! {
    pub use task::spawn;
}

cfg_time! {
    pub mod time;
}

mod trace {

    #[inline(always)]
    #[allow(dead_code)]
    pub(crate) fn trace_leaf(_: &mut std::task::Context<'_>) -> std::task::Poll<()> {
        std::task::Poll::Ready(())
    }

    #[cfg_attr(not(feature = "sync"), allow(dead_code))]
    pub(crate) async fn async_trace_leaf() {
        std::future::poll_fn(trace_leaf).await
    }
}

mod util;

/// Due to the `Stream` trait's inclusion in `std` landing later than Tokio's 1.0
/// release, most of the Tokio stream utilities have been moved into the [`tokio-stream`]
/// crate.
///
/// # Why was `Stream` not included in Tokio 1.0?
///
/// Originally, we had planned to ship Tokio 1.0 with a stable `Stream` type
/// but unfortunately the [RFC] had not been merged in time for `Stream` to
/// reach `std` on a stable compiler in time for the 1.0 release of Tokio. For
/// this reason, the team has decided to move all `Stream` based utilities to
/// the [`tokio-stream`] crate. While this is not ideal, once `Stream` has made
/// it into the standard library and the `MSRV` period has passed, we will implement
/// stream for our different types.
///
/// While this may seem unfortunate, not all is lost as you can get much of the
/// `Stream` support with `async/await` and `while let` loops. It is also possible
/// to create a `impl Stream` from `async fn` using the [`async-stream`] crate.
///
/// [`tokio-stream`]: https://docs.rs/tokio-stream
/// [`async-stream`]: https://docs.rs/async-stream
/// [RFC]: https://github.com/rust-lang/rfcs/pull/2996
///
/// # Example
///
/// Convert a [`sync::mpsc::Receiver`] to an `impl Stream`.
///
/// ```rust,no_run
/// use tokio::sync::mpsc;
///
/// let (tx, mut rx) = mpsc::channel::<usize>(16);
///
/// let stream = async_stream::stream! {
///     while let Some(item) = rx.recv().await {
///         yield item;
///     }
/// };
/// ```
pub mod stream {}

// local re-exports of platform specific things, allowing for decent
// documentation to be shimmed in on docs.rs

#[cfg(all(docsrs, unix))]
pub mod doc;

#[cfg(any(feature = "net", feature = "fs"))]
#[cfg(all(docsrs, unix))]
#[allow(unused)]
pub(crate) use self::doc::os;

#[cfg(not(all(docsrs, unix)))]
#[allow(unused)]
pub(crate) use std::os;

cfg_macros! {
    /// Implementation detail of the `select!` macro. This macro is **not**
    /// intended to be used as part of the public API and is permitted to
    /// change.
    #[doc(hidden)]
    pub use tokio_macros::select_priv_declare_output_enum;

    /// Implementation detail of the `select!` macro. This macro is **not**
    /// intended to be used as part of the public API and is permitted to
    /// change.
    #[doc(hidden)]
    pub use tokio_macros::select_priv_clean_pattern;

    cfg_rt! {
        #[cfg(feature = "rt-multi-thread")]
        #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
        #[doc(inline)]
        pub use tokio_macros::main;

        #[cfg(feature = "rt-multi-thread")]
        #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
        #[doc(inline)]
        pub use tokio_macros::test;

        cfg_not_rt_multi_thread! {
            #[doc(inline)]
            pub use tokio_macros::main_rt as main;

            #[doc(inline)]
            pub use tokio_macros::test_rt as test;
        }
    }

    // Always fail if rt is not enabled.
    cfg_not_rt! {
        #[doc(inline)]
        pub use tokio_macros::main_fail as main;

        #[doc(inline)]
        pub use tokio_macros::test_fail as test;
    }
}

// TODO: rm
#[cfg(test)]
include!("../../../../../test/foundation/runtime/task-scheduler/unit/src/lib/8458.rs");

/// fuzz test (`fuzz_linked_list`)
#[cfg(fuzzing)]
pub mod fuzz;
