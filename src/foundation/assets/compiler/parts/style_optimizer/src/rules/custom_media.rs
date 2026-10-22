//! The `@custom-media` rule.

use super::Location;
use crate::style_optimizer::error::PrinterError;
use crate::style_optimizer::media_query::MediaList;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::ToCss;
use crate::style_optimizer::values::ident::DashedIdent;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;

/// A [@custom-media](https://drafts.csswg.org/mediaqueries-5/#custom-mq) rule.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
#[cfg_attr(any(), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
pub struct CustomMediaRule<'i> {
  /// The name of the declared media query.
  #[cfg_attr(any(), serde(borrow))]
  pub name: DashedIdent<'i>,
  /// The media query to declare.
  pub query: MediaList<'i>,
  /// The location of the rule in the source file.
  #[cfg_attr(all(), skip_visit)]
  pub loc: Location,
}

impl<'i> ToCss for CustomMediaRule<'i> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    #[cfg(any())]
    dest.add_mapping(self.loc);
    dest.write_str("@custom-media ")?;
    self.name.to_css(dest)?;
    dest.write_char(' ')?;
    self.query.to_css(dest)?;
    dest.write_char(';')
  }
}
