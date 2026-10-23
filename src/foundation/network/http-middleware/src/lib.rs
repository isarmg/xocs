//! HTTP middleware maintained for Blog's file delivery and request tracing.
//! Compression, CORS, proxy redirects, authorization and generic middleware
//! combinators are deliberately outside this package's supported surface.
#![forbid(unsafe_code)]
#![deny(unreachable_pub)]
#![warn(missing_docs)]

#[macro_use]
pub(crate) mod macros;
#[cfg(feature = "fs")]
pub mod body;
#[cfg(feature = "trace")]
pub mod classify;
#[cfg(feature = "fs")]
mod content_encoding;
pub mod services;
#[cfg(feature = "fs")]
pub mod set_status;
#[path = "../../../../../test/foundation/network/http-middleware/unit/src/test_helpers.rs"]
#[cfg(test)]
mod test_helpers;
#[cfg(feature = "trace")]
pub mod trace;

/// The latency unit used to report latencies by middleware.
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum LatencyUnit {
    /// Use seconds.
    Seconds,
    /// Use milliseconds.
    Millis,
    /// Use microseconds.
    Micros,
    /// Use nanoseconds.
    Nanos,
}

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
