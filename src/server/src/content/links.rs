use super::PageResult;
use crate::{ApiResult, AppState, db_error, invalid};
use axum::{Json, extract::State};
use blog_application::server_cli::ContractQuery as Query;
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
