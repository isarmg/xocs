//! Generic values for two component properties.

use crate::style_optimizer::error::{ParserError, PrinterError};
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::{IsCompatible, Parse, ToCss};
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;
use crate::style_parser::*;

/// A generic value that represents a value with two components, e.g. a border radius.
///
/// When serialized, only a single component will be written if both are equal.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
pub struct Size2D<T>(pub T, pub T);

impl<'i, T> Parse<'i> for Size2D<T>
where
  T: Parse<'i> + Clone,
{
  fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ParserError<'i>>> {
    let first = T::parse(input)?;
    let second = input.try_parse(T::parse).unwrap_or_else(|_| first.clone());
    Ok(Size2D(first, second))
  }
}

impl<T> ToCss for Size2D<T>
where
  T: ToCss + PartialEq,
{
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    self.0.to_css(dest)?;
    if self.1 != self.0 {
      dest.write_str(" ")?;
      self.1.to_css(dest)?;
    }
    Ok(())
  }
}

impl<T: IsCompatible> IsCompatible for Size2D<T> {
  fn is_compatible(&self, browsers: crate::style_optimizer::targets::Browsers) -> bool {
    self.0.is_compatible(browsers) && self.1.is_compatible(browsers)
  }
}
