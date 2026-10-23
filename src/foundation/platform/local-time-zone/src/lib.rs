#![warn(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![allow(unknown_lints)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]

//! Discover the current IANA timezone name on Linux AMD64 GNU/glibc and browser WASM.
//!
//! [`get_timezone`] returns the host's timezone name. It does not bundle a timezone database or
//! convert timestamps; the maintained date-time package handles local calendar conversion.
//!
//! ```rust
//! let timezone = blog_platform_local_time_zone::get_timezone()?;
//! println!("{timezone}");
//! # Ok::<(), blog_platform_local_time_zone::GetTimezoneError>(())
//! ```

#[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
#[path = "tz_linux.rs"]
mod platform;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[path = "tz_wasm32_unknown.rs"]
mod platform;
#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"),
    all(target_arch = "wasm32", target_os = "unknown")
)))]
compile_error!("timezone lookup supports Linux amd64 GNU and browser wasm32");

/// Error types
#[derive(Debug)]
pub enum GetTimezoneError {
    /// Failed to parse
    FailedParsingString,
    /// Wrapped IO error
    IoError(std::io::Error),
    /// Platform-specific error from the operating system
    OsError,
}

impl std::error::Error for GetTimezoneError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GetTimezoneError::FailedParsingString => None,
            GetTimezoneError::IoError(err) => Some(err),
            GetTimezoneError::OsError => None,
        }
    }
}

impl std::fmt::Display for GetTimezoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(match self {
            GetTimezoneError::FailedParsingString => "GetTimezoneError::FailedParsingString",
            GetTimezoneError::IoError(err) => return err.fmt(f),
            GetTimezoneError::OsError => "OsError",
        })
    }
}

impl From<std::io::Error> for GetTimezoneError {
    fn from(orig: std::io::Error) -> Self {
        GetTimezoneError::IoError(orig)
    }
}

/// Get the current IANA time zone as a string.
///
/// See the module-level documentation for a usage example and more details
/// about this function.
#[inline]
pub fn get_timezone() -> Result<String, GetTimezoneError> {
    platform::get_timezone_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_current() {
        println!("current: {}", get_timezone().unwrap());
    }
}
