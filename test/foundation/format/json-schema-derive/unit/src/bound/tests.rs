
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_enum_bounds() {
        // All type params should be included in `JsonSchema` trait bounds except `Z`
        let input = parse_quote! {
            #[schemars(rename = "MyEnum<{T}, {U}, {V}, {W}, {X}, {Y}, {{Z}}>")]
            pub enum MyEnum<'a, const LEN: usize, T, U, V, W, X, Y, Z>
            where
                X: Trait,
                Z: OtherTrait
            {
                A,
                B(),
                C(T),
                D(U, (i8, V, bool)),
                E {
                    a: W,
                    b: [&'a Option<Box<<X as Trait>::AssocType::Z>>; LEN],
                    c: Token![Z],
                    d: PhantomData<Z>,
                    #[serde(skip)]
                    e: Z,
                },
                #[serde(skip)]
                F(Z),
            }
        };

        let cont = Container::from_ast(&input).unwrap();

        assert_eq!(
            cont.generics.where_clause,
            Some(parse_quote!(
                where
                    X: Trait,
                    Z: OtherTrait,
                    T: schemars::JsonSchema,
                    U: schemars::JsonSchema,
                    V: schemars::JsonSchema,
                    W: schemars::JsonSchema,
                    X: schemars::JsonSchema,
                    Y: schemars::JsonSchema
            ))
        );

        let relevant_type_params =
            Vec::from_iter(cont.relevant_type_params.into_iter().map(Ident::to_string));
        assert_eq!(relevant_type_params, vec!["T", "U", "V", "W", "X", "Y"]);
    }
