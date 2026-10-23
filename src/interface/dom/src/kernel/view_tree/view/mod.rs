use std::{cell::RefCell, rc::Rc};

/// Add attributes to typed views.
pub mod add_attr;
/// A typed-erased view type.
pub mod any_view;
/// Allows choosing between one of several views.
pub mod either;
/// View rendering for `Result<_, _>` types.
pub mod error_boundary;
/// A type-erased view collection.
pub mod fragment;
/// View implementations for several iterable types.
pub mod iterators;
/// Keyed list iteration.
pub mod keyed;
mod primitives;

/// View implementation for string types.
pub mod strings;
/// View implementations for tuples.
pub mod tuples;

/// The `Render` trait allows rendering something as part of the user interface.
pub trait Render: Sized {
    /// The “view state” for this type, which can be retained between updates.
    ///
    /// For example, for a text node, `State` might be the actual DOM text node
    /// and the previous string, to allow for diffing between updates.
    type State: Mountable;

    /// Creates fresh DOM nodes for this view.
    fn build(self) -> Self::State;

    /// Updates the view with new data.
    fn rebuild(self, state: &mut Self::State);
}

/// A CSR view that can own its input.
///
/// DOM construction and updates live in [`Render`]. Ownership conversion lets
/// borrowed strings and other view inputs enter a long-lived type-erased view.
pub trait DomView
where
    Self: Render + Send,
{
    /// An equivalent value that is `'static`.
    type Owned: DomView + 'static;

    /// Whether this should actually exist in the DOM, if it is the child of an element.
    const EXISTS: bool = true;

    /// Convert into the equivalent value that is `'static`.
    fn into_owned(self) -> Self::Owned;
}

/// Allows a type to be mounted to the DOM.
pub trait Mountable {
    /// Detaches the view from the DOM.
    fn unmount(&mut self);

    /// Mounts a node to the interface.
    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    );

    /// Mounts a node to the interface. Returns `false` if it could not be mounted.
    fn try_mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) -> bool {
        self.mount(parent, marker);
        true
    }

    /// Inserts another `Mountable` type before this one. Returns `false` if
    /// this does not actually exist in the UI (for example, `()`).
    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool;

    /// Inserts another `Mountable` type before this one, or before the marker
    /// if this one doesn't exist in the UI (for example, `()`).
    fn insert_before_this_or_marker(
        &self,
        parent: &crate::view_tree::renderer::types::Element,
        child: &mut dyn Mountable,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        if !self.insert_before_this(child) {
            child.mount(parent, marker);
        }
    }

    /// wip
    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element>;
}

/// Indicates where a node should be mounted to its parent.
pub enum MountKind {
    /// Node should be mounted before this marker node.
    Before(crate::view_tree::renderer::types::Node),
    /// Node should be appended to the parent’s children.
    Append,
}

impl<T> Mountable for Option<T>
where
    T: Mountable,
{
    fn unmount(&mut self) {
        if let Some(ref mut mounted) = self {
            mounted.unmount()
        }
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        if let Some(ref mut inner) = self {
            inner.mount(parent, marker);
        }
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        self.as_ref()
            .map(|inner| inner.insert_before_this(child))
            .unwrap_or(false)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        self.as_ref()
            .map(|inner| inner.elements())
            .unwrap_or_default()
    }
}

impl<T> Mountable for Rc<RefCell<T>>
where
    T: Mountable,
{
    fn unmount(&mut self) {
        self.borrow_mut().unmount()
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        self.borrow_mut().mount(parent, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        self.borrow().insert_before_this(child)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        self.borrow().elements()
    }
}

/// Declares that this type can be converted into some other type, which can be rendered.
pub trait IntoRender {
    /// The renderable type into which this type can be converted.
    type Output;

    /// Consumes this value, transforming it into the renderable type.
    fn into_render(self) -> Self::Output;
}

impl<T> IntoRender for T
where
    T: Render,
{
    type Output = Self;

    fn into_render(self) -> Self::Output {
        self
    }
}
