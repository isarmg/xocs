use crate::{ApiResult, AppState, db_error};
use axum::{Json, extract::State};
use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct SiteInfo {
    web_name: Option<String>,
    web_title: Option<String>,
    notices: Option<String>,
    background_image: Option<String>,
    avatar: Option<String>,
    random_cover: Option<String>,
}
pub(super) async fn site_info(State(state): State<AppState>) -> ApiResult<SiteInfo> {
    Ok(Json(sqlx::query_as::<_,SiteInfo>("SELECT web_name,web_title,notices,background_image,avatar,random_cover FROM web_info ORDER BY id LIMIT 1")
        .fetch_optional(&state.pool).await.map_err(db_error)?.unwrap_or(SiteInfo {web_name:Some("xocs".into()),web_title:Some("相信记录的力量".into()),notices:None,background_image:None,avatar:None,random_cover:None})))
}
