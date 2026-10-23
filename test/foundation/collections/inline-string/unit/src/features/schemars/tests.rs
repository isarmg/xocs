
    use alloc::string::String;

    use schemars::{JsonSchema, SchemaGenerator};

    use crate::CompactString;

    #[test]
    fn test_metadata_matches_string() {
        assert_eq!(CompactString::schema_name(), String::schema_name());
        assert_eq!(CompactString::schema_id(), String::schema_id());
        assert_eq!(
            CompactString::inline_schema(),
            String::inline_schema(),
            "inlining behavior must match String so nested schemas are identical",
        );
    }

    #[test]
    fn test_json_schema_matches_string() {
        let mut compact_gen = SchemaGenerator::default();
        let compact_schema = CompactString::json_schema(&mut compact_gen);

        let mut string_gen = SchemaGenerator::default();
        let string_schema = String::json_schema(&mut string_gen);

        assert_eq!(compact_schema, string_schema);
    }

    #[test]
    fn test_schema_describes_a_string() {
        let mut generator = SchemaGenerator::default();
        let schema = CompactString::json_schema(&mut generator);
        let value = serde_json::to_value(&schema).unwrap();

        assert_eq!(value.get("type").and_then(|t| t.as_str()), Some("string"));
    }

    // Even when generated as part of a larger document (which exercises the
    // reference/definition machinery via `subschema_for`), the output for a
    // `CompactString` field must be byte-for-byte identical to a `String` field.
    #[test]
    fn test_subschema_matches_string() {
        let mut compact_gen = SchemaGenerator::default();
        let compact_sub = compact_gen.subschema_for::<CompactString>();
        let compact_defs = compact_gen.into_root_schema_for::<CompactString>();

        let mut string_gen = SchemaGenerator::default();
        let string_sub = string_gen.subschema_for::<String>();
        let string_defs = string_gen.into_root_schema_for::<String>();

        assert_eq!(
            serde_json::to_value(&compact_sub).unwrap(),
            serde_json::to_value(&string_sub).unwrap(),
        );
        assert_eq!(
            serde_json::to_value(&compact_defs).unwrap(),
            serde_json::to_value(&string_defs).unwrap(),
        );
    }
