//! HTTP routing, request extraction and response conversion for Blog.
//!
//! [`Router`] combines method routes and nested routers. Handlers are async
//! functions whose arguments implement [`FromRequestParts`] or [`FromRequest`],
//! and whose return values implement [`IntoResponse`]. Request-parts extractors
//! run before the final extractor consumes the body.
//!
//! # Routes and state
//!
//! ```
//! use http_routing::{extract::State, routing::get, Router};
//!
//! #[derive(Clone)]
//! struct AppState { title: String }
//!
//! async fn title(State(state): State<AppState>) -> String {
//!     state.title
//! }
//!
//! let app: Router = Router::new()
//!     .route("/title", get(title))
//!     .with_state(AppState { title: "Blog".into() });
//! ```
//!
//! Use [`Router::merge`] to combine routers and [`Router::nest`] to mount routes
//! under a prefix. State types are checked at compile time. Shared mutable state
//! still needs the synchronization appropriate to its actual access pattern.
//!
//! # Middleware and errors
//!
//! The local [`service_stack::Service`] and [`service_stack::Layer`] traits form
//! the middleware boundary. See [`middleware`] for request/response adapters and
//! [`error_handling`] for converting fallible service errors into responses.
//! Capacity limits and request deadlines are application responsibilities.
//!
//! # Body handling
//!
//! [`Json`] and [`Form`] decode bodies through the shared data model and codecs.
//! [`DefaultBodyLimit`] configures the limit used by body-buffering extractors;
//! custom streaming extractors must enforce their own limits. See [`extract`]
//! for ordering, custom extractors and rejection handling, and [`response`] for
//! response composition.
//!
//! # Features
//!
//! The current default features are `form`, `http1`, `json`, `matched-path`,
//! `original-uri`, `query`, `tokio`, `tower-log` and `tracing`.
//! `http1` and `tokio` enable the local transport and Linux runtime integration;
//! `matched-path` and `original-uri` expose routing metadata extractors.
//! Codec features select the corresponding request extractors.
//!
//! The native target is AMD64 Linux GNU/glibc. Dependencies are local Cargo
//! paths; the package's Cargo name is `blog-network-request-router`, and its
//! Rust library name is `http_routing`.
//!
//! [`Router`]: crate::Router
//! [`FromRequest`]: crate::extract::FromRequest
//! [`FromRequestParts`]: crate::extract::FromRequestParts
//! [`IntoResponse`]: crate::response::IntoResponse
//! [`DefaultBodyLimit`]: crate::extract::DefaultBodyLimit
//! [`Json`]: crate::Json
//! [`Form`]: crate::Form
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]

#[doc(hidden)]
pub mod kernel;

#[macro_use]
pub(crate) mod macros;

mod boxed;
mod extension;
#[cfg(feature = "form")]
mod form;
#[cfg(feature = "json")]
mod json;
mod service_ext;
mod util;

pub mod body;
pub mod error_handling;
pub mod extract;
pub mod handler;
pub mod middleware;
pub mod response;
pub mod routing;
#[cfg(all(feature = "tokio", feature = "http1"))]
pub mod serve;

#[doc(no_inline)]
pub use http;

#[doc(inline)]
pub use self::extension::Extension;
#[doc(inline)]
#[cfg(feature = "json")]
pub use self::json::Json;
#[doc(inline)]
pub use self::routing::Router;

#[doc(inline)]
#[cfg(feature = "form")]
pub use self::form::Form;

#[doc(inline)]
pub use crate::kernel::{BoxError, Error, RequestExt, RequestPartsExt};

#[cfg(all(feature = "tokio", feature = "http1"))]
#[doc(inline)]
pub use self::serve::serve;

pub use self::service_ext::ServiceExt;

#[cfg(test)]
use axum_macros::__private_axum_test as test;
