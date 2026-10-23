#![cfg_attr(docsrs, doc(cfg(feature = "schemars")))]

use alloc::borrow::Cow;
use alloc::string::String;

use schemars::{JsonSchema, Schema, SchemaGenerator};

use crate::CompactString;

// A `CompactString` serializes exactly like a `std::string::String`, so its JSON
// schema should be identical. We delegate every method to `String`'s impl to keep
// the two perfectly in sync (name, id, inlining behavior, and the schema itself).
impl JsonSchema for CompactString {
    fn inline_schema() -> bool {
        <String as JsonSchema>::inline_schema()
    }

    fn schema_name() -> Cow<'static, str> {
        <String as JsonSchema>::schema_name()
    }

    fn schema_id() -> Cow<'static, str> {
        <String as JsonSchema>::schema_id()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <String as JsonSchema>::json_schema(generator)
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/schemars/tests.rs"]
#[cfg(test)]
mod tests;
