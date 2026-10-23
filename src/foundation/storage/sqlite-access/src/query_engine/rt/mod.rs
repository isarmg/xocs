//! Tokio support used by the SQLite pool. Direct worker connections do not need
//! a runtime; timed pool acquisition and background pool maintenance do.
use std::future::Future;
use std::time::Duration;

pub use tokio::task::{spawn, yield_now};
pub use tokio::time::sleep;

#[derive(Debug, thiserror::Error)]
#[error("operation timed out")]
pub struct TimeoutError;

pub async fn timeout<F: Future>(duration: Duration, future: F) -> Result<F::Output, TimeoutError> {
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| TimeoutError)
}
