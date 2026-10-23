#[test]
#[cfg(not(miri))]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("../../../../test/foundation/sync/lazy-global/integration/compile_fail/*.rs");
}
