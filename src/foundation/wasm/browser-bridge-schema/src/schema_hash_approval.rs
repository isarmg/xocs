// The build script hashes the encoded AST and descriptor opcode definitions in
// src/lib.rs and src/tys.rs. Incidental comment, formatting, or non-protocol
// helper changes require review and an updated approval hash. Actual wire-format
// changes also require a new repository SCHEMA_VERSION (blog-browser-bridge/N).
// The generator and macro must agree exactly on that protocol identity; Cargo
// versions and Git revisions do not define compatibility.
//
// The version helper now reports the fixed protocol identity rather than a Git
// revision. The encoded AST and descriptor opcode definitions are unchanged.
const APPROVED_SCHEMA_FILE_HASH: &str = "16027499667722352305";

#[test]
fn schema_version() {
    assert_eq!(env!("SCHEMA_FILE_HASH"), APPROVED_SCHEMA_FILE_HASH)
}
