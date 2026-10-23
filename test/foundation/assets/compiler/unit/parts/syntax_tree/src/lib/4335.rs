#[test]
fn lifetime_variance() {
    use crate::syntax_tree::ast;

    fn _assert_program_variant_lifetime<'a: 'b, 'b>(program: ast::Program<'a>) -> ast::Program<'b> {
        program
    }
}
