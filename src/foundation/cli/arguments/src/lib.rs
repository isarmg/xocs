// Copyright ⓒ 2015-2016 Kevin B. Knapp and [`clap-rs` contributors](https://github.com/clap-rs/clap/graphs/contributors).
// Licensed under the MIT license
// (see LICENSE or <http://opensource.org/licenses/MIT>) All files in the project carrying such
// notice may not be copied, modified, or distributed except according to those terms.

//! Command-line argument parsing for the Blog server and setup tools.
//!
//! [`Command`], [`Arg`] and [`ArgAction`] define a parser through the builder API.
//! [`ArgMatches`] provides typed access to parsed values; [`Error`] includes
//! structured error kinds and help/version outcomes.
//!
//! # Builder example
//!
//! ```
//! use blog_cli_arguments::{Arg, Command};
//!
//! let matches = Command::new("blog-tool")
//!     .arg(Arg::new("root").long("root").required(true))
//!     .try_get_matches_from(["blog-tool", "--root", "/srv/blog"])?;
//! assert_eq!(matches.get_one::<String>("root").map(String::as_str), Some("/srv/blog"));
//! # Ok::<(), blog_cli_arguments::Error>(())
//! ```
//!
//! # Derive example
//!
//! The `derive` feature exports the [`Parser`], [`Args`], [`Subcommand`] and
//! [`ValueEnum`] macros alongside their traits. The macros generate paths through
//! the consumer name `clap`; examples therefore explicitly bind this local library
//! to that name. Builder-only consumers use `blog_cli_arguments` directly.
//!
//! ```
//! extern crate blog_cli_arguments as clap;
//! # #[cfg(feature = "derive")]
//! # fn main() -> Result<(), blog_cli_arguments::Error> {
//! use blog_cli_arguments::Parser;
//!
//! #[derive(Parser)]
//! struct Options {
//!     #[arg(long)]
//!     root: String,
//! }
//!
//! let options = Options::try_parse_from(["blog-tool", "--root", "/srv/blog"])?;
//! assert_eq!(options.root, "/srv/blog");
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "derive"))]
//! # fn main() {}
//! ```
//!
//! # Features and reference material
//!
//! Default features include standard-library support, color, generated help and
//! usage, error context and suggestions. `derive`, `cargo`, `env` and `string`
//! enable parser macros, Cargo metadata macros, environment defaults and owned
//! parser strings respectively. The product's local manifest selects these features.
//!
//! Full tutorials and examples are stored in this repository under
//! `docs/dependencies/guides/cli/`; the `unstable-doc` feature also exposes the
//! `_tutorial`, `_derive`, `_cookbook`, `_concepts`, `_faq` and `_features` modules
//! in generated API documentation. Current product commands are described by
//! `blog --help`, `blog-setup --help` and `blog-assets --help`.
//!

#![doc(html_logo_url = "https://raw.githubusercontent.com/clap-rs/clap/master/assets/clap.png")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

// Copyright ⓒ 2015-2016 Kevin B. Knapp and [`clap-rs` contributors](https://github.com/clap-rs/clap/graphs/contributors).
// Licensed under the MIT license
// (see LICENSE or <http://opensource.org/licenses/MIT>) All files in the project carrying such
// notice may not be copied, modified, or distributed except according to those terms.


#[cfg(not(feature = "std"))]
compile_error!("`std` feature is currently required to build `command-line`");

pub use crate::builder::ArgAction;
pub use crate::builder::Command;
pub use crate::builder::ValueHint;
pub use crate::builder::{Arg, ArgGroup};
pub use crate::parser::ArgMatches;
pub use crate::util::Id;
pub use crate::util::color::ColorChoice;

/// Command Line Argument Parser Error
///
/// See [`Command::error`] to create an error.
///
/// [`Command::error`]: crate::Command::error
pub type Error = error::Error<error::DefaultFormatter>;

pub use crate::derive::{Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};

#[macro_use]
#[allow(missing_docs)]
mod macros;

mod derive;

pub mod builder;
pub mod error;
pub mod parser;

mod mkeymap;
mod output;
mod util;

const INTERNAL_ERROR_MSG: &str = "Fatal internal error. Please consider filing a bug \
                                  report at https://github.com/clap-rs/clap/issues";

#[doc = include_str!("../../../../../docs/dependencies/guides/cli/builder/README.md")]
#[cfg(doctest)]
pub struct BuilderReadmeDoctests;


// Imported tokenizer retains its reviewed unsafe OS-string slicing.
#[allow(unsafe_code)]
mod lexer;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use clap_derive::{self, Args, Parser, Subcommand, ValueEnum};

#[cfg(feature = "unstable-doc")]
pub mod _concepts;
#[cfg(feature = "unstable-doc")]
pub mod _cookbook;
#[cfg(feature = "unstable-doc")]
pub mod _derive;
#[cfg(feature = "unstable-doc")]
pub mod _faq;
#[cfg(feature = "unstable-doc")]
pub mod _features;
#[cfg(feature = "unstable-doc")]
pub mod _tutorial;

#[doc = include_str!("../../../../../docs/dependencies/guides/cli/runtime/README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
