
    use super::*;
    use pretty_assertions::assert_eq;
    use syn::DeriveInput;

    #[test]
    fn test_process_serde_attrs() {
        let mut input: DeriveInput = parse_quote! {
            #[serde(rename(serialize = "ser_name"), rename_all = "camelCase", from = "T")]
            #[serde(default, unknown_word)]
            #[schemars(rename = "overriden", another_unknown_word, !from)]
            #[misc]
            struct MyStruct {
                /// blah blah blah
                #[serde(skip_serializing_if = "some_fn", bound = "removed")]
                field1: i32,
                #[serde(serialize_with = "se", deserialize_with = "de")]
                #[schemars(with = "with", bound = "bound")]
                field2: i32,
                #[schemars(skip)]
                #[serde(skip_serializing)]
                field3: i32,
            }
        };
        let expected: DeriveInput = parse_quote! {
            #[schemars(rename = "overriden", another_unknown_word, !from)]
            #[misc]
            #[serde(rename = "overriden", rename_all = "camelCase", default)]
            struct MyStruct {
                #[doc = r" blah blah blah"]
                #[serde(skip_serializing_if = "some_fn")]
                field1: i32,
                #[schemars(with = "with", bound = "bound")]
                #[serde(bound = "bound", serialize_with = "se")]
                field2: i32,
                #[schemars(skip)]
                #[serde(skip)]
                field3: i32,
            }
        };

        if let Err(e) = process_serde_attrs(&mut input) {
            panic!("process_serde_attrs returned error: {e}")
        }

        assert_eq!(input, expected);
    }
