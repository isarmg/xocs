use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use sha2::{Digest, Sha256};
use sqlx::{SqliteConnection, SqlitePool};

use crate::{AppError, db_error};

const WINDOW_SECONDS: i64 = 15 * 60;

pub(crate) async fn record_attempt(
    pool: &SqlitePool,
    key: &[u8],
    limit: i64,
) -> Result<(), AppError> {
    let mut connection = pool.acquire().await.map_err(db_error)?;
    record_attempt_on(&mut connection, key, limit).await
}

pub(crate) async fn record_attempt_on(
    connection: &mut SqliteConnection,
    key: &[u8],
    limit: i64,
) -> Result<(), AppError> {
    let now = i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "时钟不可用"))?
            .as_secs(),
    )
    .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "时钟不可用"))?;
    // Anonymous publishing and protected article access use this limiter.
    let count: i64 = sqlx::query_scalar("INSERT INTO member_login_failures(failure_key,failures,expires_at) VALUES(?,1,?) ON CONFLICT(failure_key) DO UPDATE SET failures=CASE WHEN expires_at<=? THEN 1 ELSE failures+1 END,expires_at=CASE WHEN expires_at<=? THEN excluded.expires_at ELSE expires_at END RETURNING failures")
        .bind(key).bind(now + WINDOW_SECONDS).bind(now).bind(now)
        .fetch_one(connection).await.map_err(db_error)?;
    if count > limit {
        return Err(AppError(
            StatusCode::TOO_MANY_REQUESTS,
            "请求过于频繁，请稍后再试",
        ));
    }
    Ok(())
}

pub(crate) fn attempt_key(kind: &str, ip: &str, subject: &str) -> Vec<u8> {
    let mut hash = Sha256::new();
    hash.update(kind.as_bytes());
    hash.update([0]);
    hash.update(ip.as_bytes());
    hash.update([0]);
    hash.update(subject.as_bytes());
    hash.finalize().to_vec()
}
