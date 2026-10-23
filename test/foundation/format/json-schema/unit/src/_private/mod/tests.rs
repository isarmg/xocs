
    use pretty_assertions::assert_eq;

    #[test]
    fn nested_option_schemas() {
        let mut option_schema = schema_for!(Option<Result<i8, u8>>);
        option_schema.remove("title");
        let mut nested_option_schema = schema_for!(Option<Option<Option<Result<i8, u8>>>>);
        nested_option_schema.remove("title");

        assert_eq!(option_schema, nested_option_schema);
    }
