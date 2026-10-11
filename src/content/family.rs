use crate::{ApiResult, AppState, db_error};
use axum::{Json, extract::State};
use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Family {
    id: i64,
    bg_cover: Option<String>,
    man_cover: Option<String>,
    woman_cover: Option<String>,
    man_name: Option<String>,
    woman_name: Option<String>,
    timing: Option<String>,
    countdown_title: Option<String>,
    countdown_time: Option<String>,
    family_info: Option<String>,
}
pub(super) async fn family(State(state): State<AppState>) -> ApiResult<Vec<Family>> {
    let author = crate::site_author_id(&state.pool).await?;
    // Choose the owner's current home before checking visibility, so hiding it
    // cannot reveal an older story from a previous version of the site.
    Ok(Json(sqlx::query_as::<_,Family>("SELECT id,bg_cover,man_cover,woman_cover,man_name,woman_name,timing,countdown_title,countdown_time,family_info FROM family WHERE user_id=? AND id=(SELECT MAX(id) FROM family WHERE user_id=?) AND status=1")
        .bind(author).bind(author).fetch_all(&state.pool).await.map_err(db_error)?))
}
