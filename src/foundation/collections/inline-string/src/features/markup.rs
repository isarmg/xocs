#[cfg(test)]
include!("../../../../../../test/foundation/collections/inline-string/unit/src/features/markup/0.rs");

use markup::Render;

use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "markup")))]
impl Render for CompactString {
    #[inline]
    fn render(&self, writer: &mut impl core::fmt::Write) -> core::fmt::Result {
        self.as_str().render(writer)
    }
}

#[cfg(test)]
include!("../../../../../../test/foundation/collections/inline-string/unit/src/features/markup/311.rs");
