use crate::syntax_arena::{Allocator, CloneIn, CloneInSemanticIds};

use crate::regular_expression::ast::Modifier;

impl<'alloc> CloneIn<'alloc> for Modifier {
    type Cloned = Self;

    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        _: &'alloc Allocator,
    ) -> Self::Cloned {
        *self
    }
}
