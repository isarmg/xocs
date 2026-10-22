// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/main.rs`.

use proc_macro2::TokenStream;
use quote::quote;

pub fn get_trait_crate_and_generics(trait_name: &str) -> Option<(TokenStream, TokenStream)> {
    let res = match trait_name {
        "CloneIn" => (quote!(::blog_assets_compiler::syntax_arena::CloneIn), quote!(< 'static >)),
        "Dummy" => (quote!(::blog_assets_compiler::syntax_arena::Dummy), quote!(< 'static >)),
        "TakeIn" => (quote!(::blog_assets_compiler::syntax_arena::TakeIn), quote!(< 'static >)),
        "ReplaceWith" => (quote!(::blog_assets_compiler::syntax_arena::ReplaceWith), quote!(< 'static >)),
        "GetAddress" => (quote!(::blog_assets_compiler::syntax_arena::GetAddress), TokenStream::new()),
        "UnstableAddress" => (quote!(::blog_assets_compiler::syntax_arena::UnstableAddress), TokenStream::new()),
        "GetNodeId" => (quote!(::blog_assets_compiler::syntax_types::GetNodeId), TokenStream::new()),
        "GetSpan" => (quote!(::blog_assets_compiler::source_location::GetSpan), TokenStream::new()),
        "GetSpanMut" => (quote!(::blog_assets_compiler::source_location::GetSpanMut), TokenStream::new()),
        "ContentEq" => (quote!(::blog_assets_compiler::source_location::ContentEq), TokenStream::new()),
        "ESTree" => (quote!(::blog_assets_compiler::syntax_serialization::ESTree), TokenStream::new()),
        _ => return None,
    };
    Some(res)
}
