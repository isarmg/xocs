use super::*;

#[test]
fn invalid_prop_names_return_errors_for_all_patterns() {
    for name in [
        "",
        "bad name",
        "a::b",
        "1first",
        "type",
        " value ",
        "value/*comment*/",
        "value//comment",
        "value\u{200e}",
        "value\u{200f}",
        "\u{feff}value",
    ] {
        for pattern in ["value", "(value,)"] {
            let arg: FnArg =
                syn::parse_str(&format!("#[prop(name = {name:?})] {pattern}: String")).unwrap();
            let error = Prop::new(arg)
                .err()
                .expect("invalid rename must be rejected");
            assert!(
                error.to_string().contains("valid Rust identifier"),
                "{error}"
            );
            assert!(
                error
                    .to_compile_error()
                    .to_string()
                    .contains("valid Rust identifier")
            );
        }
    }
}

#[test]
fn ordinary_unicode_and_raw_prop_names_remain_supported() {
    for name in ["renamed", "donnée", "e\u{301}", "r#type"] {
        let arg: FnArg =
            syn::parse_str(&format!("#[prop(name = {name:?})] value: String")).unwrap();
        let prop = Prop::new(arg).unwrap();
        assert_eq!(prop.name.ident.to_string(), name);
    }
}
