
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn restrict_formats() {
        let mut schema = json_schema!({
            "$schema": meta_schemas::DRAFT2020_12,
            "anyOf": [
                { "format": "uuid" },
                { "$schema": meta_schemas::DRAFT07, "format": "uuid" },
                { "$schema": "http://unknown", "format": "uuid" },
                { "format": "date" },
                { "$schema": meta_schemas::DRAFT07, "format": "date" },
                { "$schema": "http://unknown", "format": "date" },
                { "format": "custom1" },
                { "$schema": meta_schemas::DRAFT07, "format": "custom1" },
                { "$schema": "http://unknown", "format": "custom1" },
                { "format": "custom2" },
                { "$schema": meta_schemas::DRAFT07, "format": "custom2" },
                { "$schema": "http://unknown", "format": "custom2" },
            ]
        });

        let mut transform = RestrictFormats::default();
        transform.allowed_formats.insert("custom1".into());
        transform.transform(&mut schema);

        assert_eq!(
            schema,
            json_schema!({
                "$schema": meta_schemas::DRAFT2020_12,
                "anyOf": [
                    { "format": "uuid" },
                    { "$schema": meta_schemas::DRAFT07 },
                    { "$schema": "http://unknown", "format": "uuid" },
                    { "format": "date" },
                    { "$schema": meta_schemas::DRAFT07, "format": "date" },
                    { "$schema": "http://unknown", "format": "date" },
                    { "format": "custom1" },
                    { "$schema": meta_schemas::DRAFT07, "format": "custom1" },
                    { "$schema": "http://unknown", "format": "custom1" },
                    { },
                    { "$schema": meta_schemas::DRAFT07 },
                    { "$schema": "http://unknown", "format": "custom2" },
                ]
            })
        );
    }
