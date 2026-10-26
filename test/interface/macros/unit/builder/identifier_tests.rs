use super::*;

#[test]
fn invalid_builder_and_setter_names_are_diagnostics() {
    for source in [
        r#"#[builder(builder_type(name = "Bad Name"))] struct Form { value: u8 }"#,
        r#"struct Form { #[builder(setter(prefix = "bad prefix"))] value: u8 }"#,
        r#"struct Form { #[builder(setter(suffix = "!"))] value: u8 }"#,
        r#"struct Form { #[builder(setter(strip_option(fallback_prefix = "bad prefix")))] value: Option<u8> }"#,
    ] {
        let ast = syn::parse_str(source).expect("a valid derive input parses");
        let error = impl_my_derive(&ast).expect_err("invalid generated identifiers are rejected");
        assert!(error.to_string().contains("identifier"), "{error}");
    }
}

#[test]
fn raw_builder_names_and_valid_setter_affixes_generate_rust() {
    let ast = syn::parse_str(
        r#"#[builder(builder_type(name = r#type))]
        struct Form {
            #[builder(setter(prefix = "set_", suffix = "_now", strip_option(fallback_prefix = "maybe_")))]
            value: Option<u8>
        }"#,
    )
    .expect("a valid derive input parses");
    let expanded = impl_my_derive(&ast).expect("valid identifiers generate a builder");
    syn::parse2::<syn::File>(expanded).expect("builder declarations parse as Rust");
}
