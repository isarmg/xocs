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
    for field in [
        "product",
        "version",
        "source_revision",
        "source_dirty",
        "source_tree_sha256",
        "build_mode",
        "schema_sha256",
        "core_source_sha256",
        "web_assets_sha256",
    ] {
        let mut identity: CompiledIdentity = serde_json::from_slice(BUILD_IDENTITY).unwrap();
        match field {
            "product" => identity.product = "foreign".into(),
            "version" => identity.version = "999.0.0".into(),
            "source_revision" => identity.source_revision = "a".repeat(40),
            "source_dirty" => identity.source_dirty = Some(!identity.source_dirty.unwrap_or(false)),
            "source_tree_sha256" => identity.source_tree_sha256 = Some("e".repeat(64)),
            "build_mode" => identity.build_mode = "release".into(),
            "schema_sha256" => identity.schema_sha256 = "a".repeat(64),
            "core_source_sha256" => identity.core_source_sha256 = "b".repeat(64),
            "web_assets_sha256" => identity.web_assets_sha256 = "c".repeat(64),
            _ => unreachable!(),
        }
        let bytes = serde_json::to_vec(&identity).unwrap();
        assert!(
            verify_identity_bytes(&bytes, &digest(&bytes)).is_err(),
            "{field}"
        );
    }
    assert!(verify_identity_bytes(BUILD_IDENTITY, &"0".repeat(64)).is_err());
}
