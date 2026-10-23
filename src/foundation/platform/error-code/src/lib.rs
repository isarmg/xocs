//! Linux interface to the thread-local `errno` variable.
//!
//! # Examples
//! ```
//! use errno::{Errno, errno, set_errno};
//!
//! // Get the current value of errno
//! let e = errno();
//!
//! // Set the current value of errno
//! set_errno(e);
//!
//! // Extract the error code as an i32
//! let code = e.0;
//!
//! // Display a human-friendly error message
//! println!("Error {}: {}", code, e);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu")))]
compile_error!("errno access is a Linux amd64 GNU native dependency");
#[path = "unix.rs"]
mod sys;

use core::fmt;
#[cfg(feature = "std")]
use std::error::Error;
#[cfg(feature = "std")]
use std::io;

/// Wraps a platform-specific error code.
///
/// The `Display` instance maps the code to a human-readable string using
/// [`strerror_r`](https://pubs.opengroup.org/onlinepubs/009695399/functions/strerror.html).
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Errno(pub i32);

impl fmt::Debug for Errno {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        sys::with_description(*self, |desc| {
            fmt.debug_struct("Errno")
                .field("code", &self.0)
                .field("description", &desc.ok())
                .finish()
        })
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        sys::with_description(*self, |desc| match desc {
            Ok(desc) => fmt.write_str(desc),
            Err(fm_err) => write!(
                fmt,
                "OS error {} ({} returned error {})",
                self.0,
                sys::STRERROR_NAME,
                fm_err.0
            ),
        })
    }
}

impl From<Errno> for i32 {
    fn from(e: Errno) -> Self {
        e.0
    }
}

#[cfg(feature = "std")]
impl Error for Errno {}

#[cfg(feature = "std")]
impl From<Errno> for io::Error {
    fn from(errno: Errno) -> Self {
        io::Error::from_raw_os_error(errno.0)
    }
}

/// Returns the platform-specific value of `errno`.
pub fn errno() -> Errno {
    sys::errno()
}

/// Sets the platform-specific value of `errno`.
pub fn set_errno(err: Errno) {
    sys::set_errno(err)
}

#[test]
fn it_works() {
    let x = errno();
    set_errno(x);
}

#[cfg(feature = "std")]
#[test]
fn it_works_with_to_string() {
    let x = errno();
    let _ = x.to_string();
}

#[cfg(feature = "std")]
#[test]
fn check_description() {
    let expect = "Operation not permitted";
    let errno_code = libc::EPERM;
    set_errno(Errno(errno_code));

    assert_eq!(errno().to_string(), expect);
    assert_eq!(
        format!("{:?}", errno()),
        format!(
            "Errno {{ code: {}, description: Some({:?}) }}",
            errno_code, expect
        )
    );
}

#[cfg(feature = "std")]
#[test]
fn check_error_into_errno() {
    const ERROR_CODE: i32 = 1;

    let error = io::Error::from_raw_os_error(ERROR_CODE);
    let new_error: io::Error = Errno(ERROR_CODE).into();
    assert_eq!(error.kind(), new_error.kind());
}
