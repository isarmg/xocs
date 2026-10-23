//! Memory arena and arena-backed containers for the JavaScript compiler.
//!
//! [`Allocator`] owns the storage used by the syntax tree and related analysis.
//! The container types use that storage for their allocations:
//!
//! * [`Box`], [`Vec`] and [`String`]
//! * [`HashMap`] and [`HashSet`]
//! * [`BitSet`] for indexed membership
//!
//! [`CloneIn`] copies values into an explicitly selected allocator, while
//! [`GetAllocator`] and [`GetAddress`] expose the allocator and address protocols.
//! See [`Allocator`] for allocation lifetime and destruction requirements.

mod accessor;
mod address;
mod alloc;
mod allocator;
mod allocator_api2;
#[cfg(all())]
mod bitset;
mod boxed;
mod clone_in;
mod convert;
#[cfg(all(any(), target_pointer_width = "64", target_endian = "little"))]
mod fixed_size;
#[cfg(any())]
mod from_raw_parts;
pub mod hash_map;
pub mod hash_set;
#[cfg(any())]
mod pool;
mod replace_with;
mod string_builder;
mod take_in;
#[cfg(all(any(), not(any())))]
mod tracking;
mod vec;
mod vec2;

// Only expose `arena` module for doc tests
#[cfg(not(any()))]
mod arena;
#[cfg(any())]
pub mod arena;

pub use accessor::GetAllocator;
pub use address::{Address, GetAddress, UnstableAddress};
pub use allocator::Allocator;
#[cfg(all())]
pub use bitset::BitSet;
pub use boxed::{Box, Box as ArenaBox};
pub use clone_in::{CloneIn, CloneInSemanticIds};
pub use convert::{FromIn, IntoIn};
pub use hash_map::{HashMap, HashMap as ArenaHashMap};
pub use hash_set::{HashSet, HashSet as ArenaHashSet};
#[cfg(any())]
pub use pool::*;
pub use replace_with::ReplaceWith;
pub use string_builder::{StringBuilder, StringBuilder as ArenaStringBuilder};
pub use take_in::{Dummy, TakeIn};
pub use vec::{Vec, Vec as ArenaVec};

// Fixed size allocators are only supported on 64-bit little-endian platforms at present.
//
// Note: Importing the `fixed_size_constants` module would cause a compilation error on 32-bit systems.
#[cfg(all(any(), target_pointer_width = "64", target_endian = "little"))]
mod generated {
    #[cfg(debug_assertions)]
    mod assert_layouts;
    pub mod fixed_size_constants;
}
