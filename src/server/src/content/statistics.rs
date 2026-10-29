use crate::{ApiResult, AppState};
use axum::{Json, extract::State};
use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Statistics {
    articles: i64,
    comments: i64,
    members: i64,
    categories: i64,
}
pub(super) async fn statistics(State(state): State<AppState>) -> ApiResult<Statistics> {
    // One statement gives every count the same SQLite read snapshot.
    Ok(Json(
        sqlx::query_as(
            "SELECT
                (SELECT COUNT(*) FROM article WHERE deleted=0) AS articles,
                (SELECT COUNT(*) FROM comment) AS comments,
                (SELECT COUNT(*) FROM user WHERE deleted=0) AS members,
                (SELECT COUNT(*) FROM sort) AS categories",
        )
        .fetch_one(&state.pool)
        .await?,
    ))
}
