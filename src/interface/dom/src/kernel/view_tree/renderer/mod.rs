/// A DOM renderer.
pub mod dom;

/// Browser DOM operations used throughout the CSR view tree.
pub type Rndr = dom::Dom;

/// Types used by the renderer.
///
/// See [`Rndr`] for additional information on this rendering approach.
pub mod types {
    pub use super::dom::{ClassList, CssStyleDeclaration, Element, Event, Node, Placeholder, Text};
}

/// A function that can be called to remove an event handler from an element after it has been added.
#[must_use = "This will invalidate the event handler when it is dropped. You \
              should store it in some other data structure to clean it up \
              later to avoid dropping it immediately, or leak it with \
              std::mem::forget() to never drop it."]
#[allow(clippy::type_complexity)]
pub struct RemoveEventHandler(Option<Box<dyn FnOnce() + Send + Sync>>);

impl RemoveEventHandler {
    /// Creates a new container with a function that will be called when it is dropped.
    pub(crate) fn new(remove: impl FnOnce() + Send + Sync + 'static) -> Self {
        Self(Some(Box::new(remove)))
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn into_inner(mut self) -> Option<Box<dyn FnOnce() + Send + Sync>> {
        self.0.take()
    }
}

impl Drop for RemoveEventHandler {
    fn drop(&mut self) {
        if let Some(cb) = self.0.take() {
            cb()
        }
    }
}

/// Attempts to cast from one type to another.
///
/// This works in a similar way to `TryFrom`. We implement it as a separate trait
/// simply so we don't have to create wrappers for the `web_sys` types; it can't be
/// implemented on them directly because of the orphan rules.
pub trait CastFrom<T>
where
    Self: Sized,
{
    /// Casts a node from one type to another.
    fn cast_from(source: T) -> Option<Self>;
}
