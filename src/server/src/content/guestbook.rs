use super::{AdminPageQuery, PageResult};
use crate::{ApiResult, AppState, absent, db_error, invalid};
use axum::{Json, extract::State};
use blog_application::admin_core::AuthenticatedIdentity;
use blog_application::server_cli::{ContractPath as Path, ContractQuery as Query};
use serde::{Deserialize, Serialize};

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
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let id = sqlx::query("INSERT INTO tree_hole(message) VALUES(?)")
        .bind(message)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?
        .last_insert_rowid();
    let entry = sqlx::query_as("SELECT t.id,t.user_id,u.username,t.avatar,t.message,t.image_path,t.create_time FROM tree_hole t LEFT JOIN user u ON u.id=t.user_id WHERE t.id=?")
        .bind(id).fetch_one(&mut *transaction).await.map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(entry))
}

pub(super) async fn content_tree_hole(
    State(state): State<AppState>,
    Query(query): Query<AdminPageQuery>,
) -> ApiResult<PageResult<TreeHole>> {
    let (page, size) = query.bounds();
    let mut transaction = state.pool.begin().await.map_err(db_error)?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tree_hole")
        .fetch_one(&mut *transaction)
        .await
        .map_err(db_error)?;
    let items = sqlx::query_as::<_, TreeHole>(
        "SELECT t.id,t.user_id,u.username,t.avatar,t.message,t.image_path,t.create_time FROM tree_hole t LEFT JOIN user u ON u.id=t.user_id ORDER BY t.id DESC LIMIT ? OFFSET ?",
    )
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&mut *transaction)
    .await
    .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
pub(super) async fn delete_tree_hole(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let changed = sqlx::query("DELETE FROM tree_hole WHERE id=?")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(serde_json::json!({"deleted": true})))
}
