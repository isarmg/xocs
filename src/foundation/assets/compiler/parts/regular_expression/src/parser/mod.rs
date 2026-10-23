mod flags_parser;
mod parser_impl;
mod pattern_parser;
mod reader;
mod span_factory;

pub use parser_impl::{ConstructorParser, LiteralParser};

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/regular_expression/src/parser/mod/test.rs"]
#[cfg(test)]
mod test;
