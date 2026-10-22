//! The `@import` rule.

use super::layer::LayerName;
use super::supports::SupportsCondition;
use super::Location;
use crate::style_optimizer::dependencies::{Dependency, ImportDependency};
use crate::style_optimizer::error::PrinterError;
use crate::style_optimizer::media_query::MediaList;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::ToCss;
use crate::style_optimizer::values::string::CowArcStr;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;
use crate::style_parser::*;

/// A [@import](https://drafts.csswg.org/css-cascade/#at-import) rule.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
#[cfg_attr(any(), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
pub struct ImportRule<'i> {
  /// The url to import.
  #[cfg_attr(any(), serde(borrow))]
  #[cfg_attr(all(), skip_visit)]
  pub url: CowArcStr<'i>,
  /// An optional cascade layer name, or `None` for an anonymous layer.
  #[cfg_attr(all(), skip_visit)]
  pub layer: Option<Option<LayerName<'i>>>,
  /// An optional `supports()` condition.
  pub supports: Option<SupportsCondition<'i>>,
  /// A media query.
  #[cfg_attr(any(), serde(default))]
  pub media: MediaList<'i>,
  /// The location of the rule in the source file.
  #[cfg_attr(all(), skip_visit)]
  pub loc: Location,
}

impl<'i> ToCss for ImportRule<'i> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    let dep = if dest.dependencies.is_some() {
      Some(ImportDependency::new(self, dest.filename()))
    } else {
      None
    };

    #[cfg(any())]
    dest.add_mapping(self.loc);
    dest.write_str("@import ")?;
    if let Some(dep) = dep {
      serialize_string(&dep.placeholder, dest)?;

      if let Some(dependencies) = &mut dest.dependencies {
        dependencies.push(Dependency::Import(dep))
      }
    } else {
      serialize_string(&self.url, dest)?;
    }

    if let Some(layer) = &self.layer {
      dest.write_str(" layer")?;
      if let Some(name) = layer {
        dest.write_char('(')?;
        name.to_css(dest)?;
        dest.write_char(')')?;
      }
    }

    if let Some(supports) = &self.supports {
      dest.write_str(" supports")?;
      if matches!(supports, SupportsCondition::Declaration { .. }) {
        supports.to_css(dest)?;
      } else {
        dest.write_char('(')?;
        supports.to_css(dest)?;
        dest.write_char(')')?;
      }
    }
    if !self.media.media_queries.is_empty() {
      dest.write_char(' ')?;
      self.media.to_css(dest)?;
    }
    dest.write_str(";")
  }
}
