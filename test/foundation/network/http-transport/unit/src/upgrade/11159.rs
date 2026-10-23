#[cfg(all(any(feature = "client", feature = "server"), feature = "http1"))]
#[cfg(test)]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../test/foundation/network/http-transport/legacy/unit/src/upgrade/tests_376.rs"
));
