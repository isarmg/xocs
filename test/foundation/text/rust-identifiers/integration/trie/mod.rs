#![allow(clippy::module_inception)]

#[path = "trie.rs"]
#[allow(dead_code, clippy::redundant_static_lifetimes, clippy::unreadable_literal)]
#[rustfmt::skip]
mod trie;

pub(crate) use self::trie::*;
