use crate::browser_helpers::helpers::TimeoutHandle;
use crate::reactivity::{
    effect::RenderEffect,
    owner::{on_cleanup, StoredValue},
    signal::RwSignal,
    traits::{Get, GetUntracked, GetValue, Set, SetValue},
    wrappers::read::Signal,
};
use crate::view_tree::prelude::*;
use crate::{
    children::{ChildrenFn, ToChildren},
    component,
    control_flow::{Show, ShowProps},
    IntoView,
};
use core::time::Duration;

/// Shows its children while `when` is true and delays unmounting by `hide_delay`.
/// The optional classes are updated immediately so a hide animation can finish
/// before the children are unmounted. Pending timers are cleared on cleanup.
#[component]
pub fn AnimatedShow(
    /// The components Show wraps
    children: ChildrenFn,
    /// If the component should show or not
    #[prop(into)]
    when: Signal<bool>,
    /// Optional CSS class to apply if `when == true`
    #[prop(optional)]
    show_class: &'static str,
    /// Optional CSS class to apply if `when == false`
    #[prop(optional)]
    hide_class: &'static str,
    /// The timeout after which the component will be unmounted if `when == false`
    hide_delay: Duration,
) -> impl IntoView {
    let handle: StoredValue<Option<TimeoutHandle>> = StoredValue::new(None);
    let cls = RwSignal::new(if when.get_untracked() {
        show_class
    } else {
        hide_class
    });
    let show = RwSignal::new(when.get_untracked());

    let eff = RenderEffect::new(move |_| {
        if when.get() {
            // clear any possibly active timer
            if let Some(h) = handle.get_value() {
                h.clear();
            }

            cls.set(show_class);
            show.set(true);
        } else {
            cls.set(hide_class);

            let h = crate::browser_helpers::helpers::set_timeout_with_handle(
                move || show.set(false),
                hide_delay,
            )
            .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
            handle.set_value(Some(h));
        }
    });

    on_cleanup(move || {
        if let Some(Some(h)) = handle.try_get_value() {
            h.clear();
        }
        drop(eff);
    });

    Show(
        ShowProps::builder()
            .when(move || show.get())
            .fallback(|| ())
            .children(ToChildren::to_children(move || {
                crate::html::div()
                    .class(move || cls.get())
                    .child(children())
            }))
            .build(),
    )
}
