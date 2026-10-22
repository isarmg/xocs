//! The `@viewport` rule.

use super::Location;
use crate::style_optimizer::declaration::DeclarationBlock;
use crate::style_optimizer::error::PrinterError;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::ToCss;
use crate::style_optimizer::vendor_prefix::VendorPrefix;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;

/// A [@viewport](https://drafts.csswg.org/css-device-adapt/#atviewport-rule) rule.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
#[cfg_attr(
  any(),
  derive(serde::Serialize, serde::Deserialize),
  serde(rename_all = "camelCase")
)]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
pub struct ViewportRule<'i> {
  /// The vendor prefix for this rule, e.g. `@-ms-viewport`.
  #[cfg_attr(all(), skip_visit)]
  pub vendor_prefix: VendorPrefix,
  /// The declarations within the `@viewport` rule.
  #[cfg_attr(any(), serde(borrow))]
  pub declarations: DeclarationBlock<'i>,
  /// The location of the rule in the source file.
  #[cfg_attr(all(), skip_visit)]
  pub loc: Location,
}

impl<'i> ToCss for ViewportRule<'i> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    #[cfg(any())]
    dest.add_mapping(self.loc);
    dest.write_char('@')?;
    self.vendor_prefix.to_css(dest)?;
    dest.write_str("viewport")?;
    self.declarations.to_css_block(dest)
  }
}
