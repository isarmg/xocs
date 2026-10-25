use super::*;

fn identity(mode: BuildMode) -> BuildIdentity<'static> {
    BuildIdentity {
        format: "blog-build-v1",
        product: "blog".into(),
        version: "2.0.0".into(),
        source_revision: "a".repeat(40),
        target: "x86_64-unknown-linux-gnu".into(),
        web_assets_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        source_dirty: Some(false),
        source_tree_sha256: Some("b".repeat(64)),
        build_mode: mode,
        schema_sha256: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        core_source_sha256: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    }
}

#[test]
fn bound_identity_requires_matching_profile_complete_stamp_and_clean_release() {
    for mode in [BuildMode::Candidate, BuildMode::Release] {
        let mut value = identity(mode);
        validate_build_identity(&value, false).unwrap();
        assert!(matches!(
            validate_build_identity(&value, true),
            Err(Error::BoundProfile)
        ));
        value.source_tree_sha256 = None;
        assert!(matches!(
            validate_build_identity(&value, false),
            Err(Error::MissingSourceStamp)
        ));
    }
    let mut value = identity(BuildMode::Release);
    value.source_dirty = Some(true);
    assert!(matches!(
        validate_build_identity(&value, false),
        Err(Error::DirtyRelease)
    ));
    value.build_mode = BuildMode::Candidate;
    validate_build_identity(&value, false).unwrap();
}

#[test]
fn development_identity_has_an_explicit_unavailable_source_stamp() {
    let mut value = identity(BuildMode::Development);
    value.source_revision = "unbound".into();
    value.source_dirty = None;
    value.source_tree_sha256 = None;
    validate_build_identity(&value, true).unwrap();
    assert!(matches!(
        validate_build_identity(&value, false),
        Err(Error::DevelopmentIdentity)
    ));
    value.source_dirty = Some(false);
    assert!(matches!(
        validate_build_identity(&value, true),
        Err(Error::IncompleteSourceStamp)
    ));
    value.source_tree_sha256 = Some("invalid".into());
    assert!(matches!(
        validate_build_identity(&value, true),
        Err(Error::SourceTreeHash)
    ));
}

#[test]
fn build_modes_use_the_same_closed_set_at_environment_and_json_boundaries() {
    for mode in [
        BuildMode::Development,
        BuildMode::Candidate,
        BuildMode::Release,
    ] {
        let encoded = serde_json::to_string(&mode).unwrap();
        assert_eq!(encoded, format!("\"{}\"", mode.as_str()));
        assert_eq!(serde_json::from_str::<BuildMode>(&encoded).unwrap(), mode);
        assert_eq!(mode.as_str().parse::<BuildMode>().unwrap(), mode);
    }
    assert!(matches!(
        "preview".parse::<BuildMode>(),
        Err(Error::BuildMode)
    ));
    assert!(serde_json::from_str::<BuildMode>("\"preview\"").is_err());
}
