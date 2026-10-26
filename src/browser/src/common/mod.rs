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
pub(crate) use request::{
    ApiError, ApiErrorCode, Data, LocalErrorCode, RequestOptions, invalid, load, load_typed,
    request,
};
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
    v[k].as_i64()
        .expect("invariant: a required integer passed the response contract or form initialization")
}

pub(crate) fn nullable_num(v: &Value, k: &str) -> Option<i64> {
    let value = v
        .get(k)
        .expect("invariant: a required nullable integer field is present");
    if value.is_null() {
        None
    } else {
        Some(num(v, k))
    }
}

pub(crate) fn flag(v: &Value, k: &str) -> bool {
    match v
        .get(k)
        .expect("invariant: a flag passed the response contract or form initialization")
    {
        Value::Bool(value) => *value,
        Value::Number(value) => {
            value
                .as_i64()
                .expect("invariant: response flags are binary integers")
                == 1
        }
        // Nullable publication status is displayed as unpublished.
        Value::Null => false,
        Value::String(_) | Value::Array(_) | Value::Object(_) => {
            unreachable!("invariant: a response or form flag is boolean, binary or null")
        }
    }
}

pub(crate) fn rows(v: &Value, k: &str) -> Vec<Value> {
    v[k].as_array()
        .expect("invariant: list items passed the response contract")
        .clone()
}

pub(crate) fn list(v: Option<Value>) -> Vec<Value> {
    match v {
        None => Vec::new(),
        Some(Value::Array(items)) => items,
        Some(_) => unreachable!("invariant: an available list passed the response contract"),
    }
}

pub(crate) fn table_time(value: &str) -> String {
    let formatted = crate::dates::format(value);
    if formatted.is_empty() {
        "—".into()
    } else {
        formatted
    }
}
