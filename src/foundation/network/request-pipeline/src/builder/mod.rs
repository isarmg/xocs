//! Typed composition for the layers used by the Blog service stack.
use crate::{Identity, Layer, Stack};
use std::fmt;

/// Composes layers in request order without runtime allocation.
#[derive(Clone)]
pub struct ServiceBuilder<L> {
    layer: L,
}

impl Default for ServiceBuilder<Identity> {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceBuilder<Identity> {
    /// Starts a service stack with no layers.
    pub const fn new() -> Self {
        Self {
            layer: Identity::new(),
        }
    }
}

impl<L> ServiceBuilder<L> {
    /// Appends a layer inside the previously added layers.
    pub fn layer<T>(self, layer: T) -> ServiceBuilder<Stack<T, L>> {
        ServiceBuilder {
            layer: Stack::new(layer, self.layer),
        }
    }

    /// Adds a layer only when it is present.
    #[cfg(feature = "util")]
    pub fn option_layer<T>(
        self,
        layer: Option<T>,
    ) -> ServiceBuilder<Stack<crate::util::Either<T, Identity>, L>> {
        self.layer(crate::util::option_layer(layer))
    }

    /// Adds a layer created from a wrapping function.
    pub fn layer_fn<F>(self, f: F) -> ServiceBuilder<Stack<crate::LayerFn<F>, L>> {
        self.layer(crate::layer_fn(f))
    }

    /// Returns the composed layer.
    pub fn into_inner(self) -> L {
        self.layer
    }

    /// Wraps a service with the composed layer.
    pub fn service<S>(&self, service: S) -> L::Service
    where
        L: Layer<S>,
    {
        self.layer.layer(service)
    }

    /// Wraps an asynchronous function as the innermost service.
    #[cfg(feature = "util")]
    pub fn service_fn<F>(self, f: F) -> L::Service
    where
        L: Layer<crate::util::ServiceFn<F>>,
    {
        self.service(crate::util::service_fn(f))
    }
}

impl<L: fmt::Debug> fmt::Debug for ServiceBuilder<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ServiceBuilder").field(&self.layer).finish()
    }
}

impl<S, L> Layer<S> for ServiceBuilder<L>
where
    L: Layer<S>,
{
    type Service = L::Service;
    fn layer(&self, inner: S) -> Self::Service {
        self.layer.layer(inner)
    }
}
