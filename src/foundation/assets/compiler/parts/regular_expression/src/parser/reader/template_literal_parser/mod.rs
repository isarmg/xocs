pub mod ast;
mod diagnostics;
mod parser_impl;

pub use parser_impl::Parser;

#[path = "../../../../../../../../../../test/foundation/assets/compiler/unit/parts/regular_expression/src/parser/reader/template_literal_parser/mod/test.rs"]
#[cfg(test)]
mod test;
