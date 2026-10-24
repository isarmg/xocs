//! Content-addressed publication and reconciliation of the deletion journal.
use axum::{body::Bytes, http::StatusCode};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use super::formats::{MediaKind, is_media_filename};
use crate::{AppError, AppState, db_error, invalid};

#[derive(Serialize)]
pub(super) struct UploadedMedia {
    id: i64,
    path: String,
    mime_type: &'static str,
    size: usize,
}

pub(super) async fn save(
    state: &AppState,
    body: Bytes,
    owner: i64,
    kind: MediaKind,
    original_name: Option<String>,
) -> Result<UploadedMedia, AppError> {
    let limit = kind.max_bytes();
    if body.is_empty() || body.len() > limit {
        return Err(invalid("媒体大小无效"));
    }
    let format = kind
        .detect(&body)
        .ok_or_else(|| invalid(kind.invalid_format_message()))?;
    let extension = format.extension();
    let mime_type = format.mime_type();
    let hash = hex::encode(Sha256::digest(&body));
    let filename = format!("{hash}.{extension}");
    let path = format!("/media/{filename}");
    let directory = state.media.clone();
    let content = body.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let directory = blog_application::fs_safety::PrivateDirectory::open_existing(directory)?;
        let name = blog_application::fs_safety::EntryName::new(filename)?;
        match blog_application::fs_safety::AtomicFile::create(&directory, &name, &content) {
            Ok(()) => Ok(()),
            Err(blog_application::fs_safety::Error::DestinationExists(_)) => {
                let existing = directory.read_bounded(&name, limit)?;
                anyhow::ensure!(
                    existing.as_slice() == content.as_ref(),
                    "existing media content does not match its identity"
                );
                Ok(())
            }
            Err(error) => Err(error.into()),
        }
    })
    .await
    .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "媒体保存失败"))?
    .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "媒体保存失败"))?;
    sqlx::query("INSERT INTO resource(user_id,type,path,size,original_name,mime_type,status,store_type) VALUES(?,?,?,?,?,?,1,'local') ON CONFLICT(path) DO NOTHING")
        .bind(owner).bind(kind.resource_type())
        .bind(&path).bind(body.len() as i64).bind(original_name.as_deref()).bind(mime_type)
        .execute(&state.pool).await.map_err(db_error)?;
    let id: i64 = sqlx::query_scalar("SELECT id FROM resource WHERE path=?")
        .bind(&path)
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    Ok(UploadedMedia {
        id,
        path,
        mime_type,
        size: body.len(),
    })
}

pub(super) fn filesystem_error(error: impl std::fmt::Display) -> AppError {
    tracing::error!(%error, "media filesystem operation failed");
    AppError(StatusCode::INTERNAL_SERVER_ERROR, "媒体文件操作失败")
}

pub(super) fn not_found(error: &blog_application::fs_safety::Error) -> bool {
    matches!(error, blog_application::fs_safety::Error::Io(inner) if inner.kind() == std::io::ErrorKind::NotFound)
}

pub(super) const DELETE_PREFIX: &str = ".blog-delete-";

pub(super) async fn reconcile_deletion(
    pool: &SqlitePool,
    media: &std::path::Path,
    filename: &str,
) -> Result<(), AppError> {
    let path = format!("/media/{filename}");
    let retained: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM resource WHERE path=?)")
        .bind(path)
        .fetch_one(pool)
        .await
        .map_err(db_error)?;
    let media = media.to_path_buf();
    let filename = filename.to_owned();
    tokio::task::spawn_blocking(move || -> Result<(), blog_application::fs_safety::Error> {
        use blog_application::fs_safety::{EntryName, NoClobberPublish, PrivateDirectory};
        let directory = PrivateDirectory::open_existing(media)?;
        let original = EntryName::new(&filename)?;
        let quarantine = EntryName::new(format!("{DELETE_PREFIX}{filename}"))?;
        if retained {
            match NoClobberPublish::publish(
                &directory,
                &quarantine.as_relative(),
                &original.as_relative(),
            ) {
                Ok(()) => Ok(()),
                Err(error) if not_found(&error) => Ok(()),
                Err(blog_application::fs_safety::Error::DestinationExists(_)) => {
                    let max_bytes = MediaKind::Image
                        .max_bytes()
                        .max(MediaKind::Video.max_bytes());
                    let pending = directory.read_bounded(&quarantine, max_bytes)?;
                    let published = directory.read_bounded(&original, max_bytes)?;
                    if pending != published {
                        return Err(blog_application::fs_safety::Error::IdentityChanged(
                            original.as_path().to_path_buf(),
                        ));
                    }
                    directory.remove_file(&quarantine)
                }
                Err(error) => Err(error),
            }
        } else {
            match directory.remove_file(&quarantine) {
                Err(error) if not_found(&error) => Ok(()),
                result => result,
            }
        }
    })
    .await
    .map_err(filesystem_error)?
    .map_err(filesystem_error)
}

pub(crate) async fn recover_deletions(
    pool: &SqlitePool,
    media: &std::path::Path,
) -> anyhow::Result<()> {
    let directory = blog_application::fs_safety::PrivateDirectory::open_existing(media)?;
    for entry in directory.files(blog_application::fs_safety::InventoryLimits {
        max_entries: 100_000,
        max_total_bytes: 1_000_000_000_000,
    })? {
        let Some(name) = entry
            .name
            .as_os_str()
            .to_str()
            .and_then(|name| name.strip_prefix(DELETE_PREFIX))
        else {
            continue;
        };
        anyhow::ensure!(
            is_media_filename(name),
            "invalid media deletion journal name"
        );
        reconcile_deletion(pool, media, name)
            .await
            .map_err(|error| anyhow::anyhow!("{}", error.1))?;
    }
    Ok(())
}
