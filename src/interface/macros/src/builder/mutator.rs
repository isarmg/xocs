use std::collections::HashSet;
use syn_legacy as syn;

use proc_macro2::Ident;
use syn::{
    Error, Expr, FnArg, ItemFn, PatIdent, ReturnType, Signature, Token, Type,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::builder::util::{ApplyMeta, AttrArg, pat_to_ident};

#[derive(Debug, Clone)]
pub struct Mutator {
    pub fun: ItemFn,
    pub required_fields: HashSet<Ident>,
}

#[derive(Default)]
struct MutatorAttribute {
    requires: HashSet<Ident>,
}

impl ApplyMeta for MutatorAttribute {
    fn apply_meta(&mut self, expr: AttrArg) -> Result<(), Error> {
        if expr.name() != "requires" {
            return Err(Error::new_spanned(
                expr.name(),
                "Only `requires` is supported",
            ));
        }

        let value = expr.key_value()?.parse_value()?;
        let Expr::Array(syn::ExprArray { elems, .. }) = value else {
            return Err(Error::new_spanned(
                value,
                "Only list of field names [field1, field2, …] supported",
            ));
        };
        for expr in elems {
            let Expr::Path(path) = expr else {
                return Err(Error::new_spanned(expr, "Expected field name"));
            };
            let name = path
                .path
                .get_ident()
                .cloned()
                .ok_or_else(|| Error::new_spanned(path, "Expected field name"))?;
            self.requires.insert(name);
        }
        Ok(())
    }
}

impl Parse for Mutator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut fun: ItemFn = input.parse()?;

        let mut attribute = MutatorAttribute::default();

        let mut i = 0;
        while i < fun.attrs.len() {
            let attr = &fun.attrs[i];
            if attr.path().is_ident("mutator") {
                attribute.apply_attr(attr)?;
                fun.attrs.remove(i);
            } else {
                i += 1;
            }
        }

        // Ensure `&mut self` receiver
        if let Some(FnArg::Receiver(receiver)) = fun.sig.inputs.first_mut() {
            *receiver = parse_quote!(&mut self);
        } else {
            // Error either on first argument or `()`
            return Err(syn::Error::new(
                fun.sig
                    .inputs
                    .first()
                    .map(Spanned::span)
                    .unwrap_or(fun.sig.paren_token.span.span()),
                "mutator needs to take a reference to `self`",
            ));
        };

        Ok(Self {
            fun,
            required_fields: attribute.requires,
        })
    }
}

impl Mutator {
    /// Signature for Builder::<mutator> function
    pub fn outer_sig(&self, output: Type) -> Signature {
        let mut sig = self.fun.sig.clone();
        sig.output = ReturnType::Type(Default::default(), output.into());

        sig.inputs = sig
            .inputs
            .into_iter()
            .enumerate()
            .map(|(i, input)| match input {
                FnArg::Receiver(_) => parse_quote!(self),
                FnArg::Typed(mut input) => {
                    input.pat = Box::new(
                        PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: None,
                            ident: pat_to_ident(i, &input.pat),
                            subpat: None,
                        }
                        .into(),
                    );
                    FnArg::Typed(input)
                }
            })
            .collect();
        sig
    }

    /// Arguments to call inner mutator function
    pub fn arguments(&self) -> Punctuated<Ident, Token![,]> {
        self.fun
            .sig
            .inputs
            .iter()
            .enumerate()
            .filter_map(|(i, input)| match &input {
                FnArg::Receiver(_) => None,
                FnArg::Typed(input) => Some(pat_to_ident(i, &input.pat)),
            })
            .collect()
    }
}
