
    use std::string::String;
    use utoipa::PartialSchema;

    #[test]
    fn test_compact_string_schema_matches_string() {
        let compact_schema = <crate::CompactString as PartialSchema>::schema();
        let string_schema = <String as PartialSchema>::schema();

        assert!(compact_schema == string_schema);
    }
