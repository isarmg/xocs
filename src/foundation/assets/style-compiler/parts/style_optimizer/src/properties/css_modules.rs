//! Properties related to CSS modules.

use crate::style_optimizer::dependencies::Location;
use crate::style_optimizer::error::{ParserError, PrinterError};
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::{Parse, ToCss};
use crate::style_optimizer::values::ident::{CustomIdent, CustomIdentList};
use crate::style_optimizer::values::string::CowArcStr;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;
use crate::style_parser::*;
use smallvec::SmallVec;

/// A value for the [composes](https://github.com/css-modules/css-modules/#dependencies) property from CSS modules.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
#[cfg_attr(any(), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
pub struct Composes<'i> {
  /// A list of class names to compose.
  #[cfg_attr(any(), serde(borrow))]
  pub names: CustomIdentList<'i>,
  /// Where the class names are composed from.
  pub from: Option<Specifier<'i>>,
  /// The source location of the `composes` property.
  pub loc: Location,
}

/// Defines where the class names referenced in the `composes` property are located.
///
/// See [Composes].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
#[cfg_attr(
  any(),
  derive(serde::Serialize, serde::Deserialize),
  serde(tag = "type", content = "value", rename_all = "kebab-case")
)]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
pub enum Specifier<'i> {
  /// The referenced name is global.
  Global,
  /// The referenced name comes from the specified file.
  #[cfg_attr(any(), serde(borrow))]
  File(CowArcStr<'i>),
  /// The referenced name comes from a source index (used during bundling).
  SourceIndex(u32),
}

impl<'i> Parse<'i> for Composes<'i> {
  fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ParserError<'i>>> {
    let loc = input.current_source_location();
    let mut names = SmallVec::new();
    while let Ok(name) = input.try_parse(parse_one_ident) {
      names.push(name);
    }

    if names.is_empty() {
      return Err(input.new_custom_error(ParserError::InvalidDeclaration));
    }

    let from = if input.try_parse(|input| input.expect_ident_matching("from")).is_ok() {
      Some(Specifier::parse(input)?)
    } else {
      None
    };

    Ok(Composes {
      names,
      from,
      loc: loc.into(),
    })
  }
}

fn parse_one_ident<'i, 't>(
  input: &mut Parser<'i, 't>,
) -> Result<CustomIdent<'i>, ParseError<'i, ParserError<'i>>> {
  let name = CustomIdent::parse(input)?;
  if name.0.eq_ignore_ascii_case("from") {
    return Err(input.new_error_for_next_token());
  }

  Ok(name)
}

impl ToCss for Composes<'_> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    let mut first = true;
    for name in &self.names {
      if first {
        first = false;
      } else {
        dest.write_char(' ')?;
      }
      name.to_css(dest)?;
    }

    if let Some(from) = &self.from {
      dest.write_str(" from ")?;
      from.to_css(dest)?;
    }

    Ok(())
  }
}

impl<'i> Parse<'i> for Specifier<'i> {
  fn parse<'t>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i, ParserError<'i>>> {
    if let Ok(file) = input.try_parse(|input| input.expect_string_cloned()) {
      Ok(Specifier::File(file.into()))
    } else {
      input.expect_ident_matching("global")?;
      Ok(Specifier::Global)
    }
  }
}

impl<'i> ToCss for Specifier<'i> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    match self {
      Specifier::Global => dest.write_str("global")?,
      Specifier::File(file) => serialize_string(&file, dest)?,
      Specifier::SourceIndex(..) => {}
    }
    Ok(())
  }
}
