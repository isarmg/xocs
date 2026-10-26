use super::environment_error_message;
use std::{env::VarError, ffi::OsString, os::unix::ffi::OsStringExt};

#[test]
fn environment_diagnostics_report_the_failure_without_the_value() {
    assert_eq!(
        environment_error_message("PRIVATE_TEST_VALUE", VarError::NotPresent),
        "environment variable \"PRIVATE_TEST_VALUE\" is not set"
    );
    let secret = OsString::from_vec(b"review-sentinel\xff".to_vec());
    assert_eq!(
        environment_error_message("PRIVATE_TEST_VALUE", VarError::NotUnicode(secret)),
        "environment variable \"PRIVATE_TEST_VALUE\" is not valid Unicode"
    );
}
