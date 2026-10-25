//! Media HTTP routes and the write/deletion transaction coordinators.
use blog_application::error::HttpStatus;
mod formats;
mod references;
mod storage;

pub(crate) use formats::is_media_filename;
pub(crate) use storage::recover_deletions;

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::HeaderMap,
    routing::post,
};
use blog_application::server_cli::ContractPath;
use serde::Deserialize;
use sqlx::Row;

use self::{
    formats::{MediaFormat, MediaKind},
    references::referenced,
    storage::{DELETE_PREFIX, UploadedMedia, not_found, reconcile_deletion},
};
use crate::{ApiResult, AppError, AppState, db_error};

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/content/upload",
            post(upload).layer(DefaultBodyLimit::max(MediaKind::Image.max_bytes())),
        )
        .route(
            "/api/v1/content/upload-video",
            post(upload_video).layer(DefaultBodyLimit::max(MediaKind::Video.max_bytes())),
        )
        .route(
            "/api/v1/content/resources/{id}",
            axum::routing::delete(delete_resource),
        )
}

async fn upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<UploadedMedia> {
    let author = crate::site_author_id(&state.pool).await?;
    save_media(&state, &headers, body, author, MediaKind::Image).await
}

async fn upload_video(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<UploadedMedia> {
    let author = crate::site_author_id(&state.pool).await?;
    save_media(&state, &headers, body, author, MediaKind::Video).await
}

async fn save_media(
    state: &AppState,
    headers: &HeaderMap,
    body: Bytes,
    owner: i64,
    kind: MediaKind,
) -> ApiResult<UploadedMedia> {
    let owned = state.clone();
    let original_name = headers
        .get("x-file-name")
        .map(|value| {
            let value = value.to_str().map_err(AppError::FileNameHeader)?;
            let value = url_encoding::percent::percent_decode_str(value)
                .decode_utf8()
                .map_err(AppError::FileNameEncoding)?;
            if value.len() > 200 || value.contains('/') || value.contains('\\') {
                return Err(crate::invalid("文件名无效"));
            }
            Ok(value.into_owned())
        })
        .transpose()?;
    let result = state
        .scope
        .commit_tasks
        .try_spawn(async move {
            let _operation = owned.media_operations.lock().await;
            storage::save(&owned, body, owner, kind, original_name).await
        })?
        .await
        .map_err(|source| AppError::Task {
            message: "媒体保存失败",
            source,
        })?;
    result.map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteResourceInput {
    path: String,
}

async fn delete_resource(
    State(state): State<AppState>,
    ContractPath(id): ContractPath<i64>,
    crate::ContractJson(input): crate::ContractJson<DeleteResourceInput>,
) -> ApiResult<serde_json::Value> {
    let owned = state.clone();
    state
        .scope
        .commit_tasks
        .try_spawn(async move {
            let _operation = owned.media_operations.lock().await;
            let mut transaction = owned
                .pool
                .begin_with("BEGIN IMMEDIATE")
                .await
                .map_err(db_error)?;
            let row = sqlx::query("SELECT path,type,store_type FROM resource WHERE id=?")
                .bind(id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(db_error)?
                .ok_or_else(crate::absent)?;
            let path: Option<String> = row.try_get("path").map_err(db_error)?;
            let kind: Option<String> = row.try_get("type").map_err(db_error)?;
            let storage: Option<String> = row.try_get("store_type").map_err(db_error)?;
            let path = path
                .filter(|path| {
                    path.strip_prefix("/media/")
                        .and_then(MediaFormat::from_filename)
                        .is_some_and(|format| {
                            kind.as_deref() == Some(format.kind().resource_type())
                        })
                })
                .filter(|_| storage.as_deref() == Some("local"))
                .ok_or(AppError::Public(
                    HttpStatus::Conflict,
                    "仅可删除本站上传的图片或视频",
                ))?;
            if path != input.path {
                return Err(AppError::Public(
                    HttpStatus::Conflict,
                    "文件记录已经改变，请刷新后重试",
                ));
            }
            if referenced(&mut transaction, &path).await? {
                return Err(AppError::Public(
                    HttpStatus::Conflict,
                    "媒体仍被站内内容引用，请先移除引用",
                ));
            }
            let filename = path
                .strip_prefix("/media/")
                .expect("validated media path")
                .to_owned();
            let media = owned.media.clone();
            let name = filename.clone();
            let moved = tokio::task::spawn_blocking(move || {
                use blog_application::fs_safety::{EntryName, NoClobberPublish, PrivateDirectory};
                let directory = PrivateDirectory::open_existing(media)?;
                NoClobberPublish::publish(
                    &directory,
                    &EntryName::new(&name)?.as_relative(),
                    &EntryName::new(format!("{DELETE_PREFIX}{name}"))?.as_relative(),
                )
            })
            .await
            .map_err(|source| AppError::Task {
                message: "媒体删除失败",
                source,
            })?;
            if let Err(error) = moved
                && !not_found(&error)
            {
                transaction.rollback().await.map_err(db_error)?;
                reconcile_deletion(&owned.pool, &owned.media, &filename).await?;
                return Err(error.into());
            }
            let deletion = sqlx::query("DELETE FROM resource WHERE id=?")
                .bind(id)
                .execute(&mut *transaction)
                .await;
            if let Err(error) = deletion {
                transaction.rollback().await.map_err(db_error)?;
                reconcile_deletion(&owned.pool, &owned.media, &filename).await?;
                return Err(db_error(error));
            }
            if let Err(error) = transaction.commit().await {
                reconcile_deletion(&owned.pool, &owned.media, &filename).await?;
                return Err(db_error(error));
            }
            reconcile_deletion(&owned.pool, &owned.media, &filename).await?;
            Ok(Json(serde_json::json!({"deleted":true})))
        })?
        .await
        .map_err(|source| AppError::Task {
            message: "媒体删除失败",
            source,
        })?
}
