//! The `@starting-style` rule.

use super::Location;
use super::{CssRuleList, MinifyContext};
use crate::style_optimizer::error::{MinifyError, PrinterError};
use crate::style_optimizer::parser::DefaultAtRule;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::ToCss;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;

/// A [@starting-style](https://drafts.csswg.org/css-transitions-2/#defining-before-change-style-the-starting-style-rule) rule.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
pub struct StartingStyleRule<'i, R = DefaultAtRule> {
  /// Nested rules within the `@starting-style` rule.
  #[cfg_attr(any(), serde(borrow))]
  pub rules: CssRuleList<'i, R>,
  /// The location of the rule in the source file.
  #[cfg_attr(all(), skip_visit)]
  pub loc: Location,
}

impl<'i, T: Clone> StartingStyleRule<'i, T> {
  pub(crate) fn minify(
    &mut self,
    context: &mut MinifyContext<'_, 'i>,
    parent_is_unused: bool,
  ) -> Result<bool, MinifyError> {
    self.rules.minify(context, parent_is_unused)?;
    Ok(self.rules.0.is_empty())
  }
}

impl<'i, T: ToCss> ToCss for StartingStyleRule<'i, T> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    #[cfg(any())]
    dest.add_mapping(self.loc);
    dest.write_str("@starting-style")?;
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
