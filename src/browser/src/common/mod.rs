//! Shared browser primitives with separate request, navigation and UI ownership.
mod action;
mod decoration;
mod feedback;
mod forms;
mod lightbox;
mod navigation;
mod pagination;
mod request;

pub(crate) use action::Action;
pub(crate) use blog_interface::i18n::t;
pub(crate) use decoration::{Petals, Sliding};
pub(crate) use feedback::{Feedback, ListState, public_error};
pub(crate) use forms::{Check, ControlledCheck, ControlledField, Field, FilePicker};
pub(crate) use lightbox::Lightbox;
pub(crate) use navigation::{
    browse, enc, hash_signal, href, navigate, page_number, param, pathname, query, safe_url,
    search_signal,
};
pub(crate) use pagination::{BrowsePager, Pager};
pub(crate) use request::{ApiError, Data, RequestOptions, invalid, load, load_typed, request};
use serde_json::Value;

pub(crate) fn tr(zh: &str, en: &str, args: &[String]) -> String {
    let mut out = t(zh, en);
    for (i, arg) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), arg);
    }
    out
}

pub(crate) fn text(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(crate) fn num(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}

pub(crate) fn flag(v: &Value, k: &str) -> bool {
    v[k].as_bool().unwrap_or_else(|| v[k].as_i64() == Some(1))
}

pub(crate) fn rows(v: &Value, k: &str) -> Vec<Value> {
    v.get(k)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub(crate) fn list(v: Option<Value>) -> Vec<Value> {
    v.and_then(|v| v.as_array().cloned()).unwrap_or_default()
}

pub(crate) fn table_time(value: &str) -> String {
    let formatted = crate::dates::format(value);
    if formatted.is_empty() {
        "—".into()
    } else {
        formatted
    }
}
