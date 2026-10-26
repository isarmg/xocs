use crate::browser_helpers::helpers::document;
use crate::reactivity::{effect::Effect, graph::untrack, owner::Owner};
use crate::{children::TypedChildrenFn, mount, IntoView};
use blog_component_macros::component;
use std::sync::Arc;

/// Renders components somewhere else in the DOM.
///
/// Useful for inserting modals and tooltips outside of a cropping layout.
/// If no mount point is given, the portal is inserted in `document.body`;
/// it is wrapped in a `<div>` unless  `is_svg` is `true` in which case it's wrapped in a `<g>`.
/// Setting `use_shadow` to `true` places the element in a shadow root to isolate styles.
/// This requires the `shadow-dom` feature, when explicitly enabled.
/// Without it, setting `use_shadow` to `true` panics before mounting.
#[component]
pub fn Portal<V>(
    /// Target element where the children will be appended
    #[prop(into, optional)]
    mount: Option<web_sys::Element>,
    /// Whether to use a shadow DOM inside `mount`. Defaults to `false`.
    /// Requires the `shadow-dom` feature when set to `true`.
    #[prop(optional)]
    use_shadow: bool,
    /// When using SVG this has to be set to `true`. Defaults to `false`.
    #[prop(optional)]
    is_svg: bool,
    /// The children to teleport into the `mount` element
    children: TypedChildrenFn<V>,
) -> impl IntoView
where
    V: IntoView + 'static,
{
    #[cfg(not(feature = "shadow-dom"))]
    assert!(
        !use_shadow,
        "Portal use_shadow=true requires the `shadow-dom` feature on blog_interface"
    );

    if cfg!(target_arch = "wasm32") {
        use crate::thread_bound::SendWrapper;
        use wasm_bindgen::JsCast;

        let mount =
            mount.unwrap_or_else(|| document().body().expect("body to exist").unchecked_into());
        let children = children.into_inner();

        Effect::new(move |_| {
            let container = if is_svg {
                document()
                    .create_element_ns(Some("http://www.w3.org/2000/svg"), "g")
                    .expect("SVG element creation to work")
            } else {
                document()
                    .create_element("div")
                    .expect("HTML element creation to work")
            };

            #[cfg(feature = "shadow-dom")]
            let render_root = if use_shadow {
                container
                    .attach_shadow(&web_sys::ShadowRootInit::new(web_sys::ShadowRootMode::Open))
                    .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
                    .unchecked_into()
            } else {
                container.clone()
            };
            #[cfg(not(feature = "shadow-dom"))]
            let render_root = container.clone();

            mount
                .append_child(&container)
                .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
            let handle = SendWrapper::new((
                mount::mount_to(render_root.unchecked_into(), {
                    let children = Arc::clone(&children);
                    move || untrack(|| children())
                }),
                container,
            ));

            Owner::on_cleanup({
                move || {
                    let (handle, container) = handle.take();
                    drop(handle);
                    container.remove();
                }
            })
        });
    }
}
