use js_sys::{Array, Intl, Object, Reflect};
use wasm_bindgen::JsValue;

pub(crate) fn get_timezone_inner() -> Result<String, crate::GetTimezoneError> {
    let intl = Intl::DateTimeFormat::new(&Array::new(), &Object::new()).resolved_options();
    Reflect::get(&intl, &JsValue::from_str("timeZone"))
        .ok()
        .and_then(|tz| tz.as_string())
        .ok_or(crate::GetTimezoneError::OsError)
}

#[path = "../../../../../test/foundation/platform/local-time-zone/unit/src/tz_wasm32_unknown/tests.rs"]
#[cfg(test)]
mod tests;
