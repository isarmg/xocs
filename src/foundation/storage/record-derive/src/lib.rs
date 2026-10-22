//! SQLite record mapping derive; no database connections, environment probing,
//! runtime executors or compile-time query evaluation are linked into this macro.
//! Row expansion and attribute semantics originate in SQLx 0.9.0. Original
//! licenses and source archives remain in compliance; see the integration map.
use proc_macro::TokenStream;

mod attributes;
mod row;

use attributes::RenameAll;
use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};

#[proc_macro_derive(FromRow, attributes(sqlx))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    row::expand_derive_from_row(&input)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

fn rename_all(value: &str, pattern: RenameAll) -> String {
    match pattern {
        RenameAll::Lower => value.to_lowercase(),
        RenameAll::Snake => value.to_snake_case(),
        RenameAll::Upper => value.to_uppercase(),
        RenameAll::ScreamingSnake => value.to_shouty_snake_case(),
        RenameAll::Kebab => value.to_kebab_case(),
        RenameAll::Camel => value.to_lower_camel_case(),
        RenameAll::Pascal => value.to_upper_camel_case(),
    }
}
