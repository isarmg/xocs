use crate::rate_limit;
use crate::{ApiResult, AppState, db_error, invalid};
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct TreeHole {
    id: i64,
    user_id: Option<i64>,
    username: Option<String>,
    avatar: Option<String>,
    message: String,
    image_path: Option<String>,
    create_time: Option<String>,
}
pub(super) async fn tree_hole(State(state): State<AppState>) -> ApiResult<Vec<TreeHole>> {
    Ok(Json(
        sqlx::query_as::<_, TreeHole>(
            "SELECT t.id,t.user_id,u.username,t.avatar,t.message,t.image_path,t.create_time FROM tree_hole t LEFT JOIN user u ON u.id=t.user_id ORDER BY t.id DESC LIMIT 100",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TreeHoleInput {
    message: String,
    image_path: Option<String>,
}
pub(super) async fn create_guest_tree_hole(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    crate::ContractJson(input): crate::ContractJson<TreeHoleInput>,
) -> ApiResult<TreeHole> {
    let message = input.message.trim();
    if message.is_empty()
        || message.chars().count() > 60
        || message.contains('<')
        || message.contains('>')
        || input.image_path.is_some()
    {
        return Err(invalid("留言不能为空、不能超过 60 个字，也不能包含图片"));
    }
    let visitor = rate_limit::visitor_ip(peer, &headers, &state.trusted_proxies)?;
    let key = rate_limit::attempt_key("guest_tree_hole", &visitor.to_string(), "");
    rate_limit::record_attempt(&state.pool, &key, 5).await?;
    let id = sqlx::query("INSERT INTO tree_hole(message) VALUES(?)")
        .bind(message)
        .execute(&state.pool)
        .await
        .map_err(db_error)?
        .last_insert_rowid();
    Ok(Json(sqlx::query_as("SELECT t.id,t.user_id,u.username,t.avatar,t.message,t.image_path,t.create_time FROM tree_hole t LEFT JOIN user u ON u.id=t.user_id WHERE t.id=?")
        .bind(id).fetch_one(&state.pool).await.map_err(db_error)?))
}
