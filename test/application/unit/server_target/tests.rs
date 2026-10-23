use super::*;

#[test]
fn only_the_canonical_server_target_is_accepted() {
    require_server_target(SERVER_TARGET_TRIPLE).unwrap();
    for rejected in [
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "x86_64-pc-windows-msvc",
        "amd64",
        "",
    ] {
        let error = require_server_target(rejected).unwrap_err();
        assert_eq!(error.actual(), rejected);
    }
}

#[test]
fn compile_time_target_matches_public_identity() {
    assert_eq!(std::env::consts::ARCH, "x86_64");
    assert_eq!(std::env::consts::OS, "linux");
    assert_eq!(SERVER_ARCHITECTURE, "amd64");
}
