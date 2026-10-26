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

#[doc(hidden)]
pub use crate::no_attrs;
