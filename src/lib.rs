mod admin;
mod config;
mod content;
mod home;
mod lifecycle;
mod media;
mod schema;
mod static_assets;

mod app;
mod database;

pub(crate) use app::{
    ApiResult, AppError, AppState, absent, db_error, invalid, now_micros, site_author_id,
};
use xcss::server_cli::CliError as CliFailure;
pub use xcss::server_cli::ContractJson;

const PRODUCT_ID: &str = "xocs";

pub mod offline;
mod runtime;

pub async fn run(root: &std::path::Path) -> anyhow::Result<()> {
    runtime::run(root).await
}

#[cfg(test)]
mod tests;
