//! Lightning CSS is a CSS parser, transformer, and minifier based on the
//! [cssparser](https://github.com/servo/rust-cssparser) crate used in Firefox.
//! It supports fully parsing all CSS rules, properties, and values into normalized
//! structures exactly how a browser would. Once parsed, the CSS can be transformed
//! to add or remove vendor prefixes, or lower syntax for older browsers as appropriate.
//! The style sheet can also be minified to merge longhand properties into shorthands,
//! merge adjacent rules, reduce `calc()` expressions, and more. Finally, the style sheet
//! can be printed back to CSS syntax, either minified to remove whitespace and compress
//! the output as much as possible, or pretty printed.
//!
//! The [stylesheet API](crate::style_optimizer::stylesheet::StyleSheet) parses
//! and transforms entire CSS files. Individual [CSS rules](crate::style_optimizer::rules),
//! [property types](crate::style_optimizer::properties) and [value types](crate::style_optimizer::values)
//! can also be parsed and manipulated. The product's `blog-assets` tool combines
//! CSS imports before invoking this compiler. See each module for API details
//! and examples.

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any())]
#[cfg_attr(docsrs, doc(cfg(any())))]
pub mod bundler;
mod compat;
mod context;
pub mod css_modules;
pub mod declaration;
pub mod dependencies;
pub mod error;
mod logical;
mod macros;
pub mod media_query;
mod parser;
mod prefixes;
pub mod printer;
pub mod properties;
pub mod rules;
pub mod selector;
pub mod stylesheet;
pub mod targets;
pub mod traits;
pub mod values;
pub mod vendor_prefix;
#[cfg(all())]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub mod visitor;

#[cfg(any())]
mod serialization;

#[path = "../../../../../../../test/foundation/assets/style-compiler/unit/parts/style_optimizer/src/test_helpers.rs"]
#[cfg(test)]
mod test_helpers;

#[path = "../../../../../../../test/foundation/assets/style-compiler/unit/parts/style_optimizer/src/lib/tests.rs"]
#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use crate::{visit_types};
