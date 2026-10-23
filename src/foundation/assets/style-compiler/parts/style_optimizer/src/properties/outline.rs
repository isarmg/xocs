//! CSS properties related to outlines.

use super::border::{BorderSideWidth, GenericBorder, LineStyle};
use super::{Property, PropertyId};
use crate::style_optimizer::context::PropertyHandlerContext;
use crate::style_optimizer::declaration::{DeclarationBlock, DeclarationList};
use crate::style_optimizer::error::{ParserError, PrinterError};
use crate::style_optimizer::macros::{impl_shorthand, shorthand_handler};
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::targets::Browsers;
use crate::style_optimizer::traits::{FallbackValues, IsCompatible, Parse, PropertyHandler, Shorthand, ToCss};
use crate::style_optimizer::values::color::CssColor;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;
use crate::style_parser::*;

/// A value for the [outline-style](https://drafts.csswg.org/css-ui/#outline-style) property.
#[derive(Debug, Clone, PartialEq, Parse, ToCss)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(
  any(),
  derive(serde::Serialize, serde::Deserialize),
  serde(tag = "type", content = "value", rename_all = "kebab-case")
)]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
pub enum OutlineStyle {
  /// The `auto` keyword.
  Auto,
  /// A value equivalent to the `border-style` property.
  LineStyle(LineStyle),
}

impl Default for OutlineStyle {
  fn default() -> OutlineStyle {
    OutlineStyle::LineStyle(LineStyle::None)
  }
}

impl IsCompatible for OutlineStyle {
  fn is_compatible(&self, _browsers: Browsers) -> bool {
    true
  }
}

/// A value for the [outline](https://drafts.csswg.org/css-ui/#outline) shorthand property.
pub type Outline = GenericBorder<OutlineStyle, 11>;

impl_shorthand! {
  Outline(Outline) {
    width: [OutlineWidth],
    style: [OutlineStyle],
    color: [OutlineColor],
  }
}

shorthand_handler!(OutlineHandler -> Outline fallbacks: true {
  width: OutlineWidth(BorderSideWidth),
  style: OutlineStyle(OutlineStyle),
  color: OutlineColor(CssColor, fallback: true),
});
