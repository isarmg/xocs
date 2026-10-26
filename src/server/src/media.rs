//! Media HTTP routes and the write/deletion transaction coordinators.
use blog_application::admin_core::AuthenticatedIdentity;
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
    formats::{MediaFilename, MediaKind},
    references::referenced,
    storage::{UploadedMedia, not_found, reconcile_deletion, recover_failed_deletion},
};
use crate::{ApiResult, AppError, AppState, db_error};

struct MediaUpload {
    body: Bytes,
    filename: MediaFilename,
    original_name: Option<String>,
}

impl MediaUpload {
    fn parse(kind: MediaKind, headers: &HeaderMap, body: Bytes) -> Result<Self, AppError> {
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
        if body.is_empty() || body.len() > kind.max_bytes() {
            return Err(crate::invalid("媒体大小无效"));
        }
        let format = kind
            .detect(&body)
            .ok_or_else(|| crate::invalid(kind.invalid_format_message()))?;
        use sha2::{Digest, Sha256};
        let filename = MediaFilename::from_digest(Sha256::digest(&body).into(), format);
        Ok(Self {
            body,
            filename,
            original_name,
        })
    }
}

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
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<UploadedMedia> {
    save_media(&state, &headers, body, identity, MediaKind::Image).await
}

async fn upload_video(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<UploadedMedia> {
    save_media(&state, &headers, body, identity, MediaKind::Video).await
}

async fn save_media(
    state: &AppState,
    headers: &HeaderMap,
    body: Bytes,
    identity: AuthenticatedIdentity,
    kind: MediaKind,
) -> ApiResult<UploadedMedia> {
    let upload = MediaUpload::parse(kind, headers, body)?;
    let owned = state.clone();
    let result = state
        .scope
        .commit_tasks
        .try_spawn(async move {
            let _operation = owned.media_operations.lock().await;
            let mut transaction = crate::content::begin_mutation(&owned.pool, &identity).await?;
            let owner = crate::site_author_id(&mut *transaction).await?;
            let result = storage::save(&owned, &mut transaction, upload, owner).await?;
            transaction.commit().await.map_err(db_error)?;
            Ok::<_, AppError>(result)
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
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    ContractPath(id): ContractPath<i64>,
    crate::ContractJson(input): crate::ContractJson<DeleteResourceInput>,
) -> ApiResult<serde_json::Value> {
    let owned = state.clone();
    state
        .scope
        .commit_tasks
        .try_spawn(async move {
            let _operation = owned.media_operations.lock().await;
            let mut transaction = crate::content::begin_mutation(&owned.pool, &identity).await?;
            let row = sqlx::query("SELECT path,type,store_type FROM resource WHERE id=?")
                .bind(id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(db_error)?
                .ok_or_else(crate::absent)?;
            let path: Option<String> = row.try_get("path").map_err(db_error)?;
            let kind: Option<String> = row.try_get("type").map_err(db_error)?;
            let storage: Option<String> = row.try_get("store_type").map_err(db_error)?;
            let filename = path
                .as_deref()
                .and_then(|path| path.strip_prefix("/media/").and_then(MediaFilename::parse))
                .filter(|filename| {
                    kind.as_deref() == Some(filename.format().kind().resource_type())
                })
                .filter(|_| storage.as_deref() == Some("local"))
                .ok_or(AppError::Public(
                    HttpStatus::Conflict,
                    "仅可删除本站上传的图片或视频",
                ))?;
            let path = filename.path();
            if path != input.path {
                return Err(AppError::Public(
                    HttpStatus::Conflict,
                    "文件记录已经改变，请刷新后重试",
                ));
            }
            if referenced(&mut transaction, &filename).await? {
                return Err(AppError::Public(
                    HttpStatus::Conflict,
                    "媒体仍被站内内容引用，请先移除引用",
                ));
            }
            let media = owned.media.clone();
            let name = filename.clone();
            let (directory, moved) = tokio::task::spawn_blocking(move || {
                use blog_application::fs_safety::{NoClobberPublish, PrivateDirectory};
                let directory = std::sync::Arc::new(PrivateDirectory::open_existing(media)?);
                let moved = NoClobberPublish::publish(
                    &directory,
                    &name.entry_name().as_relative(),
                    &name.deletion_name().as_relative(),
                );
                Ok::<_, blog_application::fs_safety::Error>((directory, moved))
            })
            .await
            .map_err(|source| AppError::Task {
                message: "媒体删除失败",
                source,
            })??;
            let deletion = async {
                match moved {
                    Ok(()) => {}
                    // A row whose upload has already disappeared can still be removed.
                    Err(error) if not_found(&error) => {}
                    Err(error) => return Err(error.into()),
                }
                sqlx::query("DELETE FROM resource WHERE id=?")
                    .bind(id)
                    .execute(&mut *transaction)
                    .await
                    .map_err(db_error)?;
                Ok::<_, AppError>(())
            }
            .await;
            if let Err(operation) = deletion {
                if let Err(rollback) = transaction.rollback().await {
                    return Err(AppError::DeletionRollback {
                        operation: Box::new(operation),
                        rollback,
                    });
                }
                return Err(
                    recover_failed_deletion(&owned.pool, &directory, &filename, operation).await,
                );
            }
            if let Err(error) = transaction.commit().await {
                return Err(recover_failed_deletion(
                    &owned.pool,
                    &directory,
                    &filename,
                    db_error(error),
                )
                .await);
            }
            reconcile_deletion(&owned.pool, &directory, &filename).await?;
            Ok(Json(serde_json::json!({"deleted":true})))
        })?
        .await
        .map_err(|source| AppError::Task {
            message: "媒体删除失败",
            source,
        })?
}
