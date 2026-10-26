//! Component constructors for the repository's browser CSR interface.
//!
//! DOM trees use ordinary Rust element builders. This crate deliberately has no
//! RSX, template, server, hydration, island, slot, or lazy-loading macro surface.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use component::{DummyModel, unmodified_fn_name_from_fn_name};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Visibility, spanned::Spanned, token::Pub};

mod builder;
mod component;
mod diagnostics;
mod names;
mod token_join;

/// Generates a typed props builder and component constructor.
///
/// Supports optional/default/into/renamed/destructured props, generic and
/// lifetime parameters, and `#[component(transparent)]`. The regular constructor
/// runs outside the caller's reactive observer; transparent constructors retain
/// that observer. Children use the same props builder as other values.
#[proc_macro_attribute]
pub fn component(args: TokenStream, item: TokenStream) -> TokenStream {
    component_impl(args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn component_impl(
    args: proc_macro::TokenStream,
    s: TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let is_transparent = if !args.is_empty() {
        let transparent = syn::parse::<syn::Ident>(args)?;

        if transparent != "transparent" {
            return Err(syn::Error::new_spanned(
                transparent,
                diagnostics::message_with_help(
                    "only `transparent` is supported",
                    "try `#[component(transparent)]` or `#[component]`",
                ),
            ));
        }

        true
    } else {
        false
    };

    component_macro(s, is_transparent)
}

fn component_macro(s: TokenStream, is_transparent: bool) -> syn::Result<proc_macro2::TokenStream> {
    let dummy = syn::parse::<DummyModel>(s.clone());
    let model = match syn::parse::<component::Model>(s) {
        Ok(model) => model,
        Err(error) => {
            let diagnostic = error.to_compile_error();
            return dummy.map(|mut dummy| {
                dummy.sig.ident = unmodified_fn_name_from_fn_name(&dummy.sig.ident);
                quote! {
                    #[doc(hidden)]
                    #[allow(clippy::too_many_arguments, clippy::needless_lifetimes)]
                    #dummy
                    #diagnostic
                }
            });
        }
    };
    let mut unexpanded = dummy?;
    let mut errors = diagnostics::Errors::default();
    let expanded = model.is_transparent(is_transparent).expand(&mut errors);
    if !matches!(unexpanded.vis, Visibility::Public(_)) {
        unexpanded.vis = Visibility::Public(Pub {
            span: unexpanded.vis.span(),
        })
    }
    unexpanded.sig.ident = unmodified_fn_name_from_fn_name(&unexpanded.sig.ident);

    let output = expanded.map(|expanded| {
        quote! {
            #expanded

            #[doc(hidden)]
            #[allow(clippy::too_many_arguments, clippy::needless_lifetimes)]
            #unexpanded
        }
    });
    errors.finish(output)
}

/// Generates the typed props builder used by component expansion.
#[proc_macro_derive(TypedBuilder, attributes(builder))]
pub fn typed_builder(input: TokenStream) -> TokenStream {
    builder::derive_typed_builder(input)
}
/// Joins identifiers in the generated reactive/view implementations.
#[proc_macro]
pub fn paste(input: TokenStream) -> TokenStream {
    token_join::paste(input)
}
