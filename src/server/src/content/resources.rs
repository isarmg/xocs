use super::{AdminPageQuery, PageResult};
use crate::{ApiResult, AppState};
use axum::{Json, extract::State};
use blog_application::server_cli::ContractQuery as Query;
use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Resource {
    id: i64,
    original_name: Option<String>,
    path: Option<String>,
    resource_type: Option<String>,
    size: Option<i64>,
    status: Option<i64>,
    create_time: Option<String>,
}
pub(super) async fn resources(
    State(state): State<AppState>,
    Query(query): Query<AdminPageQuery>,
) -> ApiResult<PageResult<Resource>> {
    let (page, size) = query.bounds();
    let mut transaction = state.pool.begin().await?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource")
        .fetch_one(&mut *transaction)
        .await?;
    let items=sqlx::query_as::<_,Resource>("SELECT id,original_name,path,type AS resource_type,size,status,create_time FROM resource ORDER BY id DESC LIMIT ? OFFSET ?")
        .bind(size).bind((page-1)*size).fetch_all(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
