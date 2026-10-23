//! An unknown at-rule.

use super::Location;
use crate::style_optimizer::error::PrinterError;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::properties::custom::TokenList;
use crate::style_optimizer::traits::ToCss;
use crate::style_optimizer::values::string::CowArcStr;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;

/// An unknown at-rule, stored as raw tokens.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
#[cfg_attr(any(), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
pub struct UnknownAtRule<'i> {
  /// The name of the at-rule (without the @).
  #[cfg_attr(any(), serde(borrow))]
  #[cfg_attr(all(), skip_visit)]
  pub name: CowArcStr<'i>,
  /// The prelude of the rule.
  pub prelude: TokenList<'i>,
  /// The contents of the block, if any.
  pub block: Option<TokenList<'i>>,
  /// The location of the rule in the source file.
  #[cfg_attr(all(), skip_visit)]
  pub loc: Location,
}

impl<'i> ToCss for UnknownAtRule<'i> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    #[cfg(any())]
    dest.add_mapping(self.loc);
    dest.write_char('@')?;
    dest.write_str(&self.name)?;

    if !self.prelude.0.is_empty() {
      dest.write_char(' ')?;
      self.prelude.to_css(dest, false)?;
    }

    if let Some(block) = &self.block {
      dest.whitespace()?;
      dest.write_char('{')?;
      dest.indent();
      dest.newline()?;
      block.to_css(dest, false)?;
      dest.dedent();
      dest.newline()?;
      dest.write_char('}')
    } else {
      dest.write_char(';')
    }
  }
}
