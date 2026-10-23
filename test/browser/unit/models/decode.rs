/// Decode only after the same strict response contract used by dynamic forms.
#[cfg(test)]
pub fn decode<T: serde::de::DeserializeOwned>(
    path: &str,
    method: &str,
    value: serde_json::Value,
) -> Result<T, ()> {
    if !crate::contracts::validate(path, method, &value)
        || method == "GET" && !crate::contracts::matches_page(path, &value)
    {
        return Err(());
    }
    serde_json::from_value(value).map_err(|_| ())
}
