//! The `@scope` rule.

use super::Location;
use super::{CssRuleList, MinifyContext};
use crate::style_optimizer::error::{MinifyError, PrinterError};
use crate::style_optimizer::parser::DefaultAtRule;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::selector::{is_pure_css_modules_selector, SelectorList};
use crate::style_optimizer::traits::ToCss;
#[cfg(all())]
use crate::style_optimizer::visitor::Visit;

/// A [@scope](https://drafts.csswg.org/css-cascade-6/#scope-atrule) rule.
///
/// @scope (<scope-start>) [to (<scope-end>)]? {
///  <stylesheet>
/// }
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(all(), derive(Visit))]
#[cfg_attr(
  any(),
  derive(serde::Serialize, serde::Deserialize),
  serde(rename_all = "camelCase")
)]
#[cfg_attr(any(), derive(schemars::JsonSchema))]
#[cfg_attr(any(), derive(static_self::IntoOwned))]
pub struct ScopeRule<'i, R = DefaultAtRule> {
  /// A selector list used to identify the scoping root(s).
  pub scope_start: Option<SelectorList<'i>>,
  /// A selector list used to identify any scoping limits.
  pub scope_end: Option<SelectorList<'i>>,
  /// Nested rules within the `@scope` rule.
  #[cfg_attr(any(), serde(borrow))]
  pub rules: CssRuleList<'i, R>,
  /// The location of the rule in the source file.
  #[cfg_attr(all(), skip_visit)]
  pub loc: Location,
}

impl<'i, T: Clone> ScopeRule<'i, T> {
  pub(crate) fn minify(&mut self, context: &mut MinifyContext<'_, 'i>) -> Result<(), MinifyError> {
    if context.pure_css_modules {
      if let Some(scope_start) = &self.scope_start {
        if !scope_start.0.iter().all(is_pure_css_modules_selector) {
          return Err(MinifyError {
            kind: crate::style_optimizer::error::MinifyErrorKind::ImpureCSSModuleSelector,
            loc: self.loc,
          });
        }
      }

      if let Some(scope_end) = &self.scope_end {
        if !scope_end.0.iter().all(is_pure_css_modules_selector) {
          return Err(MinifyError {
            kind: crate::style_optimizer::error::MinifyErrorKind::ImpureCSSModuleSelector,
            loc: self.loc,
          });
        }
      }
    }

    self.rules.minify(context, false)
  }
}

impl<'i, T: ToCss> ToCss for ScopeRule<'i, T> {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    #[cfg(any())]
    dest.add_mapping(self.loc);
    dest.write_str("@scope")?;
    dest.whitespace()?;
    if let Some(scope_start) = &self.scope_start {
      dest.write_char('(')?;
      scope_start.to_css(dest)?;
      dest.write_char(')')?;
      dest.whitespace()?;
    }
    if let Some(scope_end) = &self.scope_end {
      if dest.minify {
        dest.write_char(' ')?;
      }
      dest.write_str("to (")?;
      // <scope-start> is treated as an ancestor of scope end.
      // https://drafts.csswg.org/css-nesting/#nesting-at-scope
      if let Some(scope_start) = &self.scope_start {
        dest.with_context(scope_start, |dest| scope_end.to_css(dest))?;
      } else {
        scope_end.to_css(dest)?;
      }
      dest.write_char(')')?;
      dest.whitespace()?;
    }
    dest.write_char('{')?;
    dest.indent();
    dest.newline()?;
    // Nested style rules within @scope are implicitly relative to the <scope-start>
    // so clear our style context while printing them to avoid replacing & ourselves.
    // https://drafts.csswg.org/css-cascade-6/#scoped-rules
    dest.with_cleared_context(|dest| self.rules.to_css(dest))?;
    dest.dedent();
    dest.newline()?;
    dest.write_char('}')
  }
}
