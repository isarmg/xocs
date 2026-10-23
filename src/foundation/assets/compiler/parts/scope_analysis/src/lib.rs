//! Semantic analysis of a JavaScript/TypeScript program.
//!
//! # Example
//! ```ignore
#![doc = include_str!("../examples/semantic.rs")]
//! ```

use std::ops::RangeBounds;

use crate::syntax_tree::{
    AstKind, Comment, CommentsRange, ast::IdentifierReference, comments_range, get_comment_at,
    has_comments_between, is_inside_comment,
};
#[cfg(any())]
use oxc_cfg::ControlFlowGraph;
use crate::source_location::{GetSpan, SourceType, Span};
// Re-export flags and ID types
pub use crate::syntax_types::{
    node::{NodeFlags, NodeId},
    reference::{Reference, ReferenceFlags, ReferenceId},
    scope::{ScopeFlags, ScopeId},
    symbol::{SymbolFlags, SymbolId},
};

#[cfg(any())]
pub mod dot;

#[cfg(any())]
mod ast_types_bitset;
mod binder;
mod builder;
mod checker;
mod class;
mod diagnostics;
mod is_global_reference;
#[cfg(any())]
mod jsdoc;
mod label;
mod multi_index_vec;
mod node;
mod scoping;
mod stats;
pub mod ts_enum;
mod unresolved_stack;

#[cfg(any())]
pub use ast_types_bitset::AstTypesBitset;
pub use builder::{SemanticBuilder, SemanticBuilderReturn};
pub use is_global_reference::IsGlobalReference;
#[cfg(any())]
pub use jsdoc::JSDocFinder;
pub use node::{Ancestry, AncestryStack, AstNode, AstNodes};
#[cfg(any())]
pub use oxc_jsdoc::{JSDoc, JSDocTag};
pub use scoping::Scoping;
pub use stats::Stats;

use class::ClassTable;

/// Semantic analysis of a JavaScript/TypeScript program.
///
/// [`Semantic`] contains the results of analyzing a program, including the
/// [`Abstract Syntax Tree (AST)`] and [`scoping`].
///
/// Do not construct this struct directly; instead, use [`SemanticBuilder`].
///
/// [`Abstract Syntax Tree (AST)`]: crate::scope_analysis::AstNodes
/// [`scoping`]: crate::scope_analysis::Scoping
#[derive(Default)]
pub struct Semantic<'a> {
    /// Source code of the JavaScript/TypeScript program being analyzed.
    source_text: &'a str,

    /// What kind of source code is being analyzed. Comes from the parser.
    source_type: SourceType,

    /// The Abstract Syntax Tree (AST) nodes.
    nodes: AstNodes<'a>,

    /// Number of AST nodes in the program.
    ///
    /// Tracked separately from `nodes`, which is empty unless the builder ran
    /// with [`SemanticBuilder::with_build_nodes`] enabled.
    node_count: u32,

    scoping: Scoping,

    classes: ClassTable<'a>,

    /// Parsed comments.
    comments: &'a [Comment],
    irregular_whitespaces: Box<[Span]>,

    /// Parsed JSDoc comments.
    #[cfg(any())]
    jsdoc: JSDocFinder<'a>,

    unused_labels: Vec<NodeId>,

    /// Control flow graph. Only present if [`Semantic`] is built with cfg
    /// creation enabled using [`SemanticBuilder::with_cfg`].
    #[cfg(any())]
    cfg: Option<ControlFlowGraph>,
    #[cfg(not(any()))]
    #[expect(unused)]
    cfg: (),
}

impl<'a> Semantic<'a> {
    /// Extract [`Scoping`] from [`Semantic`].
    pub fn into_scoping(self) -> Scoping {
        self.scoping
    }

    /// Extract [`Scoping`] and [`AstNode`] from the [`Semantic`].
    pub fn into_scoping_and_nodes(self) -> (Scoping, AstNodes<'a>) {
        (self.scoping, self.nodes)
    }

    /// Discard all references to AST nodes and comments held by this `Semantic`.
    ///
    /// This should not ordinarily be used. It leaves [`Semantic`] in an inconsistent state.
    /// This method is only present to support some unsafe code in `oxc_linter`.
    pub fn clear_ast_references(&mut self) {
        self.nodes = AstNodes::default();
        self.comments = &[];
    }

    /// Source code of the JavaScript/TypeScript program being analyzed.
    pub fn source_text(&self) -> &'a str {
        self.source_text
    }

    /// What kind of source code is being analyzed. Comes from the parser.
    pub fn source_type(&self) -> &SourceType {
        &self.source_type
    }

    /// Nodes in the Abstract Syntax Tree (AST)
    pub fn nodes(&self) -> &AstNodes<'a> {
        &self.nodes
    }

    /// Scoping data collected for this program.
    pub fn scoping(&self) -> &Scoping {
        &self.scoping
    }

    /// Mutable access to scoping data.
    pub fn scoping_mut(&mut self) -> &mut Scoping {
        &mut self.scoping
    }

    /// Mutable access to scoping data together with read-only AST nodes.
    pub fn scoping_mut_and_nodes(&mut self) -> (&mut Scoping, &AstNodes<'a>) {
        (&mut self.scoping, &self.nodes)
    }

    /// Class metadata collected during semantic analysis.
    pub fn classes(&self) -> &ClassTable<'_> {
        &self.classes
    }

    /// Set recorded spans for irregular unicode whitespace in source text.
    pub fn set_irregular_whitespaces(&mut self, irregular_whitespaces: Box<[Span]>) {
        self.irregular_whitespaces = irregular_whitespaces;
    }

    /// Trivias (comments) found while parsing
    pub fn comments(&self) -> &[Comment] {
        self.comments
    }

    /// Iterate comments within a byte range.
    pub fn comments_range<R>(&self, range: R) -> CommentsRange<'_>
    where
        R: RangeBounds<u32>,
    {
        comments_range(self.comments, range)
    }

    /// Returns `true` if any comment lies between `span.start` and `span.end`.
    pub fn has_comments_between(&self, span: Span) -> bool {
        has_comments_between(self.comments, span)
    }

    /// Returns `true` if `pos` is inside a parsed comment.
    pub fn is_inside_comment(&self, pos: u32) -> bool {
        is_inside_comment(self.comments, pos)
    }

    /// Get the comment containing a position, if any.
    pub fn get_comment_at(&self, pos: u32) -> Option<&Comment> {
        get_comment_at(self.comments, pos)
    }

    /// Spans of irregular whitespace discovered by the parser.
    pub fn irregular_whitespaces(&self) -> &[Span] {
        &self.irregular_whitespaces
    }

    /// Parsed [`JSDoc`] comments.
    ///
    /// Will be empty if JSDoc parsing is disabled.
    #[cfg(any())]
    pub fn jsdoc(&self) -> &JSDocFinder<'a> {
        &self.jsdoc
    }

    /// Labels that were declared but never used.
    pub fn unused_labels(&self) -> &Vec<NodeId> {
        &self.unused_labels
    }

    /// Control flow graph.
    ///
    /// Only present if [`Semantic`] is built with cfg creation enabled using
    /// [`SemanticBuilder::with_cfg`].
    #[cfg(any())]
    pub fn cfg(&self) -> Option<&ControlFlowGraph> {
        self.cfg.as_ref()
    }

    /// Control flow graph.
    ///
    /// Always returns `None` when the `cfg` feature is disabled.
    #[cfg(not(any()))]
    #[expect(clippy::unused_self)]
    pub fn cfg(&self) -> Option<&()> {
        None
    }

    /// Get statistics about data held in `Semantic`.
    pub fn stats(&self) -> Stats {
        #[expect(clippy::cast_possible_truncation)]
        Stats::new(
            self.node_count,
            self.scoping.scopes_len() as u32,
            self.scoping.symbols_len() as u32,
            self.scoping.references.len() as u32,
        )
    }

    /// Returns `true` if `node_id` points to an unresolved identifier reference.
    pub fn is_unresolved_reference(&self, node_id: NodeId) -> bool {
        let reference_node = self.nodes.get_node(node_id);
        let AstKind::IdentifierReference(id) = reference_node.kind() else {
            return false;
        };
        id.is_global_reference(&self.scoping)
    }

    /// Find which scope a symbol is declared in
    pub fn symbol_scope(&self, symbol_id: SymbolId) -> ScopeId {
        self.scoping.symbol_scope_id(symbol_id)
    }

    /// Get all resolved references for a symbol
    pub fn symbol_references(
        &self,
        symbol_id: SymbolId,
    ) -> impl Iterator<Item = &Reference> + '_ + use<'_> {
        self.scoping.get_resolved_references(symbol_id)
    }

    /// Get the AST node that declares `symbol_id`.
    pub fn symbol_declaration(&self, symbol_id: SymbolId) -> &AstNode<'a> {
        self.nodes.get_node(self.scoping.symbol_declaration(symbol_id))
    }

    /// Returns `true` if `ident` resolves to a global (unbound) reference.
    pub fn is_reference_to_global_variable(&self, ident: &IdentifierReference) -> bool {
        ident.is_global_reference(&self.scoping)
    }

    /// Get the textual name for a semantic reference.
    pub fn reference_name(&self, reference: &Reference) -> &str {
        let node = self.nodes.get_node(reference.node_id());
        match node.kind() {
            AstKind::IdentifierReference(id) => id.name.as_str(),
            _ => unreachable!(),
        }
    }

    /// Get the source span for a semantic reference.
    pub fn reference_span(&self, reference: &Reference) -> Span {
        let node = self.nodes.get_node(reference.node_id());
        node.kind().span()
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/scope_analysis/src/lib/tests.rs"]
#[cfg(test)]
mod tests;
