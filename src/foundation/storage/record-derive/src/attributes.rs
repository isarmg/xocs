//! FromRow attribute parser, narrowed from SQLx 0.9.0 derives/attributes.rs.
use syn::{Attribute, Ident, LitStr, Type, parenthesized};

#[derive(Clone, Copy)]
pub(crate) enum RenameAll {
    Lower,
    Snake,
    Upper,
    ScreamingSnake,
    Kebab,
    Camel,
    Pascal,
}

pub(crate) struct ContainerAttributes {
    pub rename_all: Option<RenameAll>,
    pub default: bool,
}

pub(crate) enum JsonAttribute {
    NonNullable,
    Nullable,
}

pub(crate) struct ChildAttributes {
    pub rename: Option<String>,
    pub default: bool,
    pub flatten: bool,
    pub try_from: Option<Type>,
    pub skip: bool,
    pub json: Option<JsonAttribute>,
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    meta: &syn::meta::ParseNestedMeta<'_>,
) -> syn::Result<()> {
    if slot.is_some() {
        return Err(meta.error("duplicate attribute"));
    }
    *slot = Some(value);
    Ok(())
}

pub(crate) fn parse_container_attributes(input: &[Attribute]) -> syn::Result<ContainerAttributes> {
    let mut rename_all = None;
    let mut default = None;
    for attribute in input
        .iter()
        .filter(|attribute| attribute.path().is_ident("sqlx"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                set_once(&mut default, true, &meta)
            } else if meta.path.is_ident("rename_all") {
                let literal: LitStr = meta.value()?.parse()?;
                let pattern = match literal.value().as_str() {
                    "lowercase" => RenameAll::Lower,
                    "snake_case" => RenameAll::Snake,
                    "UPPERCASE" => RenameAll::Upper,
                    "SCREAMING_SNAKE_CASE" => RenameAll::ScreamingSnake,
                    "kebab-case" => RenameAll::Kebab,
                    "camelCase" => RenameAll::Camel,
                    "PascalCase" => RenameAll::Pascal,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            literal,
                            "unexpected value for rename_all",
                        ));
                    }
                };
                set_once(&mut rename_all, pattern, &meta)
            } else {
                Err(meta.error("unsupported record container attribute"))
            }
        })?;
    }
    Ok(ContainerAttributes {
        rename_all,
        default: default.unwrap_or(false),
    })
}

pub(crate) fn parse_child_attributes(input: &[Attribute]) -> syn::Result<ChildAttributes> {
    let mut rename = None;
    let mut default = None;
    let mut flatten = None;
    let mut try_from = None;
    let mut skip = None;
    let mut json = None;
    for attribute in input
        .iter()
        .filter(|attribute| attribute.path().is_ident("sqlx"))
    {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let literal: LitStr = meta.value()?.parse()?;
                set_once(&mut rename, literal.value(), &meta)
            } else if meta.path.is_ident("try_from") {
                let literal: LitStr = meta.value()?.parse()?;
                set_once(&mut try_from, literal.parse()?, &meta)
            } else if meta.path.is_ident("default") {
                set_once(&mut default, true, &meta)
            } else if meta.path.is_ident("flatten") {
                set_once(&mut flatten, true, &meta)
            } else if meta.path.is_ident("skip") {
                set_once(&mut skip, true, &meta)
            } else if meta.path.is_ident("json") {
                let value = if meta.input.peek(syn::token::Paren) {
                    let content;
                    parenthesized!(content in meta.input);
                    let name: Ident = content.parse()?;
                    if name != "nullable" || !content.is_empty() {
                        return Err(syn::Error::new_spanned(
                            name,
                            "expected json or json(nullable)",
                        ));
                    }
                    JsonAttribute::Nullable
                } else {
                    JsonAttribute::NonNullable
                };
                set_once(&mut json, value, &meta)
            } else {
                Err(meta.error("unsupported record field attribute"))
            }
        })?;
        if json.is_some() && flatten == Some(true) {
            return Err(syn::Error::new_spanned(
                attribute,
                "cannot combine json and flatten",
            ));
        }
        if matches!(json, Some(JsonAttribute::Nullable)) && try_from.is_some() {
            return Err(syn::Error::new_spanned(
                attribute,
                "cannot combine try_from and json(nullable)",
            ));
        }
    }
    Ok(ChildAttributes {
        rename,
        try_from,
        json,
        default: default.unwrap_or(false),
        flatten: flatten.unwrap_or(false),
        skip: skip.unwrap_or(false),
    })
}

#[path = "../../../../../test/foundation/storage/record-derive/unit/src/attributes/tests.rs"]
#[cfg(test)]
mod tests;
