#[cfg(any())]
mod compiler;

#[cfg(any())]
pub use compiler::{Compiler, CompilerInterface};

pub mod allocator {
    //! Memory arena allocator used by all other submodules.
    //!
    //! See the [`oxc_allocator` module-level documentation](oxc_allocator) for more information.
    #[doc(inline)]
    pub use crate::syntax_arena::*;
}

pub mod ast {
    #[doc(inline)]
    pub use crate::syntax_tree::*;
}

#[cfg(all())]
pub mod ast_visit {
    #[doc(inline)]
    pub use crate::syntax_visitor::*;
}

pub mod diagnostics {
    //! Error data types and utilities for handling/reporting them.
    //!
    //! See the [`oxc_diagnostics` module-level documentation](oxc_diagnostics) for more information.
    #[doc(inline)]
    pub use crate::compiler_errors::*;
}

pub mod parser {
    //! JavaScript/TypeScript parser.
    //!
    //! See the [`oxc_parser` module-level documentation](oxc_parser) for more information.
    #[doc(inline)]
    pub use crate::parser::*;
}

#[cfg(all())]
pub mod regular_expression {
    #[doc(inline)]
    pub use crate::regular_expression::*;
}

pub mod span {
    //! Source text Span.
    //!
    //! See the [`oxc_span` module-level documentation](oxc_span) for more information.
    #[doc(inline)]
    pub use crate::source_location::*;
}

pub mod str {
    //! String types.
    //!
    //! See the [`oxc_str` module-level documentation](oxc_str) for more information.
    #[doc(inline)]
    pub use crate::compiler_strings::*;
}

pub mod syntax {
    //! Common code for JavaScript Syntax
    //!
    //! See the [`oxc_syntax` module-level documentation](oxc_syntax) for more information.
    #[doc(inline)]
    pub use crate::syntax_types::*;
}

#[cfg(all())]
pub mod semantic {
    //! Semantic analysis of a JavaScript/TypeScript program.
    //!
    //! See the [`oxc_semantic` module-level documentation](oxc_semantic) for more information.
    #[doc(inline)]
    pub use crate::scope_analysis::*;
}

#[cfg(any())]
pub mod transformer {
    //! Transformer/Transpiler
    //!
    //! See the [`oxc_transformer` module-level documentation](oxc_transformer) for more
    //! information.
    #[doc(inline)]
    pub use oxc_transformer::*;
}

#[cfg(any())]
pub mod transformer_plugins {
    //! Transformer/Transpiler
    //!
    //! See the [`oxc_transformer_plugins` module-level documentation](oxc_transformer_plugins) for more
    //! information.
    #[doc(inline)]
    pub use oxc_transformer_plugins::*;
}

#[cfg(all())]
pub mod minifier {
    //! Source code minifier.
    //!
    //! See the [`oxc_minifier` module-level documentation](oxc_minifier) for more information.
    #[doc(inline)]
    pub use crate::optimizer::*;
}

#[cfg(any())]
pub mod mangler {
    #[doc(inline)]
    pub use crate::name_compaction::*;
}

#[cfg(all())]
pub mod codegen {
    //! AST code printer
    //!
    //! See the [`oxc_codegen` module-level documentation](oxc_codegen) for more information.
    #[doc(inline)]
    pub use crate::code_generator::*;
}

#[cfg(any())]
pub mod isolated_declarations {
    //! `.d.ts` emit for Isolated Declarations.
    //!
    //! See the [`oxc_isolated_declarations` module-level documentation](oxc_isolated_declarations)
    //! for more information.
    #[doc(inline)]
    pub use oxc_isolated_declarations::*;
}

#[cfg(any())]
pub mod cfg {
    #[doc(inline)]
    pub use oxc_cfg::*;
}
