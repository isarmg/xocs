use std::{fmt, sync::Arc};
use crate::{layer_fn, Layer};
use crate::Service;

use crate::util::BoxCloneSyncService;

/// A [`Clone`] + [`Send`] + [`Sync`] boxed [`Layer`].
///
/// [`BoxCloneSyncServiceLayer`] turns a layer into a trait object, allowing both the [`Layer`] itself
/// and the output [`Service`] to be dynamic, while having consistent types.
///
/// This [`Layer`] produces [`BoxCloneSyncService`] instances erasing the type of the
/// [`Service`] produced by the wrapped [`Layer`].
///
/// This is similar to [`BoxCloneServiceLayer`](super::BoxCloneServiceLayer) except the layer and resulting
/// service implements [`Sync`].
///
/// # Example
///
/// Box a layer when callers need a consistent service type:
///
/// ```
/// use service_stack::{Identity, Service, util::BoxCloneSyncServiceLayer};
///
/// fn boxed<S, R>() -> BoxCloneSyncServiceLayer<S, R, S::Response, S::Error>
/// where
///     S: Service<R> + Send + Sync + Clone + 'static,
///     S::Future: Send + 'static,
/// {
///     BoxCloneSyncServiceLayer::new(Identity::new())
/// }
/// ```
pub struct BoxCloneSyncServiceLayer<In, T, U, E> {
    boxed: Arc<dyn Layer<In, Service = BoxCloneSyncService<T, U, E>> + Send + Sync + 'static>,
}

impl<In, T, U, E> BoxCloneSyncServiceLayer<In, T, U, E> {
    /// Create a new [`BoxCloneSyncServiceLayer`].
    pub fn new<L>(inner_layer: L) -> Self
    where
        L: Layer<In> + Send + Sync + 'static,
        L::Service: Service<T, Response = U, Error = E> + Send + Sync + Clone + 'static,
        <L::Service as Service<T>>::Future: Send + 'static,
    {
        let layer = layer_fn(move |inner: In| {
            let out = inner_layer.layer(inner);
            BoxCloneSyncService::new(out)
        });

        Self {
            boxed: Arc::new(layer),
        }
    }
}

impl<In, T, U, E> Layer<In> for BoxCloneSyncServiceLayer<In, T, U, E> {
    type Service = BoxCloneSyncService<T, U, E>;

    fn layer(&self, inner: In) -> Self::Service {
        self.boxed.layer(inner)
    }
}

impl<In, T, U, E> Clone for BoxCloneSyncServiceLayer<In, T, U, E> {
    fn clone(&self) -> Self {
        Self {
            boxed: Arc::clone(&self.boxed),
        }
    }
}

impl<In, T, U, E> fmt::Debug for BoxCloneSyncServiceLayer<In, T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BoxCloneSyncServiceLayer").finish()
    }
}
