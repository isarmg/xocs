// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Traits for data providers that produce opaque buffers.

use crate::prelude::*;

#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "serde")]
pub use self::serde::*;

/// [`DynamicDataMarker`] for raw buffers. Returned by [`BufferProvider`].
///
/// The consumer must decode the payload according to its buffer format before use.
#[non_exhaustive]
#[derive(Debug)]
pub struct BufferMarker;

impl DynamicDataMarker for BufferMarker {
    type DataStruct = &'static [u8];
}

/// A data provider that returns opaque bytes.
///
/// Along with [`DataProvider`], this is one of the two foundational traits in this crate.
/// A buffer provider can be used as a trait object and passed through FFI.
/// Loading a buffer does not decode it: consumers must supply a decoder for the
/// expected [`BufferFormat`], or use a typed [`DataProvider`] instead.
/// [`BufferFormat::check_available`] reports whether an integrated decoder is enabled.
pub trait BufferProvider: DynamicDataProvider<BufferMarker> {}

impl<P: DynamicDataProvider<BufferMarker> + ?Sized> BufferProvider for P {}

/// An enum expressing all Serde formats known to ICU4X.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[non_exhaustive]
pub enum BufferFormat {
    /// JavaScript Object Notation (JSON) buffers.
    Json,
    /// Buffers encoded in the bincode version 1 format.
    Bincode1,
    /// Buffers encoded in the Postcard version 1 format.
    Postcard1,
}

impl BufferFormat {
    /// Returns an error if the buffer format is not enabled.
    pub fn check_available(&self) -> Result<(), DataError> {
        match self {
            #[cfg(feature = "deserialize_json")]
            BufferFormat::Json => Ok(()),
            #[cfg(not(feature = "deserialize_json"))]
            BufferFormat::Json => Err(DataErrorKind::Deserialize.with_str_context("deserializing `BufferFormat::Json` requires the `deserialize_json` Cargo feature")),

            #[cfg(feature = "deserialize_bincode_1")]
            BufferFormat::Bincode1 => Ok(()),
            #[cfg(not(feature = "deserialize_bincode_1"))]
            BufferFormat::Bincode1 => Err(DataErrorKind::Deserialize.with_str_context("deserializing `BufferFormat::Bincode1` requires the `deserialize_bincode_1` Cargo feature")),

            #[cfg(feature = "deserialize_postcard_1")]
            BufferFormat::Postcard1 => Ok(()),
            #[cfg(not(feature = "deserialize_postcard_1"))]
            BufferFormat::Postcard1 => Err(DataErrorKind::Deserialize.with_str_context("deserializing `BufferFormat::Postcard1` requires the `deserialize_postcard_1` Cargo feature")),
        }
    }
}
