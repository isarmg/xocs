//! Content-addressed publication and reconciliation of the deletion journal.
use blog_application::fs_safety::PrivateDirectory;
use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::Arc;

use super::{
    MediaUpload,
    formats::{MediaFilename, MediaKind},
};
use crate::{AppError, AppState, db_error};

#[derive(Serialize)]
pub(super) struct UploadedMedia {
    id: i64,
    path: String,
    mime_type: &'static str,
    size: usize,
}

pub(super) async fn save(
    state: &AppState,
    connection: &mut sqlx::SqliteConnection,
    upload: MediaUpload,
    owner: i64,
) -> Result<UploadedMedia, AppError> {
    let MediaUpload {
        body,
        filename,
        original_name,
    } = upload;
    let format = filename.format();
    let kind = format.kind();
    let limit = kind.max_bytes();
    let mime_type = format.mime_type();
    let path = filename.path();
    let directory = state.media.clone();
    let size = body.len();
    tokio::task::spawn_blocking(move || -> Result<(), blog_application::fs_safety::Error> {
        let directory = blog_application::fs_safety::PrivateDirectory::open_existing(directory)?;
        let name = filename.entry_name();
        match blog_application::fs_safety::AtomicFile::create(&directory, name, &body) {
            Ok(()) => Ok(()),
            Err(blog_application::fs_safety::Error::DestinationExists(_)) => {
                let existing = directory.read_bounded(name, limit)?;
                if existing.as_slice() != body.as_ref() {
                    return Err(blog_application::fs_safety::Error::IdentityChanged(
                        name.as_path().to_path_buf(),
                    ));
                }
                Ok(())
            }
            Err(error) => Err(error),
        }
    })
    .await
    .map_err(|source| AppError::Task {
        message: "媒体保存失败",
        source,
    })??;
    sqlx::query("INSERT INTO resource(user_id,type,path,size,original_name,mime_type,status,store_type) VALUES(?,?,?,?,?,?,1,'local') ON CONFLICT(path) DO NOTHING")
        .bind(owner).bind(kind.resource_type())
        .bind(&path).bind(size as i64).bind(original_name.as_deref()).bind(mime_type)
        .execute(&mut *connection).await.map_err(db_error)?;
    let id: i64 = sqlx::query_scalar("SELECT id FROM resource WHERE path=?")
        .bind(&path)
        .fetch_one(connection)
        .await
        .map_err(db_error)?;
    Ok(UploadedMedia {
        id,
        path,
        mime_type,
        size,
    })
}

pub(super) fn not_found(error: &blog_application::fs_safety::Error) -> bool {
    matches!(error, blog_application::fs_safety::Error::Io(inner) if inner.kind() == std::io::ErrorKind::NotFound)
}

pub(super) const DELETE_PREFIX: &str = ".blog-delete-";

pub(super) async fn reconcile_deletion(
    pool: &SqlitePool,
    directory: &Arc<PrivateDirectory>,
    filename: &MediaFilename,
) -> Result<(), AppError> {
    let path = filename.path();
    let retained: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM resource WHERE path=?)")
        .bind(path)
        .fetch_one(pool)
        .await
        .map_err(db_error)?;
    let directory = Arc::clone(directory);
    let filename = filename.clone();
    tokio::task::spawn_blocking(move || -> Result<(), blog_application::fs_safety::Error> {
        use blog_application::fs_safety::NoClobberPublish;
        let original = filename.entry_name();
        let quarantine = filename.deletion_name();
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
                    let published = directory.read_bounded(original, max_bytes)?;
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
    .map_err(|source| AppError::Task {
        message: "媒体文件操作失败",
        source,
    })?
    .map_err(AppError::from)
}

pub(super) async fn recover_failed_deletion(
    pool: &SqlitePool,
    directory: &Arc<PrivateDirectory>,
    filename: &MediaFilename,
    operation: AppError,
) -> AppError {
    match reconcile_deletion(pool, directory, filename).await {
        Ok(()) => operation,
        Err(recovery) => AppError::DeletionRecovery {
            operation: Box::new(operation),
            recovery: Box::new(recovery),
        },
    }
}

pub(crate) async fn recover_deletions(
    pool: &SqlitePool,
    media: &std::path::Path,
) -> Result<(), AppError> {
    let directory = Arc::new(PrivateDirectory::open_existing(media)?);
    for entry in directory.files(blog_application::fs_safety::InventoryLimits {
        max_entries: 100_000,
        max_total_bytes: 1_000_000_000_000,
    })? {
        let Some(name) = entry
            .name
            .as_os_str()
            .as_encoded_bytes()
            .strip_prefix(DELETE_PREFIX.as_bytes())
        else {
            continue;
        };
        let name = std::str::from_utf8(name).map_err(AppError::DeletionJournalEncoding)?;
        let filename = MediaFilename::parse(name).ok_or(AppError::InvalidDeletionJournal)?;
        reconcile_deletion(pool, &directory, &filename).await?;
    }
    Ok(())
}
