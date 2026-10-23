//! Implements the [`valuable::Valuable`] trait for [`CompactString`], so it can be
//! inspected as a [`Value::String`] by object-safe value visitors (e.g. `tracing`).

use valuable::{Valuable, Value, Visit};

use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "valuable")))]
impl Valuable for CompactString {
    fn as_value(&self) -> Value<'_> {
        Value::String(self.as_str())
    }

    fn visit(&self, visit: &mut dyn Visit) {
        self.as_str().visit(visit);
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/valuable/tests.rs"]
#[cfg(test)]
mod tests;
