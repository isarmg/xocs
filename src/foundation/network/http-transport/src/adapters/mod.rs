// Preserve the imported adapter module's original lint scope; transport core remains strict.
#![allow(missing_debug_implementations)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Runtime, service and pooled HTTP/1 adapters maintained for Blog.
//! System proxies, alternative pool APIs and separate graceful-server support
//! are not part of this implementation. The application owns shutdown.

#[cfg(feature = "adapter-client")]
pub mod client;
mod common;
pub mod rt;
#[cfg(feature = "adapter-server")]
pub mod server;
#[cfg(any(feature = "service-adapter", feature = "client-legacy"))]
pub mod service;
