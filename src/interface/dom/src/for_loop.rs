use crate::into_view::IntoView;
use crate::reactivity::{
    owner::Owner,
    signal::{ArcRwSignal, ReadSignal},
    traits::Set,
};
use crate::view_tree::{
    reactive_graph::OwnedView,
    view::keyed::{keyed, SerializableKey},
};
use blog_component_macros::component;
use std::hash::Hash;

/// Renders keyed children while preserving each row's DOM nodes and reactive owner.
///
/// ```
/// # extern crate blog_dom as blog_interface;
/// # use blog_dom::{control_flow::{For, ForProps}, html::li, prelude::*};
/// # fn example() -> impl IntoView {
/// For(ForProps::builder()
///     .each(|| vec![1, 2, 3])
///     .key(|value| *value)
///     .children(|value| li().child(value))
///     .build())
/// # }
/// ```
/// # extern crate blog_dom as blog_interface;
#[component]
pub fn For<IF, I, T, EF, N, KF, K>(
    /// Items over which the component should iterate.
    each: IF,
    /// A key function that will be applied to each item.
    key: KF,
    /// A function that takes the item, and returns the view that will be displayed for each item.
    children: EF,
) -> impl IntoView
where
    IF: Fn() -> I + Send + 'static,
    I: IntoIterator<Item = T> + Send + 'static,
    EF: Fn(T) -> N + Send + Clone + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + Send + Clone + 'static,
    K: Eq + Hash + SerializableKey + 'static,
    T: Send + 'static,
{
    // this takes the owner of the For itself
    // this will end up with N + 1 children
    // 1) the effect for the `move || keyed(...)` updates
    // 2) an owner for each child
    //
    // this means
    // a) the reactive owner for each row will not be cleared when the whole list updates
    // b) context provided in each row will not wipe out the others
    let parent = Owner::current().expect("no reactive owner");
    let children = move |_, child| {
        let owner = parent.with(Owner::new);
        let view = owner.with(|| children(child));
        (drop, OwnedView::new_with_owner(view, owner))
    };
    move || keyed(each(), key.clone(), children.clone())
}

/// Renders keyed children and supplies each row's current reactive index.
/// Moving a row updates its index without recreating its view or owner.
#[component]
pub fn ForEnumerate<IF, I, T, EF, N, KF, K>(
    /// Items over which the component should iterate.
    each: IF,
    /// A key function that will be applied to each item.
    key: KF,
    /// A function that takes the index and the item, and returns the view that will be displayed for each item.
    children: EF,
) -> impl IntoView
where
    IF: Fn() -> I + Send + 'static,
    I: IntoIterator<Item = T> + Send + 'static,
    EF: Fn(ReadSignal<usize>, T) -> N + Send + Clone + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + Send + Clone + 'static,
    K: Eq + Hash + SerializableKey + 'static,
    T: Send + 'static,
{
    // this takes the owner of the For itself
    // this will end up with N + 1 children
    // 1) the effect for the `move || keyed(...)` updates
    // 2) an owner for each child
    //
    // this means
    // a) the reactive owner for each row will not be cleared when the whole list updates
    // b) context provided in each row will not wipe out the others
    let parent = Owner::current().expect("no reactive owner");
    let children = move |index, child| {
        let owner = parent.with(Owner::new);
        let (index, set_index) = ArcRwSignal::new(index).split();
        let view = owner.with(|| children(index.into(), child));
        (
            move |index| set_index.set(index),
            OwnedView::new_with_owner(view, owner),
        )
    };
    move || keyed(each(), key.clone(), children.clone())
}
