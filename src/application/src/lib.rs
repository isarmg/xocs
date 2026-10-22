//! Repository-owned Linux AMD64 support for blog.
#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]
#[cfg(not(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu")))]
compile_error!("blog-application supports only x86_64-unknown-linux-gnu");
extern crate self as blog_application;
pub mod admin_auth;
pub mod admin_axum;
pub mod admin_core;
pub mod admin_sqlite;
pub mod config;
pub mod configuration_contract;
pub mod contracts;
pub mod error;
pub mod fs_safety;
pub mod installation_material;
pub mod log;
pub(crate) mod platform_db;
pub mod schema_identity;
pub(crate) mod secret_envelope;
pub mod server_cli;
pub mod server_layout;
pub mod server_runtime;
pub mod server_target;
pub mod sqlite;
pub mod state_file;
#[cfg(test)]
pub(crate) mod testkit;
pub mod web_assets;
