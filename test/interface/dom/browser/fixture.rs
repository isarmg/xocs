//! A browser consumer of the public DOM helpers, kept outside product sources.
#![forbid(unsafe_code)]

use blog_dom::browser_helpers::helpers::{self, WindowListenerHandle};
use blog_dom::timers::callback::{Interval, Timeout};
use std::{cell::RefCell, time::Duration};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

thread_local! {
    static LISTENER: RefCell<Option<WindowListenerHandle>> = const { RefCell::new(None) };
}

struct DropProbe(js_sys::Function);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0
            .call0(&JsValue::UNDEFINED)
            .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
    }
}

#[wasm_bindgen]
pub fn timer_failure_cleanup(operation: &str, dropped: js_sys::Function) {
    let probe = DropProbe(dropped);
    match operation {
        "timeout" => drop(Timeout::new(0, move || drop(probe))),
        "interval" => drop(Interval::new(0, move || {
            let _ = &probe;
        })),
        "timeout-range" => drop(Timeout::new(u32::MAX, move || drop(probe))),
        "interval-range" => drop(Interval::new(u32::MAX, move || {
            let _ = &probe;
        })),
        _ => wasm_bindgen::throw_val(js_sys::TypeError::new("unknown timer operation").into()),
    }
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

#[wasm_bindgen]
pub fn attribute_transition(
    operation: &str,
    element: web_sys::Element,
    callback: js_sys::Function,
) {
    use blog_dom::either::Either;
    use blog_dom::view_tree::html::{
        attribute::{
            any_attribute::IntoAnyAttribute,
            custom::{custom_attribute, CustomAttr},
            Attribute,
        },
        element::inner_html,
        event::{click, on},
        property::prop,
    };

    fn replace_attribute<A: Attribute, B: Attribute>(old: A, new: B, element: &web_sys::Element) {
        let mut state = Either::<A, B>::Left(old).build(element);
        Either::<A, B>::Right(new).rebuild(&mut state);
    }

    match operation {
        "either-right" => {
            type Branch = Either<CustomAttr<&'static str, &'static str>, ()>;
            let first: Branch = Either::Left(custom_attribute("data-first", "left"));
            let mut state = first.build(&element);
            let next: Branch = Either::Right(());
            next.rebuild(&mut state);
        }
        "either-left" => {
            type Branch = Either<(), CustomAttr<&'static str, &'static str>>;
            let first: Branch = Either::Right(custom_attribute("data-first", "right"));
            let mut state = first.build(&element);
            let next: Branch = Either::Left(());
            next.rebuild(&mut state);
        }
        "either-dynamic-key" => {
            type Branch = Either<CustomAttr<&'static str, &'static str>, ()>;
            let first: Branch = Either::Left(custom_attribute("data-first", "same"));
            let mut state = first.build(&element);
            let next: Branch = Either::Left(custom_attribute("data-second", "same"));
            next.rebuild(&mut state);
        }
        "erased-type" => {
            let mut state = custom_attribute("data-first", "old")
                .into_any_attr()
                .build(&element);
            custom_attribute("data-second", true)
                .into_any_attr()
                .rebuild(&mut state);
        }
        "erased-same-key" => {
            let mut state = custom_attribute("data-first", true)
                .into_any_attr()
                .build(&element);
            custom_attribute("data-first", "new")
                .into_any_attr()
                .rebuild(&mut state);
        }
        "erased-dynamic-key" => {
            let mut state = custom_attribute("data-first", "same")
                .into_any_attr()
                .build(&element);
            custom_attribute("data-second", "same")
                .into_any_attr()
                .rebuild(&mut state);
        }
        "erased-retained-keys" => {
            let mut state = custom_attribute("data-first", Some("same"))
                .into_any_attr()
                .build(&element);
            custom_attribute("data-second", Some("same"))
                .into_any_attr()
                .rebuild(&mut state);
            ().into_any_attr().rebuild(&mut state);
        }
        "erased-property" => {
            let mut state = prop("contractProperty", "old")
                .into_any_attr()
                .build(&element);
            ().into_any_attr().rebuild(&mut state);
        }
        "erased-inner-html" => {
            let mut state = inner_html("<b>old</b>").into_any_attr().build(&element);
            ().into_any_attr().rebuild(&mut state);
        }
        "erased-listener" => {
            let mut state = on(click, move |_| {
                callback
                    .call0(&JsValue::UNDEFINED)
                    .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
            })
            .into_any_attr()
            .build(&element);
            custom_attribute("data-second", "new")
                .into_any_attr()
                .rebuild(&mut state);
        }
        "either-property" => replace_attribute(prop("contractProperty", "old"), (), &element),
        "either-inner-html" => replace_attribute(inner_html("<b>old</b>"), (), &element),
        "either-listener" => {
            replace_attribute(
                on(click, move |_| {
                    callback
                        .call0(&JsValue::UNDEFINED)
                        .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
                }),
                custom_attribute("data-second", "new"),
                &element,
            );
        }
        "vector-listener" => {
            let mut state = vec![
                custom_attribute("data-first", "old").into_any_attr(),
                on(click, move |_| {
                    callback
                        .call0(&JsValue::UNDEFINED)
                        .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
                })
                .into_any_attr(),
            ]
            .build(&element);
            vec![custom_attribute("data-second", "new").into_any_attr()].rebuild(&mut state);
        }
        _ => wasm_bindgen::throw_val(js_sys::TypeError::new("unknown attribute transition").into()),
    }
}

#[wasm_bindgen]
pub async fn reactive_attribute_transition(operation: &str, element: web_sys::Element) {
    use blog_dom::{
        either::Either,
        prelude::{ArcRwSignal, Get, Set},
        tasks::{Executor, ExecutorError},
        view_tree::html::attribute::{
            any_attribute::IntoAnyAttribute, custom::custom_attribute, Attribute,
        },
    };

    match Executor::init_wasm_bindgen() {
        Ok(()) | Err(ExecutorError::AlreadySet) => {}
    }

    async fn replace_either<A: Attribute>(
        old: A,
        element: &web_sys::Element,
        signal: ArcRwSignal<String>,
    ) {
        let mut state = Either::<A, ()>::Left(old).build(element);
        Either::<A, ()>::Right(()).rebuild(&mut state);
        signal.set(String::from("stale"));
        Executor::tick().await;
        drop(state);
    }

    let signal = ArcRwSignal::new(String::from("old"));
    let read = signal.clone();
    let attribute = custom_attribute("data-reactive", move || read.get());
    match operation {
        "either" => replace_either(attribute, &element, signal).await,
        "erased" => {
            let mut state = attribute.into_any_attr().build(&element);
            ().into_any_attr().rebuild(&mut state);
            signal.set(String::from("stale"));
            Executor::tick().await;
            drop(state);
        }
        _ => wasm_bindgen::throw_val(js_sys::TypeError::new("unknown reactive transition").into()),
    }
}
