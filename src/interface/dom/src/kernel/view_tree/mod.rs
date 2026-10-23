//! A statically typed CSR view tree backed by browser DOM operations.
//!
//! Rendering is independent of reactivity; reactive adapters retain state between updates.

#![deny(missing_docs)]

/// Commonly-used traits.
pub mod prelude {
    pub use crate::view_tree::{
        html::{
            attribute::{
                any_attribute::IntoAnyAttribute,
                aria::AriaAttributes,
                custom::CustomAttribute,
                global::{
                    ClassAttribute, GlobalAttributes, GlobalOnAttributes, OnAttribute,
                    OnTargetAttribute, PropAttribute, StyleAttribute,
                },
                IntoAttributeValue,
            },
            directive::DirectiveAttribute,
            element::{ElementChild, ElementExt, InnerHtmlAttribute},
            node_ref::NodeRefAttribute,
        },
        renderer::dom::Dom,
        view::{
            add_attr::AddAnyAttr,
            any_view::{AnyView, IntoAny, IntoMaybeErased},
            DomView, IntoRender, Mountable, Render,
        },
    };
}

use wasm_bindgen::JsValue;
use web_sys::Node;

/// Helpers for interacting with the DOM.
pub mod dom;
/// Types for building a statically-typed HTML view tree.
pub mod html;
/// Types for MathML.
pub mod mathml;
/// Browser DOM operations and event listener lifetimes.
pub mod renderer;
/// Types for SVG.
pub mod svg;
/// Core logic for manipulating views.
pub mod view;

pub use crate::alternatives as either;

/// View implementations for the `oco_ref` crate (cheaply-cloned string types).
pub mod oco;
/// View implementations for the `reactive_graph` crate.
pub mod reactive_graph;

/// A type-erased container.
pub mod erased;

pub(crate) trait UnwrapOrDebug {
    type Output;

    fn or_debug(self, el: &Node, label: &'static str);

    fn ok_or_debug(self, el: &Node, label: &'static str) -> Option<Self::Output>;
}

impl<T> UnwrapOrDebug for Result<T, JsValue> {
    type Output = T;

    #[track_caller]
    fn or_debug(self, el: &Node, name: &'static str) {
        #[cfg(any(debug_assertions, leptos_debuginfo))]
        {
            if let Err(err) = self {
                let location = std::panic::Location::caller();
                web_sys::console::warn_3(
                    &JsValue::from_str(&format!(
                        "[WARNING] Non-fatal error at {location}, while \
                         calling {name} on "
                    )),
                    el,
                    &err,
                );
            }
        }
        #[cfg(not(any(debug_assertions, leptos_debuginfo)))]
        {
            let _ = (self, el, name);
        }
    }

    #[track_caller]
    fn ok_or_debug(self, el: &Node, name: &'static str) -> Option<Self::Output> {
        #[cfg(any(debug_assertions, leptos_debuginfo))]
        {
            if let Err(err) = &self {
                let location = std::panic::Location::caller();
                web_sys::console::warn_3(
                    &JsValue::from_str(&format!(
                        "[WARNING] Non-fatal error at {location}, while \
                         calling {name} on "
                    )),
                    el,
                    err,
                );
            }
            self.ok()
        }
        #[cfg(not(any(debug_assertions, leptos_debuginfo)))]
        {
            let _ = (el, name);
            self.ok()
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! or_debug {
    ($action:expr, $el:expr, $label:literal) => {
        if cfg!(any(debug_assertions, leptos_debuginfo)) {
            $crate::view_tree::UnwrapOrDebug::or_debug($action, $el, $label);
        } else {
            _ = $action;
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! ok_or_debug {
    ($action:expr, $el:expr, $label:literal) => {
        if cfg!(any(debug_assertions, leptos_debuginfo)) {
            $crate::view_tree::UnwrapOrDebug::ok_or_debug($action, $el, $label)
        } else {
            $action.ok()
        }
    };
}

#[doc(hidden)]
pub use crate::{no_attrs, ok_or_debug, or_debug};
