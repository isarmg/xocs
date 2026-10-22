use axum::{extract::Request, response::Response};

include!(concat!(env!("OUT_DIR"), "/blog-browser-assets.rs"));

pub fn verify() -> anyhow::Result<()> {
    verify_identity()?;
    blog_application::web_assets::verify_embedded(ASSETS, MANIFEST, DIGEST)
}

pub async fn serve(request: Request) -> Response {
    blog_application::web_assets::wasm::response(
        request.uri().path(),
        request.method(),
        request.headers(),
        |path, method, headers| {
            blog_application::web_assets::response(ASSETS, path, method, headers)
        },
    )
    .map(axum::body::Body::from)
}

pub async fn entry(request: Request) -> Response {
    blog_application::web_assets::response(
        ASSETS,
        "/index.html",
        request.method(),
        request.headers(),
    )
    .map(axum::body::Body::from)
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct CompiledIdentity {
    format: String,
    product: String,
    version: String,
    source_revision: String,
    target: String,
    web_assets_sha256: String,
    source_dirty: Option<bool>,
    source_tree_sha256: Option<String>,
    build_mode: String,
    schema_sha256: String,
    core_source_sha256: String,
}

pub(crate) fn verify_identity() -> anyhow::Result<()> {
    verify_identity_bytes(BUILD_IDENTITY, BUILD_IDENTITY_SHA256)
}

fn verify_identity_bytes(bytes: &[u8], expected_sha256: &str) -> anyhow::Result<()> {
    use anyhow::ensure;
    use sha2::{Digest, Sha256};
    let digest: String = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    ensure!(
        digest == expected_sha256,
        "compiled build identity digest differs"
    );
    let identity: CompiledIdentity = serde_json::from_slice(bytes)?;
    ensure!(
        serde_json::to_vec(&identity)? == bytes,
        "compiled identity is not canonical"
    );
    ensure!(
        identity.format == "blog-build-v1"
            && identity.product == env!("CARGO_PKG_NAME")
            && identity.version == env!("CARGO_PKG_VERSION")
            && identity.target == env!("BLOG_BUILD_TARGET")
            && identity.source_revision == env!("BLOG_SOURCE_REVISION")
            && identity
                .source_dirty
                .map(|dirty| if dirty { "true" } else { "false" })
                .unwrap_or("unknown")
                == env!("BLOG_COMPILED_SOURCE_DIRTY")
            && identity.source_tree_sha256.as_deref().unwrap_or("unbound")
                == env!("BLOG_COMPILED_SOURCE_TREE_SHA256")
            && identity.build_mode == env!("BLOG_COMPILED_BUILD_MODE")
            && identity.web_assets_sha256 == DIGEST
            && identity.schema_sha256 == env!("BLOG_SCHEMA_SHA256")
            && identity.core_source_sha256 == env!("BLOG_CORE_SOURCE_SHA256"),
        "compiled identity differs from this product's compiled inputs"
    );
    let hash = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
    };
    match identity.build_mode.as_str() {
        "development" => {
            ensure!(
                cfg!(debug_assertions) && identity.source_revision == "unbound",
                "development identity must be unbound debug"
            );
            ensure!(
                identity.source_dirty.is_some() == identity.source_tree_sha256.is_some(),
                "development source stamp must be complete or unavailable"
            );
            if let Some(tree) = &identity.source_tree_sha256 {
                ensure!(hash(tree), "invalid source tree hash");
            }
        }
        "candidate" | "release" => {
            ensure!(
                !cfg!(debug_assertions)
                    && identity.source_revision.len() == 40
                    && identity
                        .source_revision
                        .bytes()
                        .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f')),
                "candidate and release require bound release compilation"
            );
            ensure!(
                identity.source_dirty.is_some()
                    && identity.source_tree_sha256.as_deref().is_some_and(hash),
                "bound source stamp is missing"
            );
            ensure!(
                identity.build_mode != "release" || identity.source_dirty == Some(false),
                "release identity requires clean source"
            );
        }
        _ => anyhow::bail!("invalid compiled build mode"),
    }
    Ok(())
}

#[cfg(test)]
mod identity_tests {
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
                "source_dirty" => {
                    identity.source_dirty = Some(!identity.source_dirty.unwrap_or(false))
                }
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
}
