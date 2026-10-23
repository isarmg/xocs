/*!
Regular-expression parsing, enabled by the `syntax` feature.

[`Ast`](crate::syntax::ast::Ast) preserves the structured concrete syntax.
[`Hir`](crate::syntax::hir::Hir) is a simpler representation for analysis and
compilation. [`Parser`](crate::syntax::Parser) combines AST parsing and HIR
translation; [`parse`](crate::syntax::parse) uses the default configuration.
Use [`ast::parse::Parser`](crate::syntax::ast::parse::Parser) or
[`hir::translate::Translator`](crate::syntax::hir::translate::Translator) when
these steps need separate configuration.

```
use blog_text_pattern_search::syntax::{hir::Hir, parse};
let hir = parse("a|b")?;
assert_eq!(hir, Hir::alternation(vec![
    Hir::literal("a".as_bytes()),
    Hir::literal("b".as_bytes()),
]));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Parsers accept a nesting limit for untrusted patterns. AST and HIR traversal
and destruction avoid recursive use of the call stack; literal extraction
has its own resource costs. Errors implement `Display` for human-readable
pattern diagnostics.

Unicode properties are selected by the package's `unicode-*` features. Parsing
a property whose data is disabled returns an error. Syntax and literal-search
implementations share this package; no separate parser download is needed.
*/

#![forbid(unsafe_code)]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![warn(missing_debug_implementations)]
// This adds Cargo feature annotations to items in the rustdoc output. Which is
// sadly hugely beneficial for this crate due to the number of features.
#![cfg_attr(docsrs_regex, feature(doc_cfg))]

#[cfg(any(test, feature = "std"))]
extern crate std;

extern crate alloc;

pub use crate::syntax::{
    error::Error,
    parser::{parse, Parser, ParserBuilder},
    unicode::UnicodeWordError,
};

use alloc::string::String;

pub mod ast;
mod debug;
mod either;
mod error;
pub mod hir;
mod parser;
mod rank;
mod unicode;
mod unicode_tables;
pub mod utf8;

/// Escapes all regular expression meta characters in `text`.
///
/// The string returned may be safely used as a literal in a regular
/// expression.
pub fn escape(text: &str) -> String {
    let mut quoted = String::new();
    escape_into(text, &mut quoted);
    quoted
}

/// Escapes all meta characters in `text` and writes the result into `buf`.
///
/// This will append escape characters into the given buffer. The characters
/// that are appended are safe to use as a literal in a regular expression.
pub fn escape_into(text: &str, buf: &mut String) {
    buf.reserve(text.len());
    for c in text.chars() {
        if is_meta_character(c) {
            buf.push('\\');
        }
        buf.push(c);
    }
}

/// Returns true if the given character has significance in a regex.
///
/// Generally speaking, these are the only characters which _must_ be escaped
/// in order to match their literal meaning. For example, to match a literal
/// `|`, one could write `\|`. Sometimes escaping isn't always necessary. For
/// example, `-` is treated as a meta character because of its significance
/// for writing ranges inside of character classes, but the regex `-` will
/// match a literal `-` because `-` has no special meaning outside of character
/// classes.
///
/// In order to determine whether a character may be escaped at all, the
/// [`is_escapeable_character`] routine should be used. The difference between
/// `is_meta_character` and `is_escapeable_character` is that the latter will
/// return true for some characters that are _not_ meta characters. For
/// example, `%` and `\%` both match a literal `%` in all contexts. In other
/// words, `is_escapeable_character` includes "superfluous" escapes.
///
/// Note that the set of characters for which this function returns `true` or
/// `false` is fixed and won't change in a semver compatible release. (In this
/// case, "semver compatible release" actually refers to the `regex` crate
/// itself, since reducing or expanding the set of meta characters would be a
/// breaking change for not just `regex-syntax` but also `regex` itself.)
///
/// # Example
///
/// ```
/// use blog_text_pattern_search::syntax::is_meta_character;
///
/// assert!(is_meta_character('?'));
/// assert!(is_meta_character('-'));
/// assert!(is_meta_character('&'));
/// assert!(is_meta_character('#'));
///
/// assert!(!is_meta_character('%'));
/// assert!(!is_meta_character('/'));
/// assert!(!is_meta_character('!'));
/// assert!(!is_meta_character('"'));
/// assert!(!is_meta_character('e'));
/// ```
pub fn is_meta_character(c: char) -> bool {
    match c {
        '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{'
        | '}' | '^' | '$' | '#' | '&' | '-' | '~' => true,
        _ => false,
    }
}

/// Returns true if the given character can be escaped in a regex.
///
/// This returns true in all cases that `is_meta_character` returns true, but
/// also returns true in some cases where `is_meta_character` returns false.
/// For example, `%` is not a meta character, but it is escapable. That is,
/// `%` and `\%` both match a literal `%` in all contexts.
///
/// The purpose of this routine is to provide knowledge about what characters
/// may be escaped. Namely, most regex engines permit "superfluous" escapes
/// where characters without any special significance may be escaped even
/// though there is no actual _need_ to do so.
///
/// This will return false for some characters. For example, `e` is not
/// escapable. Therefore, `\e` will either result in a parse error (which is
/// true today), or it could backwards compatibly evolve into a new construct
/// with its own meaning. Indeed, that is the purpose of banning _some_
/// superfluous escapes: it provides a way to evolve the syntax in a compatible
/// manner.
///
/// # Example
///
/// ```
/// use blog_text_pattern_search::syntax::is_escapeable_character;
///
/// assert!(is_escapeable_character('?'));
/// assert!(is_escapeable_character('-'));
/// assert!(is_escapeable_character('&'));
/// assert!(is_escapeable_character('#'));
/// assert!(is_escapeable_character('%'));
/// assert!(is_escapeable_character('/'));
/// assert!(is_escapeable_character('!'));
/// assert!(is_escapeable_character('"'));
///
/// assert!(!is_escapeable_character('e'));
/// ```
pub fn is_escapeable_character(c: char) -> bool {
    // Certainly escapable if it's a meta character.
    if is_meta_character(c) {
        return true;
    }
    // Any character that isn't ASCII is definitely not escapable. There's
    // no real need to allow things like \☃ right?
    if !c.is_ascii() {
        return false;
    }
    // Otherwise, we basically say that everything is escapable unless it's a
    // letter or digit. Things like \3 are either octal (when enabled) or an
    // error, and we should keep it that way. Otherwise, letters are reserved
    // for adding new syntax in a backwards compatible way.
    match c {
        '0'..='9' | 'A'..='Z' | 'a'..='z' => false,
        // While not currently supported, we keep these as not escapable to
        // give us some flexibility with respect to supporting the \< and
        // \> word boundary assertions in the future. By rejecting them as
        // escapable, \< and \> will result in a parse error. Thus, we can
        // turn them into something else in the future without it being a
        // backwards incompatible change.
        //
        // OK, now we support \< and \>, and we need to retain them as *not*
        // escapable here since the escape sequence is significant.
        '<' | '>' => false,
        _ => true,
    }
}

/// Returns true if and only if the given character is a Unicode word
/// character.
///
/// A Unicode word character is defined by
/// [UTS#18 Annex C](https://unicode.org/reports/tr18/#Compatibility_Properties).
/// In particular, a character
/// is considered a word character if it is in either of the `Alphabetic` or
/// `Join_Control` properties, or is in one of the `Decimal_Number`, `Mark`
/// or `Connector_Punctuation` general categories.
///
/// # Panics
///
/// If the `unicode-perl` feature is not enabled, then this function
/// panics. For this reason, it is recommended that callers use
/// [`try_is_word_character`] instead.
pub fn is_word_character(c: char) -> bool {
    try_is_word_character(c).expect("unicode-perl feature must be enabled")
}

/// Returns true if and only if the given character is a Unicode word
/// character.
///
/// A Unicode word character is defined by
/// [UTS#18 Annex C](https://unicode.org/reports/tr18/#Compatibility_Properties).
/// In particular, a character
/// is considered a word character if it is in either of the `Alphabetic` or
/// `Join_Control` properties, or is in one of the `Decimal_Number`, `Mark`
/// or `Connector_Punctuation` general categories.
///
/// # Errors
///
/// If the `unicode-perl` feature is not enabled, then this function always
/// returns an error.
pub fn try_is_word_character(
    c: char,
) -> core::result::Result<bool, UnicodeWordError> {
    unicode::is_word_character(c)
}

/// Returns true if and only if the given character is an ASCII word character.
///
/// An ASCII word character is defined by the following character class:
/// `[_0-9a-zA-Z]`.
pub fn is_word_byte(c: u8) -> bool {
    match c {
        b'_' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => true,
        _ => false,
    }
}

#[cfg(test)]
include!("../../../../../../test/foundation/text/pattern-search/unit/src/syntax/mod/9381.rs");
