pub mod ast;
mod diagnostics;
mod parser_impl;

pub use parser_impl::{Parser, parse_regexp_literal};

#[path = "../../../../../../../../../../test/foundation/assets/compiler/unit/parts/regular_expression/src/parser/reader/string_literal_parser/mod/test.rs"]
#[cfg(test)]
mod test;
