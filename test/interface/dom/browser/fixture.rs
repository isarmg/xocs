//! A browser consumer of the public DOM helpers, kept outside product sources.
#![forbid(unsafe_code)]

use blog_dom::browser_helpers::helpers::{self, WindowListenerHandle};
use blog_dom::timers::callback::{Interval, Timeout};
use std::{cell::RefCell, time::Duration};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

thread_local! {
    static LISTENER: RefCell<Option<WindowListenerHandle>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn call_helper(operation: &str) -> Result<(), JsValue> {
    match operation {
        "raf" => helpers::request_animation_frame(|| {}),
        "cancel-raf" => helpers::request_animation_frame_with_handle(|| {})?.cancel(),
        "idle" => helpers::request_idle_callback(|| {}),
        "microtask" => helpers::queue_microtask(|| {}),
        "timeout" => helpers::set_timeout(|| {}, Duration::ZERO),
        "interval" => helpers::set_interval(|| {}, Duration::ZERO),
        "callback-timeout" => drop(Timeout::new(0, || {})),
        "callback-interval" => drop(Interval::new(0, || {})),
        "timeout-range" => {
            helpers::set_timeout_with_handle(|| {}, Duration::from_millis(i32::MAX as u64 + 1))?
                .clear();
        }
        "interval-range" => {
            helpers::set_interval_with_handle(|| {}, Duration::from_millis(i32::MAX as u64 + 1))?
                .clear();
        }
        "callback-timeout-range" => drop(Timeout::new(u32::MAX, || {})),
        "callback-interval-range" => drop(Interval::new(u32::MAX, || {})),
        _ => return Err(js_sys::TypeError::new("unknown fixture operation").into()),
    }
    Ok(())
}

#[wasm_bindgen]
pub fn queue_value(element: web_sys::Element, value: &str) {
    blog_dom::view_tree::renderer::dom::Dom::set_property_or_value(
        &element,
        "value",
        &value.into(),
    );
}

#[wasm_bindgen]
pub fn listen(callback: js_sys::Function) {
    let listener = helpers::window_event_listener_untyped("dom-contract", move |_| {
        callback
            .call0(&JsValue::UNDEFINED)
            .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
    });
    let previous = LISTENER.replace(Some(listener));
    drop(previous);
}

#[wasm_bindgen]
pub fn drop_listener() {
    drop(LISTENER.take());
}

#[wasm_bindgen]
pub fn remove_listener() {
    LISTENER
        .take()
        .expect("invariant: the fixture registered a listener before removing it")
        .remove();
}
