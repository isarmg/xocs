use super::*;

#[test]
fn non_unicode_environment_values_stay_outside_build_diagnostics() {
    use std::{error::Error as _, ffi::OsString, os::unix::ffi::OsStringExt};

    let value = OsString::from_vec(b"private-web-build-sentinel\xff".to_vec());
    let failure = Error::Environment {
        name: "BLOG_SOURCE_DIRTY",
        source: env::VarError::NotUnicode(value.clone()),
    };
    for rendered in [failure.to_string(), format!("{failure:?}")] {
        assert!(rendered.contains("BLOG_SOURCE_DIRTY"));
        assert!(!rendered.contains("private-web-build-sentinel"));
    }
    assert!(matches!(
        failure.source().unwrap().downcast_ref::<env::VarError>(),
        Some(env::VarError::NotUnicode(source)) if source == &value,
    ));
}
