//! Decorative views with no request or form state.
use blog_interface::html;
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use wasm_bindgen::JsCast;

#[component]
pub(crate) fn Petals() -> impl IntoView {
    View::new(
        blog_interface::html::div()
            .child(
                (0..18)
                    .map(|i| {
                        View::new(
    blog_interface::html::span()
        .style(
            format!(
                "--petal-left:{}%;--petal-delay:{}s;--petal-duration:{}s;--petal-drift:{}px",
                (i * 37 + 9) % 100,- (i as f64 * 2.7), 16 + i % 7, if i % 2 == 1 { -
                70 } else { 90 }
            ),
        ),
)
                    })
                    .collect_view(),
            )
            .class("reference-petals")
            .aria_hidden("true"),
    )
}

#[component]
pub(crate) fn Sliding(title: String) -> impl IntoView {
    let node = NodeRef::<html::Span>::new();
    Effect::new(move |_| {
        if let Some(node) = node.get() {
            let cleanup = SendWrapper::new(crate::browser::observe_sliding(
                &node.unchecked_into::<web_sys::Element>(),
            ));
            on_cleanup(move || {
                drop(cleanup);
            });
        }
    });
    View::new(
        html::span()
            .child(
                html::span()
                    .child(title.clone())
                    .class("sliding-title-text"),
            )
            .class("sliding-title")
            .node_ref(node)
            .title(title),
    )
}
