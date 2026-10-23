#[path = "regression.rs"]
mod regression;
#[path = "suite.rs"]
#[cfg(not(miri))]
mod suite;
