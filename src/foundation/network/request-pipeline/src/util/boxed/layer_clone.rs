use crate::util::BoxCloneService;
use std::{fmt, sync::Arc};
use crate::{layer_fn, Layer};
use crate::Service;

/// A [`Clone`] + [`Send`] boxed [`Layer`].
///
/// [`BoxCloneServiceLayer`] turns a layer into a trait object, allowing both the [`Layer`] itself
/// and the output [`Service`] to be dynamic, while having consistent types.
///
/// This [`Layer`] produces [`BoxCloneService`] instances erasing the type of the
/// [`Service`] produced by the wrapped [`Layer`].
///
/// This is similar to [`BoxLayer`](super::BoxLayer) except the layer and resulting
/// service implements [`Clone`].
///
/// # Example
///
/// Box a layer when callers need a consistent service type:
///
/// ```
/// use service_stack::{Identity, Service, util::BoxCloneServiceLayer};
///
/// fn boxed<S, R>() -> BoxCloneServiceLayer<S, R, S::Response, S::Error>
/// where
///     S: Service<R> + Send + Clone + 'static,
///     S::Future: Send + 'static,
/// {
///     BoxCloneServiceLayer::new(Identity::new())
/// }
/// ```
pub struct BoxCloneServiceLayer<In, T, U, E> {
    boxed: Arc<dyn Layer<In, Service = BoxCloneService<T, U, E>> + Send + Sync + 'static>,
}

impl<In, T, U, E> BoxCloneServiceLayer<In, T, U, E> {
    /// Create a new [`BoxCloneServiceLayer`].
    pub fn new<L>(inner_layer: L) -> Self
    where
        L: Layer<In> + Send + Sync + 'static,
        L::Service: Service<T, Response = U, Error = E> + Send + Clone + 'static,
        <L::Service as Service<T>>::Future: Send + 'static,
    {
        let layer = layer_fn(move |inner: In| {
            let out = inner_layer.layer(inner);
            BoxCloneService::new(out)
        });

        Self {
            boxed: Arc::new(layer),
        }
    }
}

impl<In, T, U, E> Layer<In> for BoxCloneServiceLayer<In, T, U, E> {
    type Service = BoxCloneService<T, U, E>;

    fn layer(&self, inner: In) -> Self::Service {
        self.boxed.layer(inner)
    }
}

impl<In, T, U, E> Clone for BoxCloneServiceLayer<In, T, U, E> {
    fn clone(&self) -> Self {
        Self {
            boxed: Arc::clone(&self.boxed),
        }
    }
}

impl<In, T, U, E> fmt::Debug for BoxCloneServiceLayer<In, T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BoxCloneServiceLayer").finish()
    }
}
