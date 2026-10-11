use super::PageResult;
use crate::{ApiResult, AppError, AppState, db_error};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use xcss::server_cli::ContractQuery as Query;

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Note {
    id: i64,
    user_id: Option<i64>,
    username: Option<String>,
    content: String,
    image_path: Option<String>,
    like_count: i64,
    is_public: i64,
    create_time: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NotePageQuery {
    page: Option<i64>,
    size: Option<i64>,
}
async fn list_note_page(
    state: &AppState,
    query: NotePageQuery,
) -> Result<PageResult<Note>, AppError> {
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(10).clamp(1, 50);
    let author = crate::site_author_id(&state.pool).await?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM wei_yan WHERE type='friend' AND user_id=? AND is_public=1",
    )
    .bind(author)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?;
    let items=sqlx::query_as::<_,Note>("SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.like_count,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id WHERE w.type='friend' AND w.user_id=? AND w.is_public=1 ORDER BY w.id DESC LIMIT ? OFFSET ?")
        .bind(author).bind(size).bind((page-1)*size)
        .fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(PageResult {
        items,
        total,
        page,
        size,
    })
}
pub(super) async fn note_page(
    State(state): State<AppState>,
    Query(query): Query<NotePageQuery>,
) -> ApiResult<PageResult<Note>> {
    Ok(Json(list_note_page(&state, query).await?))
}
