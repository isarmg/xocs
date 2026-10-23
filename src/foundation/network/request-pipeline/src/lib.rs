//! Blog's service/layer contracts, composition and readiness adapters.
//! Request admission and deadlines belong to the application's bounded runtime;
//! this package does not provide retries, background queues or load balancing.
#[macro_use]
pub(crate) mod macros;
pub mod builder;
pub mod layer;
#[cfg(feature = "make")]
pub mod make;
#[cfg(feature = "util")]
pub mod util;
#[cfg(feature = "util")]
pub use self::util::{service_fn, ServiceExt};
pub use crate::builder::ServiceBuilder;
#[cfg(feature = "make")]
pub use crate::make::MakeService;
mod layer_core;
pub use layer_core::{layer_fn, Identity, Layer, LayerFn, Stack};
mod service;
pub use service::Service;
#[allow(unreachable_pub)]
#[cfg(feature = "make")]
mod sealed {
    pub trait Sealed<T> {}
}
/// A type-erased service error.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
