/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Make |cargo bench| work.
#![cfg_attr(any(), feature(test))]

use crate::style_parser as cssparser;

pub mod attr;
pub mod bloom;
mod builder;
pub mod context;
pub mod matching;
mod nth_index_cache;
pub mod parser;
pub mod sink;
mod tree;
pub mod visitor;

#[cfg(all(any()))]
mod serialization;

pub use crate::style_selectors::nth_index_cache::NthIndexCache;
pub use crate::style_selectors::parser::{Parser, SelectorImpl, SelectorList};
pub use crate::style_selectors::tree::{Element, OpaqueElement};
