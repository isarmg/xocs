//! A public corner-controller consumer for real browser lifetime contracts.
#![forbid(unsafe_code)]

#[cfg(target_arch = "wasm32")]
mod browser {
    use blog_interface::{
        server_corners::{ServerCornersController, start},
        web_sys::Document,
    };
    use std::cell::RefCell;
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

    thread_local! {
        static CONTROLLER: RefCell<Option<ServerCornersController>> = const { RefCell::new(None) };
    }

    #[wasm_bindgen]
    pub fn start_corner_controller(document: Document) -> Result<(), JsValue> {
        let controller = start(&document, "[data-corner-listener-surface]")?;
        drop(CONTROLLER.replace(Some(controller)));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn stop_corner_controller() {
        let controller = CONTROLLER
            .with(|value| value.borrow().clone())
            .expect("invariant: fixture starts the controller before stopping it");
        controller.stop();
    }

    #[wasm_bindgen]
    pub fn drop_corner_controller() {
        drop(CONTROLLER.take());
    }
}
