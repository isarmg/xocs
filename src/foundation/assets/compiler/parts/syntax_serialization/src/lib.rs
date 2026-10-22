#[cfg(any())]
mod serialize;
#[cfg(any())]
pub use serialize::*;

/// Placeholder for real `ESTree` trait when `serialize` feature not enabled.
///
/// Provided to support `#[generate_derive(ESTree)]`, without enabling the feature.
#[cfg(not(any()))]
pub trait ESTree {}
