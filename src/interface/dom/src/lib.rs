#![deny(missing_docs)]

//! Browser CSR runtime used by the public `blog-interface` crate.
//! Use `blog_dom::prelude::*` and the DOM builders from that facade.
//! See the package README for source and license history.

// This maintained CSR runtime uses the pinned stable Rust toolchain.
// Stable closure implementations are kept below; nightly APIs are not enabled.

extern crate alloc;
extern crate self as blog_dom;
extern crate self as blog_interface;

/// Browser kernel implementation: alternatives.
#[path = "kernel/alternatives/mod.rs"]
pub mod alternatives;
/// Browser kernel implementation: browser helpers.
#[path = "kernel/browser_helpers/mod.rs"]
pub mod browser_helpers;
/// Browser kernel implementation: browser panic.
#[path = "kernel/browser_panic/mod.rs"]
pub mod browser_panic;
/// Browser kernel implementation: erased value.
#[path = "kernel/erased_value/mod.rs"]
pub mod erased_value;
/// Browser kernel implementation: events.
#[path = "kernel/events/mod.rs"]
pub mod events;
/// Browser kernel implementation: iterators.
#[path = "kernel/iterators/mod.rs"]
pub mod iterators;
/// Browser kernel implementation: owned guard.
#[path = "kernel/owned_guard/mod.rs"]
pub mod owned_guard;
/// Browser kernel implementation: poison.
#[path = "kernel/poison/mod.rs"]
pub mod poison;
/// Browser kernel implementation: reactivity.
#[path = "kernel/reactivity/mod.rs"]
pub mod reactivity;
/// Browser kernel implementation: shared value.
#[path = "kernel/shared_value/mod.rs"]
pub mod shared_value;
/// Browser kernel implementation: slots.
#[path = "kernel/slots/mod.rs"]
pub mod slots;
/// Browser kernel implementation: tasks.
#[path = "kernel/tasks/mod.rs"]
pub mod tasks;
/// Browser kernel implementation: thread bound.
#[path = "kernel/thread_bound/mod.rs"]
pub mod thread_bound;
/// Browser kernel implementation: timers.
#[path = "kernel/timers/mod.rs"]
pub mod timers;
/// Browser kernel implementation: tuples.
#[path = "kernel/tuples/mod.rs"]
pub mod tuples;
/// Browser kernel implementation: vector drain.
#[path = "kernel/vector_drain/mod.rs"]
pub mod vector_drain;
/// Browser kernel implementation: view errors.
#[path = "kernel/view_errors/mod.rs"]
pub mod view_errors;
/// Browser kernel implementation: view tree.
#[path = "kernel/view_tree/mod.rs"]
pub mod view_tree;

/// Exports all the core types of the library.
pub mod prelude {
    // Traits
    // These should always be exported from the prelude
    pub use crate::reactivity::prelude::*;
    pub use crate::view_tree::prelude::*;

    // Structs
    // In the future, maybe we should remove this blanket export
    // However, it is definitely useful relative to looking up every struct etc.
    mod export_types {
        pub use crate::browser_helpers::helpers::*;
        pub use crate::reactivity::{
            computed::*,
            effect::*,
            graph::untrack,
            owner::*,
            signal::*,
            wrappers::{read::*, write::*},
        };
        pub use crate::shared_value::*;
        pub use crate::view_tree::{
            reactive_graph::{bind::BindAttribute, node_ref::*},
            view::fragment::Fragment,
        };
        pub use crate::{
            callback::*, children::*, component::*, control_flow::*, error::*, into_view::*,
            mount::*, text_prop::*,
        };
        pub use blog_component_macros::*;
    }
    pub use export_types::*;
}

/// A standard way to wrap functions and closures to pass them to components.
pub use crate::reactivity::callback;

/// Types that can be passed as the `children` prop of a component.
pub mod children;

/// Wrapper for intercepting component attributes.
pub mod attribute_interceptor;

#[doc(hidden)]
/// Traits used to implement component constructors.
pub mod component;

/// Tools for handling errors.
pub mod error {
    pub use crate::view_errors::*;
}

/// Control-flow components like `Show`, `For`, and `AnimatedShow`.
pub mod control_flow {
    pub use crate::{animated_show::*, for_loop::*, show::*, show_let::*};
}
mod animated_show;
mod for_loop;
mod show;
mod show_let;

/// A component that allows rendering a component somewhere else.
pub mod portal;

/// Types for reactive string properties for components.
pub mod text_prop;
pub use blog_component_macros::*;
#[doc(hidden)]
pub mod typed_builder {
    pub use blog_component_macros::TypedBuilder;
    /// Supplies a default field value while preserving the builder's type state.
    pub trait NextFieldDefault<Fields> {
        /// The resulting field value.
        type Output;
        /// Resolve the field default from the existing fields.
        fn resolve(input: Fields) -> Self::Output;
    }
}
#[doc(hidden)]
pub use blog_component_macros as typed_builder_macro;
mod into_view;
#[doc(inline)]
pub use crate::browser_helpers as leptos_dom;
pub use into_view::IntoView;
mod provider;
#[doc(inline)]
pub use crate::view_tree as tachys;
/// Tools to mount an application to the browser DOM.
pub mod mount;
#[doc(inline)]
pub use crate::alternatives as either;
#[doc(inline)]
pub use crate::reactivity as reactive;
#[doc(inline)]
pub use crate::shared_value as oco;

/// Provide and access data along the reactive graph, sharing data without directly passing arguments.
pub mod context {
    pub use crate::provider::*;
    pub use crate::reactivity::owner::{provide_context, use_context};
}

/// HTML attribute types.
#[doc(inline)]
pub use crate::view_tree::html::attribute as attr;
/// HTML element types.
#[doc(inline)]
pub use crate::view_tree::html::element as html;
/// HTML event types.
#[doc(no_inline)]
pub use crate::view_tree::html::event as ev;
/// MathML element types.
#[doc(inline)]
pub use crate::view_tree::mathml as math;
/// SVG element types.
#[doc(inline)]
pub use crate::view_tree::svg;

/// Utilities for simple isomorphic logging to the console or terminal.
pub mod logging {
    pub use crate::browser_helpers::{debug_error, debug_log, debug_warn, error, log, warn};
}

/// Utilities for working with asynchronous tasks.
pub mod task {
    use crate::reactivity::computed::ScopedFuture;
    use crate::tasks::Executor;
    use std::future::Future;

    /// Spawns a thread-safe [`Future`].
    ///
    /// This will be run with the current reactive owner and observer using a [`ScopedFuture`].
    #[track_caller]
    #[inline(always)]
    pub fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
        let fut = ScopedFuture::new(fut);

        #[cfg(not(target_family = "wasm"))]
        Executor::spawn(fut);

        #[cfg(target_family = "wasm")]
        Executor::spawn_local(fut);
    }

    /// Spawns a [`Future`] that cannot be sent across threads.
    #[track_caller]
    #[inline(always)]
    pub fn spawn_local(fut: impl Future<Output = ()> + 'static) {
        Executor::spawn_local(fut)
    }

    /// Waits until the next "tick" of the current async executor.
    pub async fn tick() {
        Executor::tick().await
    }

    pub use crate::reactivity::{spawn_local_scoped, spawn_local_scoped_with_cancellation};
}
