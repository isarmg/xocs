use num_traits::ToPrimitive;

use crate::language_semantics::to_integer_or_infinity::ToIntegerOrInfinityResult;

pub trait StringCharAt {
    /// `String.prototype.charAt ( pos )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.charat>
    fn char_at(&self, pos: Option<f64>) -> StringCharAtResult;
}

impl StringCharAt for &str {
    fn char_at(&self, pos: Option<f64>) -> StringCharAtResult {
        use crate::language_semantics::to_integer_or_infinity::ToIntegerOrInfinity;

        let position = pos.unwrap_or(0.0).to_integer_or_infinity_as_i64();
        let position = match position {
            ToIntegerOrInfinityResult::Value(v)
                if v >= 0
                    && let Some(v) = v.to_usize() =>
            {
                v
            }
            _ => return StringCharAtResult::OutOfRange,
        };

        self.encode_utf16().nth(position).map_or(StringCharAtResult::OutOfRange, |n| {
            char::from_u32(u32::from(n))
                .map_or(StringCharAtResult::InvalidChar(n), StringCharAtResult::Value)
        })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StringCharAtResult {
    Value(char),
    InvalidChar(u16),
    OutOfRange,
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/language_semantics/src/string_char_at/test.rs"]
#[cfg(test)]
mod test;
