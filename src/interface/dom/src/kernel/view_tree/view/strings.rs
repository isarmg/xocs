use super::{DomView, Mountable, Render};
use crate::view_tree::{no_attrs, renderer::Rndr};
use std::{borrow::Cow, rc::Rc, sync::Arc};

no_attrs!(&'a str);
no_attrs!(String);
no_attrs!(Arc<str>);
no_attrs!(Cow<'a, str>);

/// Retained view state for `&str`.
pub struct StrState<'a> {
    pub(crate) node: crate::view_tree::renderer::types::Text,
    str: &'a str,
}

impl<'a> Render for &'a str {
    type State = StrState<'a>;

    fn build(self) -> Self::State {
        let node = Rndr::create_text_node(self);
        StrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let StrState { node, str } = state;
        if &self != str {
            Rndr::set_text(node, self);
            *str = self;
        }
    }
}

impl DomView for &str {
    type Owned = String;

    fn into_owned(self) -> Self::Owned {
        self.to_string()
    }
}

impl Mountable for StrState<'_> {
    fn unmount(&mut self) {
        self.node.unmount()
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        Rndr::insert_node(parent, self.node.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        self.node.insert_before_this(child)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}

/// Retained view state for `String`.
pub struct StringState {
    node: crate::view_tree::renderer::types::Text,
    str: String,
}

impl Render for String {
    type State = StringState;

    fn build(self) -> Self::State {
        let node = Rndr::create_text_node(&self);
        StringState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let StringState { node, str } = state;
        if &self != str {
            Rndr::set_text(node, &self);
            *str = self;
        }
    }
}

impl DomView for String {
    type Owned = Self;

    fn into_owned(self) -> Self::Owned {
        self
    }
}

impl Mountable for StringState {
    fn unmount(&mut self) {
        self.node.unmount()
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        Rndr::insert_node(parent, self.node.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        self.node.insert_before_this(child)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}

/// Retained view state for `Rc<str>`.
pub struct RcStrState {
    node: crate::view_tree::renderer::types::Text,
    str: Rc<str>,
}

impl Render for Rc<str> {
    type State = RcStrState;

    fn build(self) -> Self::State {
        let node = Rndr::create_text_node(&self);
        RcStrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let RcStrState { node, str } = state;
        if !Rc::ptr_eq(&self, str) {
            Rndr::set_text(node, &self);
            *str = self;
        }
    }
}

// can't Send an Rc<str> between threads, so can't implement async HTML rendering that might need
// to send it

impl Mountable for RcStrState {
    fn unmount(&mut self) {
        self.node.unmount()
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        Rndr::insert_node(parent, self.node.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        self.node.insert_before_this(child)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}

/// Retained view state for `Arc<str>`.
pub struct ArcStrState {
    node: crate::view_tree::renderer::types::Text,
    str: Arc<str>,
}

impl Render for Arc<str> {
    type State = ArcStrState;

    fn build(self) -> Self::State {
        let node = Rndr::create_text_node(&self);
        ArcStrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let ArcStrState { node, str } = state;
        if self != *str {
            Rndr::set_text(node, &self);
            *str = self;
        }
    }
}

impl DomView for Arc<str> {
    type Owned = Self;

    fn into_owned(self) -> Self::Owned {
        self
    }
}

impl Mountable for ArcStrState {
    fn unmount(&mut self) {
        self.node.unmount()
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        Rndr::insert_node(parent, self.node.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        self.node.insert_before_this(child)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}

/// Retained view state for `Cow<'_, str>`.
pub struct CowStrState<'a> {
    node: crate::view_tree::renderer::types::Text,
    str: Cow<'a, str>,
}

impl<'a> Render for Cow<'a, str> {
    type State = CowStrState<'a>;

    fn build(self) -> Self::State {
        let node = Rndr::create_text_node(&self);
        CowStrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let CowStrState { node, str } = state;
        if self != *str {
            Rndr::set_text(node, &self);
            *str = self;
        }
    }
}

impl DomView for Cow<'_, str> {
    type Owned = String;

    fn into_owned(self) -> <Self as DomView>::Owned {
        self.into_owned()
    }
}

impl Mountable for CowStrState<'_> {
    fn unmount(&mut self) {
        self.node.unmount()
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        Rndr::insert_node(parent, self.node.as_ref(), marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        self.node.insert_before_this(child)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}
