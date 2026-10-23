//! Terminal text wrapping, indentation and reflow for local CLI consumers.
//!
//! [`wrap()`] returns individual lines; [`fill()`] joins wrapped lines into a
//! string. [`Options`] selects the width, indentation, word boundaries,
//! splitting and line-breaking algorithm. [`unfill()`] and [`refill()`] handle
//! text that already contains line breaks.
//!
//! ```
//! use blog_text_line_layout::{wrap, Options, WordSplitter};
//! let options = Options::new(8).word_splitter(WordSplitter::NoHyphenation);
//! assert_eq!(wrap("foo bar-baz", options), vec!["foo", "bar-baz"]);
//! ```
//!
//! ## Features and width
//!
//! The default features are `std`, `unicode-linebreak`, `unicode-width` and
//! `smawk`. Unicode line breaking uses the local boundary-analysis package;
//! `unicode-width` measures displayed columns rather than UTF-8 byte counts.
//! `smawk` selects the optimal-fit algorithm. With those features disabled,
//! the package supplies its ASCII-space separator and first-fit algorithm.
//!
//! Set the available width explicitly with [`Options::new`] or
//! [`Options::width`]. Word splitting can preserve whole words, split existing
//! hyphens or use a custom callback. Automatic terminal-size detection and
//! external hyphenation dictionaries are not enabled in this package.

#![doc(html_root_url = "https://docs.rs/textwrap/0.16.4")]
#![forbid(unsafe_code)] // See https://github.com/mgeisler/textwrap/issues/210
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// Make `cargo test` execute the README doctests.
#[cfg(all(doctest, feature = "std"))]
#[doc = include_str!("../../../../../docs/dependencies/origins/textwrap-0.16.4/README.md")]
mod readme_doctest {}

pub mod core;
#[cfg(fuzzing)]
pub mod fuzzing;
pub mod word_splitters;
pub mod wrap_algorithms;

mod columns;
mod fill;
mod indentation;
mod line_ending;
mod options;
mod refill;
#[cfg(feature = "terminal_size")]
mod termwidth;
mod word_separators;
mod wrap;

pub use columns::wrap_columns;
pub use fill::{fill, fill_inplace};
pub use indentation::{dedent, indent};
pub use line_ending::LineEnding;
pub use options::Options;
pub use refill::{refill, unfill};
#[cfg(feature = "terminal_size")]
pub use termwidth::termwidth;
pub use word_separators::WordSeparator;
pub use word_splitters::WordSplitter;
pub use wrap::wrap;
pub use wrap_algorithms::WrapAlgorithm;
