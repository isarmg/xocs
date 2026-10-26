use std::cell::RefCell;
use web_sys::Url;

thread_local! { static LOCALE: RefCell<Option<String>> = const { RefCell::new(None) }; }

fn supported(value: &str) -> Option<&'static str> {
    let mut parts = value.split('-');
    let first = parts.next()?.to_ascii_lowercase();
    if parts.any(|part| {
        !(2..=8).contains(&part.len()) || !part.bytes().all(|b| b.is_ascii_alphanumeric())
    }) {
        return None;
    }
    match first.as_str() {
        "zh" => Some("zh-CN"),
        "en" => Some("en"),
        _ => None,
    }
}

pub fn locale() -> String {
    LOCALE.with(|saved| {
        if let Some(value) = saved.borrow().as_ref() {
            return value.clone();
        }
        let value = resolve();
        *saved.borrow_mut() = Some(value.clone());
        value
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve() -> String {
    "en".into()
}

#[cfg(target_arch = "wasm32")]
fn resolve() -> String {
    let Some(window) = web_sys::window() else {
        return "en".into();
    };
    if let Ok(url) = Url::new(&window.location().href().unwrap_or_default())
        && let Some(value) = url
            .search_params()
            .get("lang")
            .as_deref()
            .and_then(supported)
    {
        return value.into();
    }
    if let Ok(Some(storage)) = window.local_storage()
        && let Ok(Some(value)) = storage.get_item("sarmg.admin.language")
        && let Some(value) = supported(&value)
    {
        return value.into();
    }
    for value in window
        .navigator()
        .languages()
        .iter()
        .filter_map(|v| v.as_string())
    {
        if let Some(value) = supported(&value) {
            return value.into();
        }
    }
    "en".into()
}

pub fn t(zh: &str, en: &str) -> String {
    if locale() == "zh-CN" { zh } else { en }.to_owned()
}

pub fn tf(zh: &str, en: &str, values: &[&str]) -> String {
    let mut text = t(zh, en);
    for (index, value) in values.iter().enumerate() {
        text = text.replace(&format!("{{{index}}}"), value);
    }
    text
}

pub fn initialize() {
    let Some(window) = web_sys::window() else {
        return;
    };
    if let Some(document) = window.document()
        && let Some(root) = document.document_element()
    {
        root.set_attribute("lang", &locale())
            .expect("invariant: lang is a valid attribute name");
    }
    if let Ok(url) = Url::new(&window.location().href().unwrap_or_default())
        && url
            .search_params()
            .get("lang")
            .as_deref()
            .and_then(supported)
            .is_some()
        && let Ok(Some(storage)) = window.local_storage()
        && let Err(error) = storage.set_item("sarmg.admin.language", &locale())
    {
        web_sys::console::debug_2(
            &"Language preference remains limited to this document".into(),
            &error,
        );
    }
}

pub fn switch_language() {
    if let Some(window) = web_sys::window()
        && let Ok(url) = Url::new(&window.location().href().unwrap_or_default())
    {
        url.search_params()
            .set("lang", if locale() == "zh-CN" { "en" } else { "zh-CN" });
        if let Err(error) = window.location().assign(&url.href()) {
            web_sys::console::warn_2(&"Unable to switch the page language".into(), &error);
        }
    }
}

#[path = "../../../test/interface/unit/i18n/tests.rs"]
#[cfg(test)]
mod tests;
