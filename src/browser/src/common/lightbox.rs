//! Dialog media and its page-owned teardown.
use super::t;
use blog_interface::component::component_props_builder;
use blog_interface::prelude::*;

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
    blog_interface::modal::ModalSurface(
        component_props_builder(&blog_interface::modal::ModalSurface)
            .class("image-lightbox")
            .label(t("查看图片", "View image"))
            .close(close)
            .dismiss_on_backdrop(true)
            .children(blog_interface::children::ToChildren::to_children(
                move || {
                    (
                        blog_interface::html::button()
                            .child("×")
                            .class("image-lightbox-close")
                            .r#type("button")
                            .aria_label(t("关闭图片", "Close image"))
                            .on(blog_interface::ev::click, move |_| close.run(())),
                        blog_interface::html::img()
                            .src(url.clone())
                            .alt(t("放大的图片", "Enlarged image")),
                    )
                },
            ))
            .build(),
    )
}
