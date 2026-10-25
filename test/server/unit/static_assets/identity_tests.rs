use super::*;
use sha2::{Digest, Sha256};

#[test]
fn compiled_identity_is_consumed_and_rejects_changed_product_schema_or_source() {
    verify_identity().unwrap();
    let digest = |bytes: &[u8]| -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    };
    #[derive(Debug)]
    enum Field {
        Product,
        Version,
        SourceRevision,
        SourceDirty,
        SourceTree,
        BuildMode,
        Schema,
        CoreSource,
        WebAssets,
    }
    for field in [
        Field::Product,
        Field::Version,
        Field::SourceRevision,
        Field::SourceDirty,
        Field::SourceTree,
        Field::BuildMode,
        Field::Schema,
        Field::CoreSource,
        Field::WebAssets,
    ] {
        let mut identity: CompiledIdentity = serde_json::from_slice(BUILD_IDENTITY).unwrap();
        match field {
            Field::Product => identity.product = "foreign".into(),
            Field::Version => identity.version = "999.0.0".into(),
            Field::SourceRevision => identity.source_revision = "a".repeat(40),
            Field::SourceDirty => {
                identity.source_dirty = Some(!identity.source_dirty.unwrap_or(false))
            }
            Field::SourceTree => identity.source_tree_sha256 = Some("e".repeat(64)),
            Field::BuildMode => identity.build_mode = BuildMode::Release,
            Field::Schema => identity.schema_sha256 = "a".repeat(64),
            Field::CoreSource => identity.core_source_sha256 = "b".repeat(64),
            Field::WebAssets => identity.web_assets_sha256 = "c".repeat(64),
        }
        let bytes = serde_json::to_vec(&identity).unwrap();
        assert!(
            verify_identity_bytes(&bytes, &digest(&bytes)).is_err(),
            "{field:?}"
        );
    }
    assert!(verify_identity_bytes(BUILD_IDENTITY, &"0".repeat(64)).is_err());
}

#[test]
fn unknown_compiled_build_mode_is_rejected_at_json_boundary() {
    let mut identity: serde_json::Value = serde_json::from_slice(BUILD_IDENTITY).unwrap();
    identity["build_mode"] = serde_json::json!("unsupported");
    let bytes = serde_json::to_vec(&identity).unwrap();
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert!(matches!(
        verify_identity_bytes(&bytes, &digest),
        Err(AssetError::Json(_))
    ));
}
