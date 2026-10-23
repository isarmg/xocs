use crate::language_semantics::{StringCharAt, string_char_at::StringCharAtResult};

pub trait StringCharCodeAt {
    /// `String.prototype.charCodeAt ( pos )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.charcodeat>
    fn char_code_at(&self, index: Option<f64>) -> Option<u32>;
}

impl StringCharCodeAt for &str {
    fn char_code_at(&self, index: Option<f64>) -> Option<u32> {
        match self.char_at(index) {
            StringCharAtResult::Value(c) => Some(c as u32),
            StringCharAtResult::InvalidChar(v) => Some(u32::from(v)),
            StringCharAtResult::OutOfRange => None,
        }
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/language_semantics/src/string_char_code_at/test.rs"]
#[cfg(test)]
mod test;
