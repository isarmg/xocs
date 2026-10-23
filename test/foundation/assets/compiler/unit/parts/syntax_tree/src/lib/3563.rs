#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    use crate::syntax_tree::ast;

    assert_eq!(size_of::<ast::Statement>(), 16);
    assert_eq!(size_of::<ast::Expression>(), 16);
    assert_eq!(size_of::<ast::Declaration>(), 16);
    assert_eq!(size_of::<ast::BindingPattern>(), 16);
    assert_eq!(size_of::<ast::ModuleDeclaration>(), 16);
    assert_eq!(size_of::<ast::ClassElement>(), 16);
    assert_eq!(size_of::<ast::ExportDefaultDeclarationKind>(), 16);
    assert_eq!(size_of::<ast::AssignmentTargetPattern>(), 16);
    assert_eq!(size_of::<ast::AssignmentTargetMaybeDefault>(), 16);
    assert_eq!(size_of::<ast::AssignmentTargetProperty>(), 16);
    assert_eq!(size_of::<ast::TSLiteral>(), 16);
    assert_eq!(size_of::<ast::TSType>(), 16);
}
