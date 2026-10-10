use crate::{ApiResult, AppError, AppState, absent, db_error, invalid};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use xcss::server_cli::{ContractPath as Path, ContractQuery as Query};

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Category {
    id: i64,
    sort_name: String,
    sort_description: Option<String>,
    priority: Option<i64>,
    article_count: i64,
}
#[derive(Serialize, sqlx::FromRow)]
pub(super) struct PublicSiteStats {
    article_count: i64,
    view_count: i64,
}
pub(super) async fn public_site_stats(State(state): State<AppState>) -> ApiResult<PublicSiteStats> {
    Ok(Json(sqlx::query_as::<_, PublicSiteStats>("SELECT COUNT(*) AS article_count,COALESCE(SUM(view_count),0) AS view_count FROM article WHERE deleted=0 AND (view_status=1 OR password IS NOT NULL)")
        .fetch_one(&state.pool).await.map_err(db_error)?))
}
pub(super) async fn categories(State(state): State<AppState>) -> ApiResult<Vec<Category>> {
    Ok(Json(
        sqlx::query_as::<_, Category>(
            "SELECT id,sort_name,sort_description,priority,(SELECT COUNT(*) FROM article a WHERE a.sort_id=sort.id AND a.deleted=0 AND (a.view_status=1 OR a.password IS NOT NULL)) AS article_count FROM sort ORDER BY priority DESC,id",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LabelQuery {
    sort_id: Option<i64>,
}
#[derive(Serialize, sqlx::FromRow)]
pub(super) struct PublicLabel {
    id: i64,
    sort_id: i64,
    label_name: String,
    label_description: Option<String>,
    article_count: i64,
}
pub(super) async fn public_labels(
    State(state): State<AppState>,
    Query(query): Query<LabelQuery>,
) -> ApiResult<Vec<PublicLabel>> {
    Ok(Json(sqlx::query_as::<_, PublicLabel>("SELECT l.id,l.sort_id,l.label_name,l.label_description,(SELECT COUNT(*) FROM article a WHERE a.label_id=l.id AND a.deleted=0 AND (a.view_status=1 OR a.password IS NOT NULL)) AS article_count FROM label l WHERE ? IS NULL OR l.sort_id=? ORDER BY l.id")
        .bind(query.sort_id).bind(query.sort_id).fetch_all(&state.pool).await.map_err(db_error)?))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CategoryInput {
    name: String,
    description: Option<String>,
    priority: Option<i64>,
}
pub(super) async fn create_category(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<CategoryInput>,
) -> ApiResult<Category> {
    if input.name.trim().is_empty() || input.name.chars().count() > 64 {
        return Err(invalid("分类名称无效"));
    }
    let id = sqlx::query("INSERT INTO sort(sort_name,sort_description,priority) VALUES(?,?,?)")
        .bind(input.name)
        .bind(input.description)
        .bind(input.priority.unwrap_or(0))
        .execute(&state.pool)
        .await
        .map_err(db_error)?
        .last_insert_rowid();
    Ok(Json(
        sqlx::query_as(
            "SELECT id,sort_name,sort_description,priority,0 AS article_count FROM sort WHERE id=?",
        )
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?,
    ))
}
pub(super) async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<CategoryInput>,
) -> ApiResult<Category> {
    if input.name.trim().is_empty() || input.name.chars().count() > 64 {
        return Err(invalid("分类名称无效"));
    }
    let changed =
        sqlx::query("UPDATE sort SET sort_name=?,sort_description=?,priority=? WHERE id=?")
            .bind(input.name.trim())
            .bind(input.description)
            .bind(input.priority.unwrap_or(0))
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(
        sqlx::query_as("SELECT id,sort_name,sort_description,priority,(SELECT COUNT(*) FROM article a WHERE a.sort_id=sort.id AND a.deleted=0 AND (a.view_status=1 OR a.password IS NOT NULL)) AS article_count FROM sort WHERE id=?")
            .bind(id)
            .fetch_one(&state.pool)
            .await
            .map_err(db_error)?,
    ))
}
pub(super) async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let used: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM article WHERE sort_id=? AND deleted=0")
            .bind(id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(db_error)?;
    if used > 0 {
        return Err(AppError(StatusCode::CONFLICT, "分类中还有文章"));
    }
    let removes_last_section: i64 = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM home_sections WHERE sort_id=? AND enabled=1) AND NOT EXISTS(SELECT 1 FROM home_sections WHERE enabled=1 AND (sort_id IS NULL OR sort_id<>?))",
    )
    .bind(id)
    .bind(id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(db_error)?;
    if removes_last_section != 0 {
        return Err(AppError(
            StatusCode::CONFLICT,
            "请先启用其他首页栏目，再删除此分类",
        ));
    }
    let result = sqlx::query("DELETE FROM sort WHERE id=?")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(absent());
    }
    // Labels have no foreign-key cascade in the current database contract.
    // Remove them in the same transaction so deleted category IDs can be reused safely.
    sqlx::query("DELETE FROM label WHERE sort_id=?")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(serde_json::json!({"deleted":true})))
}
