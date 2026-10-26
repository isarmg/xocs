use super::*;

#[test]
fn attribute_values_preserve_nested_syntax_and_following_settings() {
    for (input, expected) in [
        (
            quote!(default = Vec::<u8>::new(), setter(into)),
            quote!(Vec::<u8>::new()),
        ),
        (
            quote!(default = <u8 as Default>::default(), setter(into)),
            quote!(<u8 as Default>::default()),
        ),
        (
            quote!(into = Option<(u8, u16)>, setter(into)),
            quote!(Option<(u8, u16)>),
        ),
    ] {
        let args = Punctuated::<AttrArg, Token![,]>::parse_terminated
            .parse2(input)
            .unwrap();
        assert_eq!(args.len(), 2);
        let AttrArg::KeyValue(value) = args.first().unwrap() else {
            panic!("expected a key-value setting");
        };
        assert_eq!(value.value.to_string(), expected.to_string());
        assert!(matches!(args.last(), Some(AttrArg::Sub(_))));
    }
}

#[test]
fn incomplete_attribute_values_return_syntax_errors() {
    for input in ["default =", "default = 1 2", "default = Vec<"] {
        assert!(syn::parse_str::<AttrArg>(input).is_err(), "{input}");
    }
}

#[test]
fn default_settings_reject_values_that_are_not_expressions() {
    let argument = syn::parse_str::<AttrArg>("default = value +").unwrap();
    let mut field = super::super::field_info::FieldBuilderAttr::default();
    assert!(field.apply_meta(argument).is_err());
}
