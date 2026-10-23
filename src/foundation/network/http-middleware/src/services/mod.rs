//! Streaming file services used for media and local development resources.
#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "fs")]
pub use self::fs::{ServeDir, ServeFile};
