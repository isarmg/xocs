#[path = "api.rs"]
#[cfg(all(feature = "dfa-build", feature = "dfa-search"))]
mod api;
#[path = "onepass/mod.rs"]
#[cfg(feature = "dfa-onepass")]
mod onepass;
#[path = "regression.rs"]
#[cfg(all(feature = "dfa-build", feature = "dfa-search"))]
mod regression;
#[path = "suite.rs"]
#[cfg(all(not(miri), feature = "dfa-build", feature = "dfa-search"))]
mod suite;
