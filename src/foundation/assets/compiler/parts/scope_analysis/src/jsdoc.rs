use rustc_hash::FxHashMap;

use oxc_jsdoc::JSDoc;
use crate::source_location::{GetSpan, Span};

use crate::scope_analysis::{AstNode, AstNodes};

#[derive(Debug, Default)]
pub struct JSDocFinder<'a> {
    /// JSDocs by Span
    attached: FxHashMap<u32, Vec<JSDoc<'a>>>,
    not_attached: Vec<JSDoc<'a>>,
}

impl<'a> JSDocFinder<'a> {
    pub fn new(attached: FxHashMap<u32, Vec<JSDoc<'a>>>, not_attached: Vec<JSDoc<'a>>) -> Self {
        Self { attached, not_attached }
    }

    pub fn get_one_by_node<'b>(
        &'b self,
        nodes: &AstNodes<'a>,
        node: &AstNode<'a>,
    ) -> Option<&'b JSDoc<'a>> {
        let jsdocs = self.get_all_by_node(nodes, node)?;

        // If flagged, at least 1 JSDoc is attached
        // If multiple JSDocs are attached, return the last = nearest
        jsdocs.last()
    }

    pub fn get_all_by_node<'b>(
        &'b self,
        nodes: &AstNodes<'a>,
        node: &AstNode<'a>,
    ) -> Option<&'b [JSDoc<'a>]> {
        if !nodes.flags(node.id()).has_jsdoc() {
            return None;
        }

        let span = node.kind().span();
        self.get_all_by_span(span)
    }

    pub fn get_all_by_span<'b>(&'b self, span: Span) -> Option<&'b [JSDoc<'a>]> {
        self.attached.get(&span.start).map(Vec::as_slice)
    }

    pub fn iter_all<'b>(&'b self) -> impl Iterator<Item = &'b JSDoc<'a>> + 'b {
        self.attached.values().flatten().chain(self.not_attached.iter())
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/scope_analysis/src/jsdoc/test.rs"]
#[cfg(test)]
mod test;
