mod admin;
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

#[cfg(test)]
pub(crate) use app::now_micros;
pub(crate) use app::{ApiResult, AppError, AppState, absent, db_error, invalid, site_author_id};
pub use xcss::server_cli::ContractJson;

const PRODUCT_ID: &str = "xocs";

mod runtime;

pub async fn run(root: &std::path::Path) -> anyhow::Result<()> {
    runtime::run(root).await
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "../tests/support/administrator.rs"]
mod test_support;
