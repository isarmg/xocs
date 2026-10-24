//! Dialog media and its page-owned teardown.
use super::t;
use blog_interface::component::component_props_builder;
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use wasm_bindgen::JsCast;

#[component]
pub(crate) fn Lightbox(src: RwSignal<Option<String>>) -> impl IntoView {
    View::new(For(component_props_builder(&For)
        .each(move || src.get().into_iter().collect::<Vec<_>>())
        .key(|s| s.clone())
        .children(move |url| {
            View::new(LightboxDialog(
                component_props_builder(&LightboxDialog)
                    .url(url)
                    .close(Callback::new(move |()| src.set(None)))
                    .build(),
            ))
        })
        .build()))
}

#[component]
fn LightboxDialog(url: String, close: Callback<()>) -> impl IntoView {
    let node = NodeRef::<blog_interface::html::Dialog>::new();
    Effect::new(move |_| {
        if let Some(node) = node.get() {
            let opener = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .active_element()
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
            let _ = node.show_modal();
            let node = SendWrapper::new(node);
            let opener = SendWrapper::new(opener);
            on_cleanup(move || {
                node.close();
                if let Some(e) = opener.as_ref() {
                    let _ = e.focus();
                }
            });
        }
    });
    View::new(
        blog_interface::html::dialog()
            .child((
                blog_interface::html::button()
                    .child("×")
                    .class("image-lightbox-close")
                    .r#type("button")
                    .aria_label(t("关闭图片", "Close image"))
                    .on(blog_interface::ev::click, move |_| close.run(())),
                blog_interface::html::img()
                    .src(url)
                    .alt(t("放大的图片", "Enlarged image")),
            ))
            .class("image-lightbox")
            .node_ref(node)
            .aria_label(t("查看图片", "View image"))
            .on(
                blog_interface::ev::Custom::new("cancel"),
                move |ev: web_sys::Event| {
                    ev.prevent_default();
                    close.run(())
                },
            )
            .on(blog_interface::ev::mousedown, move |ev| {
                if ev.target() == ev.current_target() {
                    close.run(())
                }
            }),
    )
}
