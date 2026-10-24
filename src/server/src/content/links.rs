use super::{AdminPageQuery, PageResult};
use crate::{ApiResult, AppError, AppState, absent, db_error, invalid};
use axum::{Json, extract::State};
use blog_application::server_cli::{ContractPath as Path, ContractQuery as Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Link {
    id: i64,
    title: Option<String>,
    classify: Option<String>,
    cover: Option<String>,
    url: Option<String>,
    introduction: Option<String>,
    link_type: Option<String>,
    create_time: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LinkQuery {
    kind: Option<String>,
}
fn valid_link_kind(kind: &str) -> bool {
    matches!(kind, "favorites" | "friendUrl" | "lovePhoto" | "funny")
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LinkPageQuery {
    kind: String,
    classify: Option<String>,
    page: Option<i64>,
    size: Option<i64>,
}
pub(super) async fn link_page(
    State(state): State<AppState>,
    Query(query): Query<LinkPageQuery>,
) -> ApiResult<PageResult<Link>> {
    if !valid_link_kind(&query.kind) || query.classify.as_ref().is_some_and(|s| s.len() > 100) {
        return Err(invalid("资源筛选无效"));
    }
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(12).clamp(1, 100);
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_path WHERE type=? AND status=1 AND (? IS NULL OR classify=?)")
        .bind(&query.kind).bind(&query.classify).bind(&query.classify)
        .fetch_one(&state.pool).await.map_err(db_error)?;
    let items=sqlx::query_as::<_,Link>("SELECT id,title,classify,cover,url,introduction,type AS link_type,create_time FROM resource_path WHERE type=? AND status=1 AND (? IS NULL OR classify=?) ORDER BY id DESC LIMIT ? OFFSET ?")
        .bind(&query.kind).bind(&query.classify).bind(&query.classify).bind(size).bind((page-1)*size)
        .fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
#[derive(Serialize, sqlx::FromRow)]
pub(super) struct LinkClass {
    classify: String,
    count: i64,
}
pub(super) async fn link_classes(
    State(state): State<AppState>,
    Query(query): Query<LinkQuery>,
) -> ApiResult<Vec<LinkClass>> {
    let kind = query.kind.unwrap_or_else(|| "lovePhoto".into());
    if !valid_link_kind(&kind) {
        return Err(invalid("资源类型无效"));
    }
    Ok(Json(sqlx::query_as::<_,LinkClass>("SELECT classify,COUNT(*) AS count FROM resource_path WHERE type=? AND status=1 AND classify IS NOT NULL AND classify<>'' GROUP BY classify ORDER BY MAX(id) DESC")
        .bind(kind).fetch_all(&state.pool).await.map_err(db_error)?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LinkStatusInput {
    active: bool,
}
#[derive(Serialize, sqlx::FromRow)]
pub(super) struct AdminLink {
    id: i64,
    title: Option<String>,
    classify: Option<String>,
    cover: Option<String>,
    url: Option<String>,
    introduction: Option<String>,
    link_type: Option<String>,
    status: Option<i64>,
    create_time: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdminLinkQuery {
    kind: Option<String>,
    status: Option<i64>,
    page: Option<i64>,
    size: Option<i64>,
}
pub(super) async fn content_links(
    State(state): State<AppState>,
    Query(query): Query<AdminLinkQuery>,
) -> ApiResult<PageResult<AdminLink>> {
    let (page, size) = AdminPageQuery {
        page: query.page,
        size: query.size,
    }
    .bounds();
    let kind = query.kind.unwrap_or_default();
    if kind.len() > 32 {
        return Err(invalid("资源类型无效"));
    }
    if query.status.is_some_and(|status| !matches!(status, 0 | 1)) {
        return Err(invalid("资源状态无效"));
    }
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM resource_path WHERE (?='' OR type=?) AND (? IS NULL OR status=?)",
    )
    .bind(&kind)
    .bind(&kind)
    .bind(query.status)
    .bind(query.status)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?;
    let items = sqlx::query_as::<_, AdminLink>("SELECT id,title,classify,cover,url,introduction,type AS link_type,status,create_time FROM resource_path WHERE (?='' OR type=?) AND (? IS NULL OR status=?) ORDER BY id DESC LIMIT ? OFFSET ?")
        .bind(&kind).bind(&kind).bind(query.status).bind(query.status).bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LinkInput {
    title: String,
    classify: Option<String>,
    cover: Option<String>,
    url: Option<String>,
    introduction: Option<String>,
    link_type: String,
    status: bool,
}
fn validate_link(input: &LinkInput) -> Result<(), AppError> {
    if input.title.trim().is_empty()
        || input.title.chars().count() > 64
        || !matches!(
            input.link_type.as_str(),
            "friendUrl" | "favorites" | "lovePhoto" | "funny"
        )
        || input
            .classify
            .as_deref()
            .is_some_and(|v| v.chars().count() > 32)
        || input
            .introduction
            .as_deref()
            .is_some_and(|v| v.chars().count() > 1024)
        || input.cover.as_deref().is_some_and(|v| {
            v.len() > 2048
                || !(v.starts_with("https://") || v.starts_with("http://") || v.starts_with('/'))
        })
        || input.url.as_deref().is_some_and(|v| {
            v.len() > 2048 || !(v.starts_with("https://") || v.starts_with("http://"))
        })
    {
        return Err(invalid("资源内容无效"));
    }
    Ok(())
}
async fn load_link(state: &AppState, id: i64) -> ApiResult<AdminLink> {
    Ok(Json(sqlx::query_as("SELECT id,title,classify,cover,url,introduction,type AS link_type,status,create_time FROM resource_path WHERE id=?")
        .bind(id).fetch_optional(&state.pool).await.map_err(db_error)?.ok_or_else(absent)?))
}
pub(super) async fn create_link(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<LinkInput>,
) -> ApiResult<AdminLink> {
    validate_link(&input)?;
    let id = sqlx::query("INSERT INTO resource_path(title,classify,cover,url,introduction,type,status) VALUES(?,?,?,?,?,?,?)")
        .bind(input.title).bind(input.classify).bind(input.cover).bind(input.url).bind(input.introduction).bind(input.link_type).bind(i64::from(input.status))
        .execute(&state.pool).await.map_err(db_error)?.last_insert_rowid();
    load_link(&state, id).await
}
pub(super) async fn update_link(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<LinkInput>,
) -> ApiResult<AdminLink> {
    validate_link(&input)?;
    let changed = sqlx::query("UPDATE resource_path SET title=?,classify=?,cover=?,url=?,introduction=?,type=?,status=? WHERE id=?")
        .bind(input.title).bind(input.classify).bind(input.cover).bind(input.url).bind(input.introduction).bind(input.link_type).bind(i64::from(input.status)).bind(id)
        .execute(&state.pool).await.map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    load_link(&state, id).await
}
pub(super) async fn change_link_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<LinkStatusInput>,
) -> ApiResult<AdminLink> {
    let changed = sqlx::query("UPDATE resource_path SET status=? WHERE id=?")
        .bind(i64::from(input.active))
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    load_link(&state, id).await
}
pub(super) async fn delete_link(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let changed = sqlx::query("DELETE FROM resource_path WHERE id=?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}
