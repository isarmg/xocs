use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Expr, Field,
    Fields, FieldsNamed, FieldsUnnamed, Lifetime, Stmt,
};

use super::{
    attributes::{parse_child_attributes, parse_container_attributes, JsonAttribute},
    rename_all,
};

pub fn expand_derive_from_row(input: &DeriveInput) -> syn::Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => expand_derive_from_row_struct(input, named),

        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) => expand_derive_from_row_struct_unnamed(input, unnamed),

        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => Err(syn::Error::new_spanned(
            input,
            "unit structs are not supported",
        )),

        Data::Enum(_) => Err(syn::Error::new_spanned(input, "enums are not supported")),

        Data::Union(_) => Err(syn::Error::new_spanned(input, "unions are not supported")),
    }
}

fn expand_derive_from_row_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let row = row_parameter(input);
    // Mixed-site hygiene keeps generated locals distinct from legal field names
    // such as `__row` and `__default` in the caller's struct.
    let row_value = syn::Ident::new("__row", Span::mixed_site());
    let default_value = syn::Ident::new("__default", Span::mixed_site());

    let generics = &input.generics;

    let (lifetime, provided) = generics
        .lifetimes()
        .next()
        .map(|def| (def.lifetime.clone(), false))
        .unwrap_or_else(|| (Lifetime::new("'a", Span::call_site()), true));

    let (_, ty_generics, _) = generics.split_for_impl();

    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(#row: ::sqlx::Row));

    if provided {
        generics.params.insert(0, parse_quote!(#lifetime));
    }

    let predicates = &mut generics.make_where_clause().predicates;

    predicates.push(parse_quote!(&#lifetime ::std::primitive::str: ::sqlx::ColumnIndex<#row>));

    let container_attributes = parse_container_attributes(&input.attrs)?;

    let default_instance: Option<Stmt> = if container_attributes.default {
        predicates.push(parse_quote!(#ident #ty_generics: ::std::default::Default));
        Some(parse_quote!(
            let #default_value = <#ident #ty_generics as ::std::default::Default>::default();
        ))
    } else {
        None
    };

    let reads: Vec<Stmt> = fields
        .iter()
        .map(|field| -> syn::Result<Stmt> {
            let id = field.ident.as_ref().expect("named struct fields have identifiers");
            let attributes = parse_child_attributes(&field.attrs)?;
            let ty = &field.ty;

            if attributes.skip {
                predicates.push(parse_quote!(#ty: ::std::default::Default));
                return Ok(parse_quote!(
                    let #id: #ty = ::std::default::Default::default();
                ));
            }

            let id_s = if let Some(s) = attributes.rename {
                s
            } else {
                let s = id.to_string().trim_start_matches("r#").to_owned();
                match container_attributes.rename_all {
                    Some(pattern) => rename_all(&s, pattern),
                    None => s
                }
            };

            let expr: Expr = match (attributes.flatten, attributes.try_from, attributes.json) {
                // <No attributes>
                (false, None, None) => {
                    predicates
                        .push(parse_quote!(#ty: ::sqlx::decode::Decode<#lifetime, #row::Database>));
                    predicates.push(parse_quote!(#ty: ::sqlx::types::Type<#row::Database>));

                    parse_quote!(#row_value.try_get(#id_s))
                }
                // Flatten
                (true, None, None) => {
                    predicates.push(parse_quote!(#ty: ::sqlx::FromRow<#lifetime, #row>));
                    parse_quote!(<#ty as ::sqlx::FromRow<#lifetime, #row>>::from_row(#row_value))
                }
                // Flatten + Try from
                (true, Some(try_from), None) => {
                    predicates.push(parse_quote!(#try_from: ::sqlx::FromRow<#lifetime, #row>));
                    parse_quote!(
                        <#try_from as ::sqlx::FromRow<#lifetime, #row>>::from_row(#row_value)
                            .and_then(|v| {
                                <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v)
                                    .map_err(|e| {
                                        // Triggers a lint warning if `TryFrom::Err = Infallible`
                                        #[allow(unreachable_code)]
                                        ::sqlx::Error::ColumnDecode {
                                            index: #id_s.to_string(),
                                            source: sqlx::__spec_error!(e),
                                        }
                                    })
                            })
                    )
                }
                // Flatten + Json
                (true, _, Some(_)) => {
                    return Err(syn::Error::new_spanned(field, "Cannot use both flatten and json"))
                }
                // Try from
                (false, Some(try_from), None) => {
                    predicates
                        .push(parse_quote!(#try_from: ::sqlx::decode::Decode<#lifetime, #row::Database>));
                    predicates.push(parse_quote!(#try_from: ::sqlx::types::Type<#row::Database>));

                    parse_quote!(
                        #row_value.try_get(#id_s)
                            .and_then(|v| {
                                <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v)
                                    .map_err(|e| {
                                        // Triggers a lint warning if `TryFrom::Err = Infallible`
                                        #[allow(unreachable_code)]
                                        ::sqlx::Error::ColumnDecode {
                                            index: #id_s.to_string(),
                                            source: sqlx::__spec_error!(e),
                                        }
                                    })
                            })
                    )
                }
                // Try from + Json mandatory
                (false, Some(try_from), Some(JsonAttribute::NonNullable)) => {
                    predicates
                        .push(parse_quote!(::sqlx::types::Json<#try_from>: ::sqlx::decode::Decode<#lifetime, #row::Database>));
                    predicates.push(parse_quote!(::sqlx::types::Json<#try_from>: ::sqlx::types::Type<#row::Database>));

                    parse_quote!(
                        #row_value.try_get::<::sqlx::types::Json<_>, _>(#id_s)
                            .and_then(|v| {
                                <#ty as ::std::convert::TryFrom::<#try_from>>::try_from(v.0)
                                    .map_err(|e| {
                                        // Triggers a lint warning if `TryFrom::Err = Infallible`
                                        #[allow(unreachable_code)]
                                        ::sqlx::Error::ColumnDecode {
                                            index: #id_s.to_string(),
                                            source: sqlx::__spec_error!(e),
                                        }
                                    })
                            })
                    )
                },
                // Try from + Json nullable
                (false, Some(_), Some(JsonAttribute::Nullable)) => {
                    return Err(syn::Error::new_spanned(field, "Cannot use both try from and json nullable"))
                },
                // Json
                (false, None, Some(JsonAttribute::NonNullable)) => {
                    predicates
                        .push(parse_quote!(::sqlx::types::Json<#ty>: ::sqlx::decode::Decode<#lifetime, #row::Database>));
                    predicates.push(parse_quote!(::sqlx::types::Json<#ty>: ::sqlx::types::Type<#row::Database>));

                    parse_quote!(#row_value.try_get::<::sqlx::types::Json<_>, _>(#id_s).map(|x| x.0))
                },
                (false, None, Some(JsonAttribute::Nullable)) => {
                    predicates
                        .push(parse_quote!(::core::option::Option<::sqlx::types::Json<#ty>>: ::sqlx::decode::Decode<#lifetime, #row::Database>));
                    predicates.push(parse_quote!(::core::option::Option<::sqlx::types::Json<#ty>>: ::sqlx::types::Type<#row::Database>));

                    parse_quote!(#row_value.try_get::<::core::option::Option<::sqlx::types::Json<_>>, _>(#id_s).map(|x| x.and_then(|y| y.0)))
                },
            };

            if attributes.default {
                predicates.push(parse_quote!(#ty: ::std::default::Default));
                Ok(parse_quote!(
                    let #id: #ty = #expr.or_else(|e| match e {
                        ::sqlx::Error::ColumnNotFound(_) => {
                            ::std::result::Result::Ok(::std::default::Default::default())
                        },
                        e => ::std::result::Result::Err(e)
                    })?;
                ))
            } else if container_attributes.default {
                Ok(parse_quote!(
                    let #id: #ty = #expr.or_else(|e| match e {
                        ::sqlx::Error::ColumnNotFound(_) => {
                            ::std::result::Result::Ok(#default_value.#id)
                        },
                        e => ::std::result::Result::Err(e)
                    })?;
                ))
            } else {
                Ok(parse_quote!(
                    let #id: #ty = #expr?;
                ))
            }
        })
        .collect::<syn::Result<_>>()?;

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let names = fields.iter().map(|field| &field.ident);

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics ::sqlx::FromRow<#lifetime, #row> for #ident #ty_generics #where_clause {
            fn from_row(#row_value: &#lifetime #row) -> ::sqlx::Result<Self> {
                #default_instance

                #(#reads)*

                ::std::result::Result::Ok(#ident {
                    #(#names),*
                })
            }
        }
    ))
}

fn expand_derive_from_row_struct_unnamed(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let row = row_parameter(input);

    let generics = &input.generics;

    let (lifetime, provided) = generics
        .lifetimes()
        .next()
        .map(|def| (def.lifetime.clone(), false))
        .unwrap_or_else(|| (Lifetime::new("'a", Span::call_site()), true));

    let (_, ty_generics, _) = generics.split_for_impl();

    let mut generics = generics.clone();
    generics.params.insert(0, parse_quote!(#row: ::sqlx::Row));

    if provided {
        generics.params.insert(0, parse_quote!(#lifetime));
    }

    let predicates = &mut generics.make_where_clause().predicates;

    predicates.push(parse_quote!(
        ::std::primitive::usize: ::sqlx::ColumnIndex<#row>
    ));

    for field in fields {
        let ty = &field.ty;

        predicates.push(parse_quote!(#ty: ::sqlx::decode::Decode<#lifetime, #row::Database>));
        predicates.push(parse_quote!(#ty: ::sqlx::types::Type<#row::Database>));
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let gets = fields
        .iter()
        .enumerate()
        .map(|(idx, _)| quote!(row.try_get(#idx)?));

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics ::sqlx::FromRow<#lifetime, #row> for #ident #ty_generics #where_clause {
            fn from_row(row: &#lifetime #row) -> ::sqlx::Result<Self> {
                ::std::result::Result::Ok(#ident (
                    #(#gets),*
                ))
            }
        }
    ))
}

// Do not reserve a generic name in the consumer's record declaration.
fn row_parameter(input: &DeriveInput) -> syn::Ident {
    for index in 0.. {
        let candidate = format_ident!("__BlogRow{index}");
        let used = input
            .generics
            .params
            .iter()
            .any(|parameter| match parameter {
                syn::GenericParam::Type(parameter) => parameter.ident == candidate,
                syn::GenericParam::Const(parameter) => parameter.ident == candidate,
                syn::GenericParam::Lifetime(_) => false,
            });
        if !used {
            return candidate;
        }
    }
    unreachable!("finite generic parameters leave an available identifier")
}
