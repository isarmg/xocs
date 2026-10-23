//! Gather node parts trait.
//!
//! Ported from: <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts>
//!
//! This trait is used to gather all the parts of a node that are identifiers.

use crate::syntax_tree::ast::*;
use crate::compiler_collections::string_ext::StringExt;
use crate::language_semantics::BoundNames;

use super::to_identifier;

const MAX_LEN: usize = 20;

/// Get a variable name for an AST node, by concatenating identifiers used in the node.
///
/// The returned name is maximum 20 bytes.
///
/// This is based on Babel, but diverges slightly. Babel limits to 20 UTF-16 chars, not 20 UTF-8 bytes.
/// But the goal is just to avoid overly long variable names, the exact length is not really important.
pub fn get_var_name_from_node<'a, N: GatherNodeParts<'a>>(node: &N) -> String {
    let mut name = String::with_capacity(MAX_LEN);

    // Visit the node, gathering identifiers.
    //
    // Append them to `name`, capping the length at max `MAX_LEN` bytes.
    // Once `name` reaches `MAX_LEN` bytes, all further calls to the closure are no-ops.
    //
    // Length is strictly prevented from exceeding `MAX_LEN` bytes here, so `name` `String` never grows
    // beyond initial `MAX_LEN` capacity. Unsafe code in this closure relies on this invariant.
    // Be very careful to always uphold this invariant when making any changes to this code.
    node.gather(&mut |mut part| {
        // If `name` is already 20 bytes, ignore this part
        if name.len() == MAX_LEN {
            return;
        }

        debug_assert!(name.len() < MAX_LEN);

        if name.is_empty() {
            part = part.trim_start_matches('_');
        } else {
            // SAFETY: `name` is initialized with `MAX_LEN` capacity and never grows or shrinks,
            // so it has spare capacity for 1 more byte - we exited above if `name.len() == MAX_LEN`.
            unsafe { name.push_unchecked('$') };
        }

        // This addition cannot overflow `usize` because slices' length cannot exceed `isize::MAX`
        let new_len = name.len() + part.len();
        if new_len > MAX_LEN {
            // Truncate `part` so adding it to `name` results in `name` being `MAX_LEN` bytes.
            // If the cut point which would give `MAX_LEN` bytes is on a UTF-8 char boundary,
            // move the cut point forwards to the start of the UTF-8 char.
            debug_assert!(name.len() <= MAX_LEN);

            let mut cut_at = MAX_LEN - name.len();
            debug_assert!(cut_at < part.len());

            let bytes = part.as_bytes();

            // SAFETY:
            // * `name.len() + part.len() > MAX_LEN` (`new_len > MAX_LEN` check above)
            // * `name.len() <= MAX_LEN` (`name.len() < MAX_LEN` at top of closure, and we pushed at most 1 `$`)
            // * Therefore: `part.len() > MAX_LEN - name.len()`
            // * `cut_at = MAX_LEN - name.len()`
            // * Therefore: `part.len() > cut_at`
            // * Reversed: `cut_at < part.len()`
            // * `cut_at` is in bounds of `bytes`
            let cut_byte = unsafe { *bytes.get_unchecked(cut_at) };
            if !cut_byte.is_ascii() {
                // `cut_at` may be in the middle of a multi-byte char. Walk back to find the start byte.
                // Unicode is rare, so mark this as cold path.
                #[cold]
                #[inline]
                fn adjust_cut_at(bytes: &[u8], cut_at: &mut usize) {
                    // SAFETY: `bytes` comes from a valid UTF-8 `&str`.
                    // UTF-8 chars are maximum 4 bytes.
                    // Non-ASCII chars' byte sequences start with a byte which is `>= 0xC0`.
                    // We checked above that the byte at `cut_at` is not ASCII, so either:
                    // 1. Byte at `cut_at` is start of a multi-byte char, or
                    // 2. `cut` at is in middle of a multi-byte character, with 1st byte of character
                    //    at one of `cut_at - 1`, `cut_at - 2`, `cut_at - 3`.
                    unsafe {
                        if *bytes.get_unchecked(*cut_at) >= 0xC0 {
                            // `cut_at` already points to start of a multi-byte char
                        } else if *bytes.get_unchecked(*cut_at - 1) >= 0xC0 {
                            *cut_at -= 1;
                        } else if *bytes.get_unchecked(*cut_at - 2) >= 0xC0 {
                            *cut_at -= 2;
                        } else {
                            *cut_at -= 3;
                        }
                    }
                }
                adjust_cut_at(bytes, &mut cut_at);
            }

            // SAFETY: `cut_at` is `< part.len()` and on a UTF-8 character boundary
            unsafe {
                part = part.get_unchecked(..cut_at);
            }
        }

        // SAFETY: `name` is initialized with `MAX_LEN` capacity and never grows or shrinks.
        // `part` has been shortened above if required to fit within `name`'s remaining capacity.
        unsafe { name.push_str_unchecked(part) };
    });

    if name.is_empty() {
        const _: () = assert!(MAX_LEN >= 3);

        // SAFETY: `name` is initialized with `MAX_LEN` capacity and never grows or shrinks.
        // `name` is empty, so has capacity for 3 bytes.
        unsafe { name.push_str_unchecked("ref") };
    } else {
        name = to_identifier(name);
    }

    name
}

pub trait GatherNodeParts<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F);
}

// -------------------- ModuleDeclaration --------------------
impl<'a> GatherNodeParts<'a> for ImportDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.source.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportAllDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.source.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.declaration.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportNamedDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        for specifier in &self.specifiers {
            specifier.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for ExportFromDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.source.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportDefaultDeclaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.declaration.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ExportDefaultDeclarationKind<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ExportDefaultDeclarationKind::FunctionDeclaration(decl) => decl.gather(f),
            ExportDefaultDeclarationKind::ClassDeclaration(decl) => decl.gather(f),
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {}
            match_expression!(ExportDefaultDeclarationKind) => self.to_expression().gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ExportSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match &self.local {
            ModuleExportName::IdentifierName(ident) => ident.gather(f),
            ModuleExportName::IdentifierReference(ident) => ident.gather(f),
            ModuleExportName::StringLiteral(lit) => lit.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ModuleExportName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ModuleExportName::IdentifierName(ident) => ident.gather(f),
            ModuleExportName::IdentifierReference(ident) => ident.gather(f),
            ModuleExportName::StringLiteral(lit) => lit.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ImportSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ImportDefaultSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ImportNamespaceSpecifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.local.gather(f);
    }
}

// -------------------- Declaration --------------------

impl<'a> GatherNodeParts<'a> for Declaration<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            Self::FunctionDeclaration(decl) => decl.gather(f),
            Self::ClassDeclaration(decl) => decl.gather(f),
            _ => (),
        }
    }
}

// -------------------- Function --------------------

impl<'a> GatherNodeParts<'a> for Function<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if let Some(id) = &self.id {
            id.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for BindingRestElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

// -------------------- BindingPattern --------------------

impl<'a> GatherNodeParts<'a> for VariableDeclarator<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.id.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for BindingPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for ObjectPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for ArrayPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

impl<'a> GatherNodeParts<'a> for AssignmentPattern<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.bound_names(&mut |id| f(id.name.as_str()));
    }
}

// -------------------- Expression --------------------

impl<'a> GatherNodeParts<'a> for Expression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            match_member_expression!(Self) => self.to_member_expression().gather(f),
            Self::Identifier(ident) => ident.gather(f),
            Self::CallExpression(expr) => expr.gather(f),
            Self::NewExpression(expr) => expr.gather(f),
            Self::ObjectExpression(expr) => expr.gather(f),
            Self::ThisExpression(expr) => expr.gather(f),
            Self::Super(expr) => expr.gather(f),
            Self::ImportExpression(expr) => expr.gather(f),
            Self::YieldExpression(expr) => expr.gather(f),
            Self::AwaitExpression(expr) => expr.gather(f),
            Self::AssignmentExpression(expr) => expr.gather(f),
            Self::FunctionExpression(expr) => expr.gather(f),
            Self::ClassExpression(expr) => expr.gather(f),
            Self::ParenthesizedExpression(expr) => expr.gather(f),
            Self::UnaryExpression(expr) => expr.gather(f),
            Self::UpdateExpression(expr) => expr.gather(f),
            Self::ChainExpression(expr) => expr.gather(f),
            Self::ImportMeta(expr) => expr.gather(f),
            Self::NewTarget(expr) => expr.gather(f),
            Self::JSXElement(expr) => expr.gather(f),
            Self::JSXFragment(expr) => expr.gather(f),
            Self::StringLiteral(expr) => expr.gather(f),
            Self::NumericLiteral(expr) => expr.gather(f),
            Self::BooleanLiteral(expr) => expr.gather(f),
            Self::BigIntLiteral(expr) => expr.gather(f),
            _ => (),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ChainExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.expression.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ChainElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ChainElement::CallExpression(expr) => expr.gather(f),
            ChainElement::TSNonNullExpression(expr) => expr.expression.gather(f),
            expr @ match_member_expression!(Self) => expr.to_member_expression().gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for MemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => {
                expr.gather(f);
            }
            MemberExpression::StaticMemberExpression(expr) => {
                expr.gather(f);
            }
            MemberExpression::PrivateFieldExpression(expr) => {
                expr.gather(f);
            }
        }
    }
}

impl<'a> GatherNodeParts<'a> for ComputedMemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.expression.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for StaticMemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.property.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for PrivateFieldExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.field.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for CallExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.callee.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for NewExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.callee.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ObjectExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        for prop in &self.properties {
            prop.gather(f);
        }
    }
}

impl GatherNodeParts<'_> for ThisExpression {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("this");
    }
}

impl GatherNodeParts<'_> for Super {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("super");
    }
}

impl<'a> GatherNodeParts<'a> for ImportExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("import");
    }
}

impl<'a> GatherNodeParts<'a> for YieldExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("yield");
        if let Some(argument) = &self.argument {
            argument.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for AwaitExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("await");
        self.argument.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for AssignmentExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.left.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for ParenthesizedExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.expression.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for UnaryExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for UpdateExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

impl GatherNodeParts<'_> for ImportMeta {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("import");
        f("meta");
    }
}

impl GatherNodeParts<'_> for NewTarget {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("new");
        f("target");
    }
}

// -------------------- AssignmentTarget --------------------
impl<'a> GatherNodeParts<'a> for AssignmentTarget<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            match_simple_assignment_target!(Self) => {
                self.to_simple_assignment_target().gather(f);
            }
            match_assignment_target_pattern!(Self) => {}
        }
    }
}

impl<'a> GatherNodeParts<'a> for SimpleAssignmentTarget<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gather(f),
            match_member_expression!(Self) => self.to_member_expression().gather(f),
            _ => {}
        }
    }
}

// -------------------- Classes --------------------

impl<'a> GatherNodeParts<'a> for Class<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if let Some(id) = &self.id {
            id.gather(f);
        }
    }
}

impl<'a> GatherNodeParts<'a> for ClassElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ClassElement::PropertyDefinition(def) => def.gather(f),
            ClassElement::MethodDefinition(def) => def.gather(f),
            ClassElement::AccessorProperty(def) => def.gather(f),
            _ => (),
        }
    }
}

impl<'a> GatherNodeParts<'a> for PropertyDefinition<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for MethodDefinition<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for AccessorProperty<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

// -------------------- Objects --------------------

impl<'a> GatherNodeParts<'a> for ObjectPropertyKind<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            ObjectPropertyKind::ObjectProperty(prop) => prop.gather(f),
            ObjectPropertyKind::SpreadProperty(prop) => prop.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for ObjectProperty<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.key.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for PropertyKey<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            PropertyKey::StaticIdentifier(ident) => ident.gather(f),
            PropertyKey::PrivateIdentifier(ident) => ident.gather(f),
            match_expression!(Self) => self.to_expression().gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for SpreadElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.argument.gather(f);
    }
}

// -------------------- Identifiers --------------------

impl<'a> GatherNodeParts<'a> for BindingIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for IdentifierReference<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for IdentifierName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for PrivateIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

// -------------------- Literals --------------------

impl<'a> GatherNodeParts<'a> for StringLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.value.as_str());
    }
}

impl<'a> GatherNodeParts<'a> for NumericLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(&self.raw_str());
    }
}

impl GatherNodeParts<'_> for BooleanLiteral {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        if self.value {
            f("true");
        } else {
            f("false");
        }
    }
}

impl<'a> GatherNodeParts<'a> for BigIntLiteral<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.value.as_str());
    }
}

// -------------------- JSX --------------------

impl<'a> GatherNodeParts<'a> for JSXElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.opening_element.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXFragment<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.opening_fragment.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXOpeningElement<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.name.gather(f);
    }
}

impl GatherNodeParts<'_> for JSXOpeningFragment {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f("Fragment");
    }
}

impl<'a> GatherNodeParts<'a> for JSXElementName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            JSXElementName::Identifier(ident) => ident.gather(f),
            JSXElementName::IdentifierReference(ident) => ident.gather(f),
            JSXElementName::NamespacedName(ns) => ns.gather(f),
            JSXElementName::MemberExpression(expr) => expr.gather(f),
            JSXElementName::ThisExpression(expr) => expr.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for JSXNamespacedName<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.namespace.gather(f);
        self.name.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXMemberExpression<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        self.object.gather(f);
        self.property.gather(f);
    }
}

impl<'a> GatherNodeParts<'a> for JSXMemberExpressionObject<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        match self {
            JSXMemberExpressionObject::IdentifierReference(ident) => ident.gather(f),
            JSXMemberExpressionObject::MemberExpression(expr) => expr.gather(f),
            JSXMemberExpressionObject::ThisExpression(expr) => expr.gather(f),
        }
    }
}

impl<'a> GatherNodeParts<'a> for JSXIdentifier<'a> {
    fn gather<F: FnMut(&str)>(&self, f: &mut F) {
        f(self.name.as_str());
    }
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/tree_traversal/src/ast_operations/gather_node_parts/tests.rs"]
#[cfg(test)]
mod tests;
