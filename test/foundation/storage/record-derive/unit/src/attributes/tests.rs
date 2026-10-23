
    #[test]
    fn malformed_record_options_report_errors_instead_of_panicking() {
        for source in [
            "struct Row { #[sqlx(json(other))] value: String }",
            "struct Row { #[sqlx(json, flatten)] value: String }",
            "struct Row { #[sqlx(json(nullable), try_from=\"String\")] value: String }",
            "struct Row { #[sqlx(unknown)] value: String }",
            "struct Row { #[sqlx(rename=\"a\",rename=\"b\")] value: String }",
            "#[sqlx(rename_all=\"unknown\")] struct Row { value: String }",
        ] {
            let input = syn::parse_str(source).unwrap();
            assert!(
                crate::row::expand_derive_from_row(&input).is_err(),
                "{source}"
            );
        }
    }
