use crate::diagnostics::{Errors, message_with_help};
use crate::names::{pascal, snake};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt, format_ident, quote, quote_spanned};
use syn::{
    AngleBracketedGenericArguments, Attribute, FnArg, GenericArgument, GenericParam, Item, ItemFn,
    LitStr, Meta, Pat, PatIdent, Path, PathArguments, ReturnType, Signature, Stmt, Type,
    TypeImplTrait, TypeParam, TypePath, Visibility, parse::Parse, parse_quote, spanned::Spanned,
    token::Colon, visit_mut::VisitMut,
};

pub struct Model {
    validation_error: Option<syn::Error>,
    is_transparent: bool,
    docs: Docs,
    unknown_attrs: UnknownAttrs,
    vis: Visibility,
    name: Ident,
    props: Vec<Prop>,
    body: ItemFn,
    ret: ReturnType,
}

impl Parse for Model {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut item = ItemFn::parse(input)?;

        convert_impl_trait_to_generic(&mut item.sig);

        let (docs, mut validation_error) = match Docs::new(&item.attrs) {
            Ok(docs) => (docs, None),
            Err(error) => (Docs(Vec::new()), Some(error)),
        };
        let unknown_attrs = UnknownAttrs::new(&item.attrs);

        let props = if validation_error.is_none() {
            match item
                .sig
                .inputs
                .clone()
                .into_iter()
                .map(Prop::new)
                .collect::<syn::Result<Vec<_>>>()
            {
                Ok(props) => props,
                Err(error) => {
                    validation_error = Some(error);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // We need to remove the `#[doc = ""]` and `#[builder(_)]`
        // attrs from the function signature
        drain_filter(&mut item.attrs, |attr| match &attr.meta {
            Meta::NameValue(attr) => attr.path == parse_quote!(doc),
            Meta::List(attr) => attr.path == parse_quote!(prop),
            _ => false,
        });
        item.sig.inputs.iter_mut().for_each(|arg| {
            if let FnArg::Typed(ty) = arg {
                drain_filter(&mut ty.attrs, |attr| match &attr.meta {
                    Meta::NameValue(attr) => attr.path == parse_quote!(doc),
                    Meta::List(attr) => attr.path == parse_quote!(prop),
                    _ => false,
                });
            }
        });

        Ok(Self {
            validation_error,
            is_transparent: false,
            docs,
            unknown_attrs,
            vis: item.vis.clone(),
            name: convert_from_snake_case(&item.sig.ident),
            props,
            ret: item.sig.output.clone(),
            body: item,
        })
    }
}

// implemented manually because Vec::drain_filter is nightly only
// follows std recommended parallel
pub fn drain_filter<T>(vec: &mut Vec<T>, mut some_predicate: impl FnMut(&mut T) -> bool) {
    let mut i = 0;
    while i < vec.len() {
        if some_predicate(&mut vec[i]) {
            _ = vec.remove(i);
        } else {
            i += 1;
        }
    }
}

pub fn convert_from_snake_case(name: &Ident) -> Ident {
    let name_str = name.to_string();
    if name_str != snake(&name_str) {
        name.clone()
    } else {
        Ident::new(&pascal(&name_str), name.span())
    }
}

impl Model {
    pub(crate) fn expand(mut self, errors: &mut Errors) -> syn::Result<TokenStream> {
        if let Some(error) = self.validation_error.take() {
            return Err(error);
        }
        let Self {
            validation_error: _,
            is_transparent,
            docs,
            unknown_attrs,
            vis,
            name,
            props,
            body,
            ret,
        } = &self;
        if !is_transparent {
            let ends_semi = body.block.stmts.last().and_then(|stmt| match stmt {
                Stmt::Item(Item::Macro(mac)) => mac.semi_token.as_ref(),
                _ => None,
            });
            if let Some(semi) = ends_semi {
                errors.push(syn::Error::new(
                    semi.span(),
                    "A component ending in a macro followed by a semicolon returns an empty view. Return () explicitly if that is intended.",
                ));
            }
        }
        let (impl_generics, generics, where_clause) = body.sig.generics.split_for_impl();
        let props_name = format_ident!("{name}Props");
        let props_builder_name = format_ident!("{name}PropsBuilder");
        let prop_builder_fields = prop_builder_fields(vis, props)?;
        let prop_names = prop_names(props);
        let builder_name_doc = LitStr::new(
            &format!(" Props for the [`{name}`] component."),
            name.span(),
        );
        let component_fn_prop_docs = generate_component_fn_prop_docs(props)?;
        let docs_and_prop_docs = if component_fn_prop_docs.is_empty() {
            quote! { #docs }
        } else {
            quote! { #docs #[doc = ""] #component_fn_prop_docs }
        };
        let body_name = unmodified_fn_name_from_fn_name(&body.sig.ident);
        let body_expr = quote! { #body_name(#prop_names) };
        let component = if *is_transparent {
            body_expr
        } else {
            quote! {
                ::blog_interface::reactive::graph::untrack_with_diagnostics(move || {
                    #body_expr
                })
            }
        };
        let (props_arg, destructure_props) = if props.is_empty() {
            (quote! {}, quote! {})
        } else {
            (
                quote! { props: #props_name #generics },
                quote! { let #props_name { #prop_names } = props; },
            )
        };
        Ok(quote! {
            #[doc = #builder_name_doc]
            #[doc = ""]
            #docs_and_prop_docs
            #[derive(::blog_interface::typed_builder_macro::TypedBuilder)]
            #[builder(crate_module_path=::blog_interface::typed_builder)]
            #[allow(non_snake_case)]
            #vis struct #props_name #impl_generics #where_clause {
                #prop_builder_fields
            }
            impl #impl_generics ::blog_interface::component::Props for #props_name #generics #where_clause {
                type Builder = #props_builder_name #generics;
                fn builder() -> Self::Builder { #props_name::builder() }
            }
            #unknown_attrs
            #docs_and_prop_docs
            #[allow(non_snake_case, clippy::too_many_arguments)]
            #[allow(clippy::needless_lifetimes)]
            #vis fn #name #impl_generics (#props_arg) #ret #where_clause {
                #destructure_props
                #component
            }
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn is_transparent(mut self, is_transparent: bool) -> Self {
        self.is_transparent = is_transparent;
        self
    }
}

/// A model that is more lenient in case of a syntax error in the function body,
/// but does not actually implement the behavior of the real model. This is
/// used to improve IDEs and rust-analyzer's auto-completion behavior in case
/// of a syntax error.
pub struct DummyModel {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub sig: Signature,
    pub body: TokenStream,
}

impl Parse for DummyModel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        // Drop unknown attributes like #[deprecated]
        drain_filter(&mut attrs, |attr| {
            !is_lint_attr(attr) && !attr.path().is_ident("doc")
        });

        let vis: Visibility = input.parse()?;
        let sig: Signature = input.parse()?;

        // The body is left untouched, so it will not cause an error
        // even if the syntax is invalid.
        let body: TokenStream = input.parse()?;

        Ok(Self {
            attrs,
            vis,
            sig,
            body,
        })
    }
}

impl ToTokens for DummyModel {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            attrs,
            vis,
            sig,
            body,
        } = self;

        // Strip attributes like documentation comments and #[prop]
        // from the signature, so as to not confuse the user with incorrect
        // error messages.
        let sig = {
            let mut sig = sig.clone();
            sig.inputs.iter_mut().for_each(|arg| {
                if let FnArg::Typed(ty) = arg {
                    ty.attrs.retain(|attr| match &attr.meta {
                        Meta::List(list) => list
                            .path
                            .segments
                            .first()
                            .map(|n| n.ident != "prop")
                            .unwrap_or(true),
                        Meta::NameValue(name_value) => name_value
                            .path
                            .segments
                            .first()
                            .map(|n| n.ident != "doc")
                            .unwrap_or(true),
                        _ => true,
                    });
                }
            });
            sig
        };

        let output = quote! {
            #(#attrs)*
            #vis #sig #body
        };

        tokens.append_all(output)
    }
}

struct Prop {
    docs: Docs,
    prop_opts: PropOpt,
    name: PatIdent,
    ty: Type,
}

fn prop_identifier(name: &str, span: Span) -> syn::Result<Ident> {
    let invalid = || {
        syn::Error::new(
            span,
            "prop name must be a valid Rust identifier; use r#name for a keyword",
        )
    };
    // parse_str consumes exactly one identifier but permits surrounding trivia.
    // Reject trivia explicitly: compiler identifiers normalize NFD to NFC, so
    // comparing their printed spelling to the original string would reject
    // valid Unicode names even though fallback parser tests accept them.
    if name.chars().any(|character| {
        character.is_whitespace() || matches!(character, '/' | '\u{200e}' | '\u{200f}' | '\u{feff}')
    }) {
        return Err(invalid());
    }
    let mut identifier = syn::parse_str::<Ident>(name).map_err(|_| invalid())?;
    identifier.set_span(span);
    Ok(identifier)
}

impl Prop {
    fn new(arg: FnArg) -> syn::Result<Self> {
        let typed = if let FnArg::Typed(ty) = arg {
            ty
        } else {
            return Err(syn::Error::new_spanned(arg, "receiver not allowed in `fn`"));
        };

        let prop_opts = PropOpt::from_attributes(&typed.attrs)?;

        let name = match *typed.pat {
            Pat::Ident(i) => {
                if let Some((name, span)) = &prop_opts.name {
                    PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: prop_identifier(name, *span)?,
                        subpat: None,
                    }
                } else {
                    i
                }
            }
            pat @ (Pat::Struct(_) | Pat::Tuple(_) | Pat::TupleStruct(_)) => {
                let Some((name, span)) = prop_opts.name.as_ref() else {
                    return Err(syn::Error::new_spanned(
                        pat,
                        "destructured props must be given a name e.g. \
                         #[prop(name = \"data\")]",
                    ));
                };
                PatIdent {
                    attrs: vec![],
                    by_ref: None,
                    mutability: None,
                    ident: prop_identifier(name, *span)?,
                    subpat: None,
                }
            }
            pat => {
                return Err(syn::Error::new_spanned(
                    pat,
                    "only `prop: bool` style types are allowed within the \
                     `#[component]` macro",
                ));
            }
        };

        Ok(Self {
            docs: Docs::new(&typed.attrs)?,
            prop_opts,
            name,
            ty: *typed.ty,
        })
    }
}

#[derive(Clone)]
pub struct Docs(Vec<(String, Span)>);

impl ToTokens for Docs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = self
            .0
            .iter()
            .map(|(doc, span)| quote_spanned!(*span=> #[doc = #doc]))
            .collect::<TokenStream>();

        tokens.append_all(s);
    }
}

impl Docs {
    pub fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        let docs = attrs
            .iter()
            .filter_map(|attribute| {
                let Meta::NameValue(attribute) = &attribute.meta else {
                    return None;
                };
                if !attribute.path.is_ident("doc") {
                    return None;
                }
                Some(
                    value_to_string(&attribute.value)
                        .map(|value| (value, attribute.path.span()))
                        .ok_or_else(|| {
                            syn::Error::new_spanned(
                                attribute,
                                "expected string literal in value of doc comment",
                            )
                        }),
                )
            })
            .collect::<syn::Result<Vec<_>>>()?;
        Ok(Self(docs))
    }

    pub fn padded(&self) -> TokenStream {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, (doc, span))| {
                let doc = if idx == 0 {
                    format!("    - {doc}")
                } else {
                    format!("      {doc}")
                };

                let doc = LitStr::new(&doc, *span);

                quote! { #[doc = #doc] }
            })
            .collect()
    }

    pub fn typed_builder(&self) -> String {
        let doc_str = self
            .0
            .iter()
            .map(|s| s.0.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        if doc_str.chars().filter(|c| *c != '\n').count() != 0 {
            format!("\n\n{doc_str}")
        } else {
            String::new()
        }
    }
}

fn is_lint_attr(attr: &Attribute) -> bool {
    let path = &attr.path();
    path.is_ident("allow")
        || path.is_ident("warn")
        || path.is_ident("expect")
        || path.is_ident("deny")
        || path.is_ident("forbid")
}

pub struct UnknownAttrs(Vec<(TokenStream, Span)>);

impl UnknownAttrs {
    pub fn new(attrs: &[Attribute]) -> Self {
        let attrs = attrs
            .iter()
            .filter_map(|attr| {
                if attr.path().is_ident("doc")
                    && let Meta::NameValue(_) = &attr.meta
                {
                    return None;
                }

                if is_lint_attr(attr) {
                    return None;
                }

                Some((attr.into_token_stream(), attr.span()))
            })
            .collect::<Vec<_>>();
        Self(attrs)
    }
}

impl ToTokens for UnknownAttrs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let s = self
            .0
            .iter()
            .map(|(attr, span)| quote_spanned!(*span=> #attr))
            .collect::<TokenStream>();
        tokens.append_all(s);
    }
}

#[derive(Clone, Debug, Default)]
struct PropOpt {
    optional: bool,
    optional_no_strip: bool,
    strip_option: bool,
    default: Option<syn::Expr>,
    into: bool,
    attrs: bool,
    name: Option<(String, Span)>,
    marker: bool,
}

impl PropOpt {
    fn from_attributes(attributes: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        let mut seen = std::collections::HashMap::new();
        for attribute in attributes
            .iter()
            .filter(|attr| attr.path().is_ident("prop"))
        {
            attribute.parse_nested_meta(|meta| {
                let ident = meta
                    .path
                    .get_ident()
                    .ok_or_else(|| meta.error("expected a prop option"))?;
                let key = ident.to_string();
                if seen.insert(key.clone(), ident.span()).is_some() {
                    return Err(meta.error(format!("duplicate prop option `{key}`")));
                }
                let flag = |meta: &syn::meta::ParseNestedMeta<'_>| -> syn::Result<bool> {
                    if meta.input.peek(syn::Token![=]) {
                        Ok(meta.value()?.parse::<syn::LitBool>()?.value)
                    } else {
                        Ok(true)
                    }
                };
                match key.as_str() {
                    "optional" => result.optional = flag(&meta)?,
                    "optional_no_strip" => result.optional_no_strip = flag(&meta)?,
                    "strip_option" => result.strip_option = flag(&meta)?,
                    "into" => result.into = flag(&meta)?,
                    "attrs" => result.attrs = flag(&meta)?,
                    "marker" => result.marker = flag(&meta)?,
                    "default" => result.default = Some(meta.value()?.parse()?),
                    "name" => {
                        let value = meta.value()?.parse::<LitStr>()?;
                        result.name = Some((value.value(), value.span()));
                    }
                    _ => return Err(meta.error(format!("unknown prop option `{key}`"))),
                }
                Ok(())
            })?;
        }
        for keys in [
            &["optional", "optional_no_strip", "strip_option"][..],
            &[
                "marker",
                "optional",
                "optional_no_strip",
                "strip_option",
                "default",
                "into",
                "attrs",
                "name",
            ][..],
        ] {
            let active = |key: &str| match key {
                "optional" => result.optional,
                "optional_no_strip" => result.optional_no_strip,
                "strip_option" => result.strip_option,
                "marker" => result.marker,
                "default" => result.default.is_some(),
                "into" => result.into,
                "attrs" => result.attrs,
                "name" => result.name.is_some(),
                _ => false,
            };
            let conflict = if keys[0] == "marker" {
                result.marker && keys[1..].iter().any(|key| active(key))
            } else {
                keys.iter().filter(|key| active(key)).count() > 1
            };
            if conflict {
                let key = keys
                    .iter()
                    .find(|key| active(key))
                    .expect("conflicting prop options");
                return Err(syn::Error::new(seen[*key], "conflicting prop options"));
            }
        }
        Ok(result)
    }
}

struct TypedBuilderOpts<'a> {
    marker: bool,
    default: bool,
    default_with_value: Option<syn::Expr>,
    strip_option: bool,
    into: bool,
    ty: &'a Type,
}

impl<'a> TypedBuilderOpts<'a> {
    fn from_opts(opts: &PropOpt, ty: &'a Type) -> Self {
        Self {
            marker: opts.marker,
            default: opts.optional || opts.optional_no_strip || opts.attrs,
            default_with_value: opts.default.clone(),
            strip_option: opts.strip_option || opts.optional && is_option(ty),
            into: opts.into,
            ty,
        }
    }
}

impl TypedBuilderOpts<'_> {
    fn expand(&self) -> syn::Result<TokenStream> {
        if self.marker {
            return Ok(quote! {
                #[builder(default, setter(skip))]
            });
        }

        let default = if let Some(v) = &self.default_with_value {
            let v = v.to_token_stream().to_string();
            quote! { default_code=#v, }
        } else if self.default {
            quote! { default, }
        } else {
            quote! {}
        };

        // If self.strip_option && self.into, then the strip_option will be represented as part of the transform closure.
        let strip_option = if self.strip_option && !self.into {
            quote! { strip_option, }
        } else {
            quote! {}
        };

        let into = if self.into {
            if !self.strip_option {
                let ty = &self.ty;
                quote! {
                    fn transform<__IntoReactiveValueMarker>(value: impl ::blog_interface::prelude::IntoReactiveValue<#ty, __IntoReactiveValueMarker>) -> #ty {
                        value.into_reactive_value()
                    },
                }
            } else {
                let ty = unwrap_option(self.ty)?;
                quote! {
                    fn transform<__IntoReactiveValueMarker>(value: impl ::blog_interface::prelude::IntoReactiveValue<#ty, __IntoReactiveValueMarker>) -> Option<#ty> {
                        Some(value.into_reactive_value())
                    },
                }
            }
        } else {
            quote! {}
        };

        let setter = if !strip_option.is_empty() || !into.is_empty() {
            quote! { setter(#strip_option #into) }
        } else {
            quote! {}
        };

        let output = if !default.is_empty() || !setter.is_empty() {
            quote! { #[builder(#default #setter)] }
        } else {
            quote! {}
        };

        Ok(output)
    }
}

fn prop_builder_fields(vis: &Visibility, props: &[Prop]) -> syn::Result<TokenStream> {
    props
        .iter()
        .map(|prop| {
            let Prop {
                docs,
                name,
                prop_opts,
                ty,
            } = prop;

            let PatIdent { ident, by_ref, .. } = &name;

            let builder_attrs = TypedBuilderOpts::from_opts(prop_opts, ty).expand()?;

            let builder_docs = prop_to_doc(prop, PropDocStyle::Inline)?;

            // Children and markers won't need documentation in many cases
            let allow_missing_docs = if prop_opts.marker || ident == "children" {
                quote!(#[allow(missing_docs)])
            } else {
                quote!()
            };
            Ok(quote! {
                #docs
                #builder_docs
                #builder_attrs
                #allow_missing_docs
                #vis #by_ref #ident: #ty,
            })
        })
        .collect()
}

fn prop_names(props: &[Prop]) -> TokenStream {
    props
        .iter()
        .map(|Prop { name, .. }| {
            // fields like mutability are removed because unneeded
            // in the contexts in which this is used
            let ident = &name.ident;
            quote! { #ident, }
        })
        .collect()
}

fn generate_component_fn_prop_docs(props: &[Prop]) -> syn::Result<TokenStream> {
    let required_prop_docs = props
        .iter()
        .filter(|Prop { prop_opts, .. }| {
            !(prop_opts.optional || prop_opts.optional_no_strip || prop_opts.default.is_some())
        })
        .filter(|prop| !prop.prop_opts.marker)
        .map(|p| prop_to_doc(p, PropDocStyle::List))
        .collect::<syn::Result<TokenStream>>()?;

    let optional_prop_docs = props
        .iter()
        .filter(|Prop { prop_opts, .. }| {
            prop_opts.optional || prop_opts.optional_no_strip || prop_opts.default.is_some()
        })
        .filter(|prop| !prop.prop_opts.marker)
        .map(|p| prop_to_doc(p, PropDocStyle::List))
        .collect::<syn::Result<TokenStream>>()?;

    let required_prop_docs = if !required_prop_docs.is_empty() {
        quote! {
            #[doc = " # Required Props"]
            #required_prop_docs
        }
    } else {
        quote! {}
    };

    let optional_prop_docs = if !optional_prop_docs.is_empty() {
        quote! {
            #[doc = " # Optional Props"]
            #optional_prop_docs
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #required_prop_docs
        #optional_prop_docs
    })
}

pub fn is_option(ty: &Type) -> bool {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let [first] = &segments.iter().collect::<Vec<_>>()[..] {
            first.ident == "Option"
        } else {
            false
        }
    } else {
        false
    }
}

pub fn unwrap_option(ty: &Type) -> syn::Result<Type> {
    const STD_OPTION_MSG: &str = "make sure you're not shadowing the `std::option::Option` type that \
         is automatically imported from the standard prelude";

    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
        && let [first] = &segments.iter().collect::<Vec<_>>()[..]
        && first.ident == "Option"
        && let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
            &first.arguments
        && let [GenericArgument::Type(ty)] = &args.iter().collect::<Vec<_>>()[..]
    {
        return Ok(ty.clone());
    }

    Err(syn::Error::new_spanned(
        ty,
        message_with_help("`Option` must be `std::option::Option`", STD_OPTION_MSG),
    ))
}

#[derive(Clone, Copy)]
enum PropDocStyle {
    List,
    Inline,
}

fn prop_to_doc(
    Prop {
        docs,
        name,
        ty,
        prop_opts,
    }: &Prop,
    style: PropDocStyle,
) -> syn::Result<TokenStream> {
    let ty = if (prop_opts.optional || prop_opts.strip_option) && is_option(ty) {
        unwrap_option(ty)?
    } else {
        ty.to_owned()
    };

    // Documentation displays this already-parsed type; a second full Rust
    // source printer is unnecessary for either generated code or validation.
    let pretty_ty = ty.to_token_stream().to_string();

    match style {
        PropDocStyle::List => {
            let arg_ty_doc = LitStr::new(
                &if !prop_opts.into {
                    format!(" - **{}**: [`{pretty_ty}`]", quote!(#name))
                } else {
                    format!(
                        " - **{}**: [`impl Into<{pretty_ty}>`]({pretty_ty})",
                        quote!(#name),
                    )
                },
                name.ident.span(),
            );

            let arg_user_docs = docs.padded();

            Ok(quote! {
                #[doc = #arg_ty_doc]
                #arg_user_docs
            })
        }
        PropDocStyle::Inline => {
            let arg_ty_doc = LitStr::new(
                &if !prop_opts.into {
                    format!(
                        "**{}**: [`{}`]{}",
                        quote!(#name),
                        pretty_ty,
                        docs.typed_builder()
                    )
                } else {
                    format!(
                        "**{}**: `impl`[`Into<{}>`]{}",
                        quote!(#name),
                        pretty_ty,
                        docs.typed_builder()
                    )
                },
                name.ident.span(),
            );

            Ok(quote! {
                #[builder(setter(doc = #arg_ty_doc))]
            })
        }
    }
}

pub fn unmodified_fn_name_from_fn_name(ident: &Ident) -> Ident {
    Ident::new(
        &format!("__component_{}", snake(&ident.to_string())),
        ident.span(),
    )
}

/// Converts all `impl Trait`s in a function signature to use generic params instead.
fn convert_impl_trait_to_generic(sig: &mut Signature) {
    fn new_generic_ident(i: usize, span: Span) -> Ident {
        Ident::new(&format!("__ImplTrait{i}"), span)
    }

    // First: visit all `impl Trait`s and replace them with new generic params.
    #[derive(Default)]
    struct RemoveImplTrait(Vec<TypeImplTrait>);
    impl VisitMut for RemoveImplTrait {
        fn visit_type_mut(&mut self, ty: &mut Type) {
            syn::visit_mut::visit_type_mut(self, ty);
            if matches!(ty, Type::ImplTrait(_)) {
                let ident = new_generic_ident(self.0.len(), ty.span());
                let generic_type = Type::Path(TypePath {
                    attrs: Vec::new(),
                    qself: None,
                    path: Path::from(ident),
                });
                let Type::ImplTrait(impl_trait) = std::mem::replace(ty, generic_type) else {
                    unreachable!();
                };
                self.0.push(impl_trait);
            }
        }

        // Early exits.
        fn visit_attribute_mut(&mut self, _: &mut Attribute) {}
        fn visit_pat_mut(&mut self, _: &mut Pat) {}
    }
    let mut visitor = RemoveImplTrait::default();
    for fn_arg in sig.inputs.iter_mut() {
        visitor.visit_fn_arg_mut(fn_arg);
    }
    let RemoveImplTrait(impl_traits) = visitor;

    // Second: Add the new generic params into the signature.
    for (i, impl_trait) in impl_traits.into_iter().enumerate() {
        let span = impl_trait.span();
        let ident = new_generic_ident(i, span);
        // We can simply append to the end (only lifetime params must be first).
        // Note currently default generics are not allowed in `fn`, so not a concern.
        sig.generics.params.push(GenericParam::Type(TypeParam {
            attrs: vec![],
            ident,
            colon_token: Some(Colon { spans: [span] }),
            bounds: impl_trait.bounds,
            default: None,
        }));
    }
}

// Literal extraction formerly supplied by leptos_hot_reload::parsing.
// The original MIT source and archive identity remain in compliance provenance.
fn value_to_string(value: &syn::Expr) -> Option<String> {
    match value {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => Some(s.value()),
            syn::Lit::Char(c) => Some(c.value().to_string()),
            syn::Lit::Int(i) => Some(i.base10_digits().to_string()),
            syn::Lit::Float(f) => Some(f.base10_digits().to_string()),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod maintained_doc_tests {
    use super::*;

    #[test]
    fn component_docs_preserve_markdown_and_rust_examples() {
        let item: syn::ItemFn = syn::parse_quote! {
            /// A component example.
            /// ```rust
            /// let value = "<literal>";
            /// assert_eq!(value, "<literal>");
            /// ```
            fn example() {}
        };
        let docs = Docs::new(&item.attrs).unwrap();
        let lines: Vec<_> = docs.0.iter().map(|(line, _)| line.as_str()).collect();
        assert_eq!(
            lines,
            [
                " A component example.",
                " ```rust",
                " let value = \"<literal>\";",
                " assert_eq!(value, \"<literal>\");",
                " ```"
            ]
        );
        assert!(!docs.to_token_stream().to_string().contains("view !"));
    }
}

#[cfg(test)]
mod prop_name_tests {
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
}
