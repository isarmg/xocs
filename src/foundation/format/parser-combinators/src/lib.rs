//! Parser combinators for strings, byte slices and token streams.
//!
//! [`Parser`] is the core interface. Functions, literals and composed parsers
//! implement it. Use [`Parser::parse`] for a complete input and
//! [`Parser::parse_next`] when a caller controls the remaining input buffer.
//!
//! - [`combinator`] provides sequencing, alternatives, repetition and error cuts.
//! - [`ascii`], [`binary`] and [`token`] provide primitive parsers.
//! - [`stream::Partial`] marks incomplete input and reports the additional data
//!   required through [`error::ErrMode::Incomplete`].
//! - [`stream::TokenSlice`] accepts tokens produced by a separate lexer.
//! - [`error::ParserError`] and [`Parser::context`] define error and context handling.
//!
//! ## Example
//!
//! ```rust
//! use blog_format_parser_combinators::{ascii::dec_uint, prelude::*};
//!
//! fn number(input: &mut &str) -> ModalResult<u64> {
//!     dec_uint.parse_next(input)
//! }
//!
//! assert_eq!(number.parse("42").unwrap(), 42);
//! ```
//!
//! `std` is enabled by default and includes `alloc`. Allocation-free consumers
//! can disable default features. The product uses the APIs selected by its local
//! Cargo consumers; parser errors and streaming state remain explicit at each
//! boundary.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(extended_key_value_attributes))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![warn(missing_docs)]
#![warn(clippy::std_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

#[cfg(feature = "alloc")]
#[cfg_attr(test, macro_use)]
#[allow(unused_extern_crates)]
extern crate alloc;

#[doc = include_str!("../../../../../docs/dependencies/origins/winnow-0.7.15/README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

pub(crate) mod util {
    #[allow(dead_code)]
    pub(crate) fn from_fn<F: Fn(&mut core::fmt::Formatter<'_>) -> core::fmt::Result>(
        f: F,
    ) -> FromFn<F> {
        FromFn(f)
    }

    pub(crate) struct FromFn<F>(F)
    where
        F: Fn(&mut core::fmt::Formatter<'_>) -> core::fmt::Result;

    impl<F> core::fmt::Debug for FromFn<F>
    where
        F: Fn(&mut core::fmt::Formatter<'_>) -> core::fmt::Result,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            (self.0)(f)
        }
    }

    impl<F> core::fmt::Display for FromFn<F>
    where
        F: Fn(&mut core::fmt::Formatter<'_>) -> core::fmt::Result,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            (self.0)(f)
        }
    }
}

#[macro_use]
mod macros;

#[macro_use]
pub mod error;

mod parser;

pub mod stream;

pub mod ascii;
pub mod binary;
pub mod combinator;
pub mod token;

#[cfg(feature = "unstable-doc")]
pub mod _topic;
#[cfg(feature = "unstable-doc")]
pub mod _tutorial;

/// Core concepts available for glob import
///
/// Including
/// - [`StreamIsPartial`][crate::stream::StreamIsPartial]
/// - [`Parser`]
///
/// ## Example
///
/// ```rust
/// use winnow::prelude::*;
///
/// fn parse_data(input: &mut &str) -> ModalResult<u64> {
///     // ...
/// #   winnow::ascii::dec_uint(input)
/// }
///
/// fn main() {
///   let result = parse_data.parse("100");
///   assert_eq!(result, Ok(100));
/// }
/// ```
pub mod prelude {
    pub use crate::error::ModalError as _;
    pub use crate::error::ParserError as _;
    pub use crate::stream::AsChar as _;
    pub use crate::stream::ContainsToken as _;
    pub use crate::stream::Stream as _;
    pub use crate::stream::StreamIsPartial as _;
    pub use crate::ModalParser;
    pub use crate::ModalResult;
    pub use crate::Parser;
    #[cfg(feature = "unstable-recover")]
    #[cfg(feature = "std")]
    pub use crate::RecoverableParser as _;

    #[cfg(test)]
include!("../../../../../test/foundation/format/parser-combinators/unit/src/lib/prelude/3815.rs");
}

pub use error::ModalResult;
pub use error::Result;
pub use parser::*;
pub use stream::BStr;
pub use stream::Bytes;
pub use stream::LocatingSlice;
pub use stream::Partial;
pub use stream::Stateful;
pub use stream::Str;

#[cfg(test)]
include!("../../../../../test/foundation/format/parser-combinators/unit/src/lib/4088.rs");
