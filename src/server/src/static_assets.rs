use axum::{extract::Request, response::Response};
use blog_application::web_assets::build::BuildMode;

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("compiled build identity digest differs")]
    DigestMismatch,
    #[error("invalid compiled build identity JSON")]
    Json(#[from] serde_json::Error),
    #[error("compiled identity is not canonical")]
    NonCanonical,
    #[error("compiled identity differs from this product's compiled inputs")]
    InputMismatch,
    #[error("development identity must be unbound debug")]
    InvalidDevelopment,
    #[error("development source stamp must be complete or unavailable")]
    IncompleteSourceStamp,
    #[error("invalid source tree hash")]
    InvalidSourceHash,
    #[error("candidate and release require bound release compilation")]
    InvalidBoundCompilation,
    #[error("bound source stamp is missing")]
    MissingSourceStamp,
    #[error("release identity requires clean source")]
    DirtyRelease,
    #[error("invalid embedded web assets")]
    WebAssets(#[from] blog_application::web_assets::Error),
}

include!(concat!(env!("OUT_DIR"), "/blog-browser-assets.rs"));

pub fn verify() -> Result<(), AssetError> {
    verify_identity()?;
    blog_application::web_assets::verify_embedded(ASSETS, MANIFEST, DIGEST)?;
    Ok(())
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
    build_mode: BuildMode,
    schema_sha256: String,
    core_source_sha256: String,
}

pub(crate) fn verify_identity() -> Result<(), AssetError> {
    verify_identity_bytes(BUILD_IDENTITY, BUILD_IDENTITY_SHA256)
}

fn verify_identity_bytes(bytes: &[u8], expected_sha256: &str) -> Result<(), AssetError> {
    use sha2::{Digest, Sha256};
    let digest: String = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if digest != expected_sha256 {
        return Err(AssetError::DigestMismatch);
    }
    let identity: CompiledIdentity = serde_json::from_slice(bytes)?;
    if serde_json::to_vec(&identity)? != bytes {
        return Err(AssetError::NonCanonical);
    }
    if !(identity.format == "blog-build-v1"
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
        && identity.build_mode.as_str() == env!("BLOG_COMPILED_BUILD_MODE")
        && identity.web_assets_sha256 == DIGEST
        && identity.schema_sha256 == env!("BLOG_SCHEMA_SHA256")
        && identity.core_source_sha256 == env!("BLOG_CORE_SOURCE_SHA256"))
    {
        return Err(AssetError::InputMismatch);
    }
    let hash = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
    };
    match identity.build_mode {
        BuildMode::Development => {
            if !cfg!(debug_assertions) || identity.source_revision != "unbound" {
                return Err(AssetError::InvalidDevelopment);
            }
            if identity.source_dirty.is_some() != identity.source_tree_sha256.is_some() {
                return Err(AssetError::IncompleteSourceStamp);
            }
            if let Some(tree) = &identity.source_tree_sha256
                && !hash(tree)
            {
                return Err(AssetError::InvalidSourceHash);
            }
        }
        BuildMode::Candidate | BuildMode::Release => {
            if !(!cfg!(debug_assertions)
                && identity.source_revision.len() == 40
                && identity
                    .source_revision
                    .bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f')))
            {
                return Err(AssetError::InvalidBoundCompilation);
            }
            if identity.source_dirty.is_none()
                || !identity.source_tree_sha256.as_deref().is_some_and(hash)
            {
                return Err(AssetError::MissingSourceStamp);
            }
            if identity.build_mode == BuildMode::Release && identity.source_dirty != Some(false) {
                return Err(AssetError::DirtyRelease);
            }
        }
    }
    Ok(())
}

#[path = "../../../test/server/unit/static_assets/identity_tests.rs"]
#[cfg(test)]
mod identity_tests;
