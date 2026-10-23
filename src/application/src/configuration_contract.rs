//! Build-time configuration materials derived from the runtime data type.
//!
//! The caller supplies its own current definition. No product source is read,
//! no process is started, and no installation state is created.

use schemars::JsonSchema;
use serde::Serialize;

mod rules;
pub use rules::validate_rules;

/// Generate the current JSON shape and defaults from one owned definition.
/// Product-specific semantic checks and path rules remain product authorities.
pub fn generate<T: JsonSchema + Serialize + Default>()
-> Result<serde_json::Value, serde_json::Error> {
    Ok(serde_json::json!({
        "schema": schemars::schema_for!(T),
        "defaults": serde_json::to_value(T::default())?,
    }))
}

#[path = "../../../test/application/unit/configuration_contract/tests.rs"]
#[cfg(test)]
mod tests;
