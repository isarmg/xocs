// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(not(miri))]

#[rustversion::attr(not(nightly), ignore)]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("../../../../test/foundation/sync/pinned-projection/integration/ui/**/*.rs");
    t.pass("../../../../test/foundation/sync/pinned-projection/integration/run-pass/**/*.rs");
}
