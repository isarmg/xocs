use crate::{ApiResult, AppState, db_error};
use axum::{Json, extract::State};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct Statistics {
    articles: i64,
    comments: i64,
    members: i64,
    categories: i64,
}
pub(super) async fn statistics(State(state): State<AppState>) -> ApiResult<Statistics> {
    let articles: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM article WHERE deleted=0")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let comments: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comment")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let members: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user WHERE deleted=0")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let categories: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sort")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    Ok(Json(Statistics {
        articles,
        comments,
        members,
        categories,
    }))
}
