use crate::shared_value::Oco;
use crate::view_tree::{
    html::{
        attribute::AttributeValue, class::IntoClass, element::InnerHtmlValue,
        property::IntoProperty, style::IntoStyle,
    },
    no_attrs,
    prelude::{DomView, Mountable, Render},
    renderer::Rndr,
};
use wasm_bindgen::JsValue;

/// Retained view state for [`Oco`].
pub struct OcoStrState {
    node: crate::view_tree::renderer::types::Text,
    str: Oco<'static, str>,
}

impl Render for Oco<'static, str> {
    type State = OcoStrState;

    fn build(self) -> Self::State {
        let node = Rndr::create_text_node(&self);
        OcoStrState { node, str: self }
    }

    fn rebuild(self, state: &mut Self::State) {
        let OcoStrState { node, str } = state;
        if &self != str {
            Rndr::set_text(node, &self);
            *str = self;
        }
    }
}

no_attrs!(Oco<'static, str>);

impl DomView for Oco<'static, str> {
    type Owned = Self;

    fn into_owned(self) -> <Self as DomView>::Owned {
        self
    }
}

impl Mountable for OcoStrState {
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

impl AttributeValue for Oco<'static, str> {
    type State = (
        crate::view_tree::renderer::types::Element,
        Oco<'static, str>,
    );
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn build(self, el: &crate::view_tree::renderer::types::Element, key: &str) -> Self::State {
        Rndr::set_attribute(el, key, &self);
        (el.clone(), self)
    }

    fn rebuild(self, key: &str, state: &mut Self::State) {
        let (el, prev_value) = state;
        if self != *prev_value {
            Rndr::set_attribute(el, key, &self);
        }
        *prev_value = self;
    }

    fn into_cloneable(mut self) -> Self::Cloneable {
        // ensure it's reference-counted
        self.upgrade_inplace();
        self
    }

    fn into_cloneable_owned(mut self) -> Self::CloneableOwned {
        // ensure it's reference-counted
        self.upgrade_inplace();
        self
    }
}

impl IntoClass for Oco<'static, str> {
    type State = (crate::view_tree::renderer::types::Element, Self);
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn build(self, el: &crate::view_tree::renderer::types::Element) -> Self::State {
        Rndr::set_attribute(el, "class", &self);
        (el.clone(), self)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (el, prev) = state;
        if self != *prev {
            Rndr::set_attribute(el, "class", &self);
        }
        *prev = self;
    }

    fn into_cloneable(mut self) -> Self::Cloneable {
        // ensure it's reference-counted
        self.upgrade_inplace();
        self
    }

    fn into_cloneable_owned(mut self) -> Self::CloneableOwned {
        // ensure it's reference-counted
        self.upgrade_inplace();
        self
    }

    fn reset(state: &mut Self::State) {
        let (el, _prev) = state;
        Rndr::remove_attribute(el, "class");
    }
}

impl IntoProperty for Oco<'static, str> {
    type State = (crate::view_tree::renderer::types::Element, JsValue);
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn build(self, el: &crate::view_tree::renderer::types::Element, key: &str) -> Self::State {
        let value = JsValue::from_str(self.as_ref());
        Rndr::set_property_or_value(el, key, &value);
        (el.clone(), value)
    }

    fn rebuild(self, state: &mut Self::State, key: &str) {
        let (el, prev) = state;
        let value = JsValue::from_str(self.as_ref());
        Rndr::set_property_or_value(el, key, &value);
        *prev = value;
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self
    }
}

impl IntoStyle for Oco<'static, str> {
    type State = (crate::view_tree::renderer::types::Element, Self);
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn build(self, el: &crate::view_tree::renderer::types::Element) -> Self::State {
        Rndr::set_attribute(el, "style", &self);
        (el.clone(), self)
    }

    fn rebuild(self, state: &mut Self::State) {
        let (el, prev) = state;
        if self != *prev {
            Rndr::set_attribute(el, "style", &self);
        }
        *prev = self;
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self
    }

    fn reset(state: &mut Self::State) {
        let (el, _prev) = state;
        Rndr::remove_attribute(el, "style");
    }
}

impl InnerHtmlValue for Oco<'static, str> {
    type State = (crate::view_tree::renderer::types::Element, Self);
    type Cloneable = Self;
    type CloneableOwned = Self;

    fn build(self, el: &crate::view_tree::renderer::types::Element) -> Self::State {
        Rndr::set_inner_html(el, &self);
        (el.clone(), self)
    }

    fn rebuild(self, state: &mut Self::State) {
        if self != state.1 {
            Rndr::set_inner_html(&state.0, &self);
            state.1 = self;
        }
    }

    fn into_cloneable(mut self) -> Self::Cloneable {
        // ensure it's reference-counted
        self.upgrade_inplace();
        self
    }

    fn into_cloneable_owned(mut self) -> Self::CloneableOwned {
        // ensure it's reference-counted
        self.upgrade_inplace();
        self
    }
}
