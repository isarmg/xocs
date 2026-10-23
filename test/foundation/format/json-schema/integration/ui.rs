#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("../../../../test/foundation/format/json-schema/integration/ui/*.rs");
}
