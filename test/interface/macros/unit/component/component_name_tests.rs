use super::*;

#[test]
fn raw_component_names_produce_valid_public_and_internal_identifiers() {
    for (name, public, internal) in [
        ("r#type", "Type", "__component_type"),
        ("r#AsyncView", "r#AsyncView", "__component_async_view"),
        ("snake_name", "SnakeName", "__component_snake_name"),
    ] {
        let model: Model = syn::parse_str(&format!("fn {name}() {{}}"))
            .expect("a valid component function parses");
        assert_eq!(model.name.to_string(), public);
        assert_eq!(
            unmodified_fn_name_from_fn_name(&model.body.sig.ident).to_string(),
            internal
        );
        let expanded = model.expand(&mut Errors::default()).unwrap();
        syn::parse2::<syn::File>(expanded).expect("generated declarations parse as Rust");
    }
}

#[test]
fn component_names_that_cannot_form_identifiers_produce_diagnostics() {
    for name in ["__", "___"] {
        let result = syn::parse_str::<Model>(&format!("fn {name}() {{}}"));
        let error = match result {
            Err(error) => error,
            Ok(model) => model.expand(&mut Errors::default()).unwrap_err(),
        };
        assert!(error.to_string().contains("component name"), "{error}");
    }
}
