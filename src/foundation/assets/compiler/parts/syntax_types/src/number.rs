use crate::syntax_arena::{CloneIn, Dummy};
use oxc_ast_macros::ast;
use crate::source_location::ContentEq;

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, Dummy, ContentEq)]
pub enum NumberBase {
    Float = 0,
    Decimal = 1,
    Binary = 2,
    Octal = 3,
    Hex = 4,
}

impl NumberBase {
    pub fn is_base_10(self) -> bool {
        matches!(self, Self::Float | Self::Decimal)
    }
}

#[ast]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[generate_derive(CloneIn, Dummy, ContentEq)]
pub enum BigintBase {
    Decimal = 0,
    Binary = 1,
    Octal = 2,
    Hex = 3,
}

impl BigintBase {
    pub fn is_base_10(self) -> bool {
        self == Self::Decimal
    }
}

/// <https://tc39.es/ecma262/#sec-numeric-types-number-tostring>
#[cfg(all())]
pub trait ToJsString {
    fn to_js_string(&self) -> String;
}

#[cfg(all())]
impl ToJsString for f64 {
    fn to_js_string(&self) -> String {
        let mut buffer = dragonbox_ecma::Buffer::new();
        buffer.format(*self).to_string()
    }
}
