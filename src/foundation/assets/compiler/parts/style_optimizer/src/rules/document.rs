//! The `@-moz-document` rule.

use super::Location;
use super::{CssRuleList, MinifyContext};
use crate::style_optimizer::error::{MinifyError, PrinterError};
use crate::style_optimizer::parser::DefaultAtRule;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::ToCss;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;

/// A [@-moz-document](https://www.w3.org/TR/2012/WD-css3-conditional-20120911/#at-document) rule.
///
/// Note that only the `url-prefix()` function with no arguments is supported, and only the `-moz` prefix
/// is allowed since Firefox was the only browser that ever implemented this rule.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
pub struct MozDocumentRule<'i, R = DefaultAtRule> {
  /// Nested rules within the `@-moz-document` rule.
  #[cfg_attr(any(), serde(borrow))]
  pub rules: CssRuleList<'i, R>,
  /// The location of the rule in the source file.
  #[cfg_attr(all(), skip_visit)]
  pub loc: Location,
}

impl<'i, T: Clone> MozDocumentRule<'i, T> {
  pub(crate) fn minify(&mut self, context: &mut MinifyContext<'_, 'i>) -> Result<(), MinifyError> {
    self.rules.minify(context, false)
  }
}

impl<'i, T: ToCss> ToCss for MozDocumentRule<'i, T> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    #[cfg(any())]
    dest.add_mapping(self.loc);
    dest.write_str("@-moz-document url-prefix()")?;
    dest.whitespace()?;
    dest.write_char('{')?;
    dest.indent();
    dest.newline()?;
    self.rules.to_css(dest)?;
    dest.dedent();
    dest.newline()?;
    dest.write_char('}')
  }
}
