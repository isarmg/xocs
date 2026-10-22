//! The `@namespace` rule.

use super::Location;
use crate::style_optimizer::error::PrinterError;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::ToCss;
use crate::style_optimizer::values::ident::Ident;
use crate::style_optimizer::values::string::CSSString;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;

/// A [@namespace](https://drafts.csswg.org/css-namespaces/#declaration) rule.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
#[cfg_attr(any(), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
pub struct NamespaceRule<'i> {
  /// An optional namespace prefix to declare, or `None` to declare the default namespace.
  #[cfg_attr(any(), serde(borrow))]
  #[cfg_attr(all(), skip_visit)]
  pub prefix: Option<Ident<'i>>,
  /// The url of the namespace.
  #[cfg_attr(any(), serde(borrow))]
  #[cfg_attr(all(), skip_visit)]
  pub url: CSSString<'i>,
  /// The location of the rule in the source file.
  #[cfg_attr(all(), skip_visit)]
  pub loc: Location,
}

impl<'i> ToCss for NamespaceRule<'i> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    #[cfg(any())]
    dest.add_mapping(self.loc);
    dest.write_str("@namespace ")?;
    if let Some(prefix) = &self.prefix {
      prefix.to_css(dest)?;
      dest.write_char(' ')?;
    }

    self.url.to_css(dest)?;
    dest.write_char(';')
  }
}
