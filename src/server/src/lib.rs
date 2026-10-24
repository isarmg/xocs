mod config;
mod config_definition;
mod content;
mod home;
mod lifecycle;
mod media;
mod schema;
mod schema_definition;
mod static_assets;

mod app;
mod database;

#[cfg(any(test, feature = "maintenance"))]
pub(crate) use app::now_micros;
pub(crate) use app::{ApiResult, AppError, AppState, absent, db_error, invalid, site_author_id};
pub use blog_application::server_cli::ContractJson;

const PRODUCT_ID: &str = "blog";

mod runtime;

pub async fn run(root: &std::path::Path) -> anyhow::Result<()> {
    runtime::run(root).await
}

#[cfg(test)]
#[path = "../../../test/server/unit/tests.rs"]
mod tests;

#[cfg(any(test, feature = "maintenance"))]
mod administrator;

#[cfg(feature = "maintenance")]
pub mod maintenance;

/// Verify the identity compiled into this product before runtime or offline work.
/// This consumes canonical identity bytes and their digest in both executables.
pub fn verify_compiled_identity() -> anyhow::Result<()> {
    static_assets::verify_identity()
}
