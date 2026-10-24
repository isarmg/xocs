mod media;

use crate::{ApiResult, AppError, AppState, db_error, invalid};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct SiteInfo {
    web_name: Option<String>,
    web_title: Option<String>,
    notices: Option<String>,
    background_image: Option<String>,
    avatar: Option<String>,
    random_cover: Option<String>,
    media: sqlx::types::Json<media::SiteMedia>,
}
pub(super) async fn site_info(State(state): State<AppState>) -> ApiResult<SiteInfo> {
    Ok(Json(sqlx::query_as::<_,SiteInfo>("SELECT web_name,web_title,notices,background_image,avatar,random_cover,media FROM web_info ORDER BY id LIMIT 1")
        .fetch_optional(&state.pool).await.map_err(db_error)?.unwrap_or(SiteInfo {web_name:Some("blog".into()),web_title:Some("相信记录的力量".into()),notices:None,background_image:None,avatar:None,random_cover:None,media:Default::default()})))
}

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct AdminSite {
    author_name: String,
    web_name: Option<String>,
    web_title: Option<String>,
    notices: Option<String>,
    background_image: Option<String>,
    avatar: Option<String>,
    random_cover: Option<String>,
    media: sqlx::types::Json<media::SiteMedia>,
}
pub(super) async fn admin_site(State(state): State<AppState>) -> ApiResult<AdminSite> {
    let author = crate::site_author_id(&state.pool).await?;
    let author_name: Option<String> = sqlx::query_scalar("SELECT username FROM user WHERE id=?")
        .bind(author)
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    Ok(Json(sqlx::query_as::<_,AdminSite>("SELECT ? AS author_name,web_name,web_title,notices,background_image,avatar,random_cover,media FROM web_info ORDER BY id LIMIT 1")
        .bind(author_name.as_deref().unwrap_or_default())
        .fetch_optional(&state.pool).await.map_err(db_error)?.unwrap_or(AdminSite{author_name:author_name.unwrap_or_default(),web_name:None,web_title:None,notices:None,background_image:None,avatar:None,random_cover:None,media:Default::default()})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SiteInput {
    author_name: String,
    web_name: String,
    web_title: Option<String>,
    notices: Option<String>,
    background_image: Option<String>,
    avatar: Option<String>,
    random_cover: Option<String>,
    media: media::SiteMedia,
}
pub(super) async fn update_site(
    State(state): State<AppState>,
    crate::ContractJson(mut input): crate::ContractJson<SiteInput>,
) -> ApiResult<AdminSite> {
    input.media.normalize()?;
    media::normalize_url(&mut input.background_image)?;
    media::normalize_url(&mut input.avatar)?;
    media::normalize_covers(&mut input.random_cover)?;
    let author_name = input.author_name.trim();
    if author_name.is_empty()
        || author_name.chars().count() > 100
        || input.web_name.trim().is_empty()
        || input.web_name.chars().count() > 100
        || input
            .web_title
            .as_deref()
            .is_some_and(|v| v.chars().count() > 200)
        || [&input.notices, &input.random_cover]
            .iter()
            .any(|item| item.as_deref().is_some_and(|v| v.len() > 16384))
    {
        return Err(invalid("站点信息无效"));
    }
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let author: i64 = sqlx::query_scalar(
        "SELECT id FROM user WHERE user_type=0 AND deleted=0 AND user_status=1 ORDER BY id LIMIT 1",
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(db_error)?
    .ok_or(AppError(StatusCode::CONFLICT, "缺少站点作者账号"))?;
    let name_used: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user WHERE username=? AND id<>?)")
            .bind(author_name)
            .bind(author)
            .fetch_one(&mut *transaction)
            .await
            .map_err(db_error)?;
    if name_used {
        return Err(AppError(StatusCode::CONFLICT, "作者展示名已被使用"));
    }
    sqlx::query("UPDATE user SET username=?,update_time=CURRENT_TIMESTAMP WHERE id=?")
        .bind(author_name)
        .bind(author)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    let id: Option<i64> = sqlx::query_scalar("SELECT id FROM web_info ORDER BY id LIMIT 1")
        .fetch_optional(&mut *transaction)
        .await
        .map_err(db_error)?;
    if let Some(id) = id {
        sqlx::query("UPDATE web_info SET web_name=?,web_title=?,notices=?,background_image=?,avatar=?,random_cover=?,media=? WHERE id=?")
            .bind(input.web_name).bind(input.web_title).bind(input.notices).bind(input.background_image).bind(input.avatar).bind(input.random_cover).bind(sqlx::types::Json(input.media)).bind(id)
            .execute(&mut *transaction).await.map_err(db_error)?;
    } else {
        sqlx::query("INSERT INTO web_info(web_name,web_title,notices,background_image,avatar,random_cover,media,status) VALUES(?,?,?,?,?,?,?,1)")
            .bind(input.web_name).bind(input.web_title).bind(input.notices).bind(input.background_image).bind(input.avatar).bind(input.random_cover).bind(sqlx::types::Json(input.media))
            .execute(&mut *transaction).await.map_err(db_error)?;
    }
    transaction.commit().await.map_err(db_error)?;
    admin_site(State(state)).await
}
