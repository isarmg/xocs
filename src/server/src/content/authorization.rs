//! Recheck revocable administrator authority under the content writer lock.
use blog_application::{admin_core::AuthenticatedIdentity, error::HttpStatus};
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{AppError, db_error};

pub(crate) async fn begin_mutation(
    pool: &SqlitePool,
    identity: &AuthenticatedIdentity,
) -> Result<Transaction<'static, Sqlite>, AppError> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await.map_err(db_error)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(AppError::Clock)?
        .as_micros()
        .try_into()
        .map_err(AppError::TimestampRange)?;
    if !blog_application::admin_sqlite::session_authorized_on_connection(
        &mut transaction,
        identity,
        now,
    )
    .await
    .map_err(AppError::AdministratorStore)?
    {
        return Err(AppError::Public(
            HttpStatus::Unauthorized,
            "管理员会话已失效，请重新登录",
        ));
    }
    Ok(transaction)
}
