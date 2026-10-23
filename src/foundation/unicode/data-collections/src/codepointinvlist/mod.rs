// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This module provides necessary functionality for highly efficient querying of sets of Unicode characters.
//!
//! It is an implementation of the code point portion of the existing
//! [ICU4C UnicodeSet API](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/classicu_1_1UnicodeSet.html).
//!
//! # Representation and construction
//!
//! [`CodePointInversionList`] represents set membership using sorted,
//! non-overlapping code point ranges. It can be constructed from serialized
//! inversion lists with [`CodePointInversionList::try_from_inversion_list`],
//! or obtained from a Unicode property provider. With the `alloc` feature,
//! `CodePointInversionListBuilder` additionally supports incremental construction.
//!
//! # Querying a set
//!
//! Membership, range membership and character iteration do not require allocation:
//!
//! ```
//! use blog_unicode_data_collections::codepointinvlist::CodePointInversionList;
//!
//! let set = CodePointInversionList::bmp();
//! assert!(set.contains('A'));
//! assert!(set.contains_range('A'..='C'));
//! assert_eq!(set.iter_chars().next(), Some('\0'));
//! ```

#[cfg(feature = "alloc")]
#[macro_use]
mod builder;
#[cfg(feature = "alloc")]
mod conversions;
mod cpinvlist;
mod utils;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
pub use builder::CodePointInversionListBuilder;
pub use cpinvlist::CodePointInversionList;
pub use cpinvlist::CodePointInversionListULE;
use displaydoc::Display;

#[derive(Display, Debug)]
/// A [`CodePointInversionList`] was constructed with an invalid inversion list
#[cfg_attr(feature = "alloc", displaydoc("Invalid set: {0:?}"))]
#[allow(clippy::exhaustive_structs)] // newtype
pub struct InvalidSetError(#[cfg(feature = "alloc")] pub Vec<potential_utf::PotentialCodePoint>);

/// A [`CodePointInversionList`] was constructed from an invalid range
#[derive(Display, Debug)]
#[displaydoc("Invalid range: {0}..{1}")]
#[allow(clippy::exhaustive_structs)] // newtype
pub struct RangeError(pub u32, pub u32);
