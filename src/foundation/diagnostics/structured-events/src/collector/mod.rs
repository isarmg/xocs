//! Composable collectors for tracing spans and events.
//!
//! A [`Layer` trait](crate::collector::layer::Layer) adds one collection behavior to a
//! [`Subscriber` trait](crate::primitives::Subscriber). Multiple layers can share a
//! registry and apply either global or per-layer filtering. See
//! [layer composition](crate::collector::layer#composing-layers) and
//! [filtering](crate::collector::layer#per-layer-filtering).
//!
//! Target-prefix and level filtering is provided by
//! [`Targets`](crate::collector::filter::Targets) and
//! [`LevelFilter`](crate::collector::filter::LevelFilter). Filters do not parse
//! span-field expressions.
//!
//! ## Features
//!
//! The `collector` feature enables this module. Additional capabilities are
//! selected explicitly:
//!
//! - `registry` adds span storage and lookup, and requires `std`.
//! - `fmt` adds formatting subscribers and requires `registry` and `std`.
//! - `ansi` adds terminal colors and the pretty formatter.
//! - `json` adds newline-delimited JSON formatting.
//! - `smallvec` uses inline storage for span data.
//! - `parking` selects the repository's synchronization provider.
//!
//! `collector-default` selects formatting, ANSI output, inline span storage and
//! the log tracer. The `log-bridge` feature converts log records into events;
//! `log-tracer` also exposes the installable logger.

#![cfg_attr(
    docsrs,
    // Allows displaying cfgs/feature flags in the documentation.
    feature(doc_cfg),
    // Allows adding traits to RustDoc's list of "notable traits"
    feature(doc_notable_trait),
    // Fail the docs build if any intra-docs links are broken
    deny(rustdoc::broken_intra_doc_links),
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_interfaces,
    private_bounds,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
// Using struct update syntax when a struct has no additional fields avoids
// a potential source change if additional fields are added to the struct in the
// future, reducing diff noise. Allow this even though clippy considers it
// "needless".
#![allow(clippy::needless_update)]

#[macro_use]
mod macros;

pub mod field;
pub mod filter;
pub mod prelude;
pub mod registry;

pub mod layer;
pub mod util;

feature! {
    #![feature = "std"]
    pub mod reload;
    pub(crate) mod sync;
}

feature! {
    #![all(feature = "fmt", feature = "std")]
    pub mod fmt;
    pub use fmt::fmt;
    pub use fmt::Subscriber as FmtSubscriber;
}

feature! {
    #![all(any(), feature = "std")]
    pub use filter::EnvFilter;
}

pub use layer::Layer;

feature! {
    #![all(feature = "registry", feature = "std")]
    pub use registry::Registry;

    /// Returns a default [`Registry`].
    pub fn registry() -> Registry {
        Registry::default()
    }
}

mod sealed {
    pub trait Sealed<A = ()> {}
}
