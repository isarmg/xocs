#![forbid(unsafe_code)]
//! Shared browser-only support. The Linux server crate remains independent.
pub mod api;
pub mod components;
pub mod contracts;
pub mod i18n;
pub mod modal;
pub mod server_corners;
pub mod theme;

extern crate self as blog_interface;

pub use blog_dom::*;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;
pub use web_sys;

use wasm_bindgen::{JsCast, JsValue};

/// Mount the application while decorative fonts load independently.
pub fn mount<F, V>(root_id: &str, app: F)
where
    F: FnOnce() -> V + 'static,
    V: IntoView + 'static,
{
    mount_application(root_id, app, None);
}

/// Mount a Server view, decorating the product's explicit surface roles.
/// The selector includes all candidate roles, including currently hidden surfaces.
pub fn mount_server<F, V>(root_id: &str, app: F, surfaces: &'static str)
where
    F: FnOnce() -> V + 'static,
    V: IntoView + 'static,
{
    mount_application(root_id, app, Some(surfaces));
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    // Keep the controller alive for this document; its pagehide listener releases
    // observers, styles and URLs before an uncached page leaves.
    static SERVER_CORNERS: std::cell::RefCell<Option<server_corners::ServerCornersController>> = const { std::cell::RefCell::new(None) };
}

fn mount_application<F, V>(root_id: &str, app: F, corner_surfaces: Option<&'static str>)
where
    F: FnOnce() -> V + 'static,
    V: IntoView + 'static,
{
    blog_dom::browser_panic::set_once();
    i18n::initialize();
    theme::initialize();
    let result: Result<(), JsValue> = (|| {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("Browser document is unavailable"))?;
        let root = document
            .get_element_by_id(root_id)
            .ok_or_else(|| JsValue::from_str("Application root is missing"))?
            .dyn_into::<web_sys::HtmlElement>()?;
        let html = document
            .document_element()
            .ok_or_else(|| JsValue::from_str("Browser document root is missing"))?;
        // A timed-out startup cannot later replace its visible failure page.
        if html.get_attribute("data-blog-startup").as_deref() == Some("failed") {
            return Err(JsValue::from_str("Application startup has already failed"));
        }
        blog_interface::mount::mount_to(root, app).forget();
        html.set_attribute("data-blog-appearance", "content-blocks")?;
        html.set_attribute("data-blog-selection", "underline")?;
        #[cfg(target_arch = "wasm32")]
        if let Some(surfaces) = corner_surfaces {
            let controller = crate::server_corners::start(&document, surfaces)?;
            SERVER_CORNERS.with(|value| *value.borrow_mut() = Some(controller));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = corner_surfaces;
        html.set_attribute("data-blog-runtime", "blog-interface")?;
        document.dispatch_event(&web_sys::Event::new("blog-startup-ready")?)?;
        Ok(())
    })();
    if let Err(error) = result {
        web_sys::console::error_2(&JsValue::from_str("Application startup failed"), &error);
        if let Some(document) = web_sys::window().and_then(|window| window.document())
            && let Ok(event) = web_sys::Event::new("blog-startup-failed")
        {
            document
                .dispatch_event(&event)
                .expect("invariant: an initialized startup failure event can be dispatched");
        }
    }
}
