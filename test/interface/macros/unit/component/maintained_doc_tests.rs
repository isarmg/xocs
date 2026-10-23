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
