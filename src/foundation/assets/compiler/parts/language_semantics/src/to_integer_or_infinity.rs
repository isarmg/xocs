use num_traits::ToPrimitive;

pub trait ToIntegerOrInfinity {
    /// `ToIntegerOrInfinity`
    /// <https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tointegerorinfinity>
    fn to_integer_or_infinity(&self) -> f64;

    /// Convert the value to i64. If the value is bigger or smaller than i64::MIN or i64::MAX,
    /// it will be converted to Infinity or NegativeInfinity.
    fn to_integer_or_infinity_as_i64(&self) -> ToIntegerOrInfinityResult {
        let res = self.to_integer_or_infinity();
        match res {
            f64::INFINITY => ToIntegerOrInfinityResult::Infinity,
            f64::NEG_INFINITY => ToIntegerOrInfinityResult::NegativeInfinity,
            _ => res.to_i64().map_or_else(
                || {
                    if res >= 0.0 {
                        ToIntegerOrInfinityResult::Infinity
                    } else {
                        ToIntegerOrInfinityResult::NegativeInfinity
                    }
                },
                ToIntegerOrInfinityResult::Value,
            ),
        }
    }
}

impl ToIntegerOrInfinity for f64 {
    fn to_integer_or_infinity(&self) -> f64 {
        if self.is_nan() || *self == 0.0 {
            return 0.0;
        }
        if self.is_infinite() {
            return *self;
        }
        self.trunc()
    }
}

pub enum ToIntegerOrInfinityResult {
    Infinity,
    NegativeInfinity,
    Value(i64),
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/language_semantics/src/to_integer_or_infinity/test.rs"]
#[cfg(test)]
mod test;
