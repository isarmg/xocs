use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{ApiResult, AppError, AppState, db_error, invalid};

const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

pub fn admin_routes() -> Router<AppState> {
    Router::new().route(
        "/api/v1/content/upload",
        post(upload).layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES)),
    )
}

#[derive(Serialize)]
struct UploadedImage {
    id: i64,
    path: String,
    mime_type: &'static str,
    size: usize,
}

fn image_type(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(("png", "image/png"));
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some(("jpg", "image/jpeg"));
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(("gif", "image/gif"));
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(("webp", "image/webp"));
    }
    None
}

async fn upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<UploadedImage> {
    let author = crate::site_author_id(&state.pool).await?;
    save_image(&state, &headers, body, author, MAX_IMAGE_BYTES).await
}

async fn save_image(
    state: &AppState,
    headers: &HeaderMap,
    body: Bytes,
    owner: i64,
    limit: usize,
) -> ApiResult<UploadedImage> {
    if body.is_empty() || body.len() > limit {
        return Err(invalid("图片大小无效"));
    }
    let (extension, mime_type) =
        image_type(&body).ok_or_else(|| invalid("仅支持 PNG、JPEG、GIF 和 WebP 图片"))?;
    let original_name = headers
        .get("x-file-name")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| percent_encoding::percent_decode_str(v).decode_utf8().ok())
        .filter(|v| v.len() <= 200 && !v.contains('/') && !v.contains('\\'));
    let hash = hex::encode(Sha256::digest(&body));
    let filename = format!("{hash}.{extension}");
    let path = format!("/media/{filename}");
    let directory = state.media.clone();
    let content = body.clone();
    state
        .scope
        .commit_tasks
        .try_spawn(async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                let directory = xcss::fs_safety::PrivateDirectory::open_existing(directory)?;
                let name = xcss::fs_safety::EntryName::new(filename)?;
                match xcss::fs_safety::AtomicFile::create(&directory, &name, &content) {
                    Ok(()) => Ok(()),
                    Err(xcss::fs_safety::Error::DestinationExists(_)) => {
                        let existing = directory.read_bounded(&name, limit)?;
                        anyhow::ensure!(
                            existing.as_slice() == content.as_ref(),
                            "existing image content does not match its identity"
                        );
                        Ok(())
                    }
                    Err(error) => Err(error.into()),
                }
            })
            .await
        })
        .map_err(|_| AppError(StatusCode::SERVICE_UNAVAILABLE, "服务正在停止"))?
        .await
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "图片保存失败"))?
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "图片保存失败"))?
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "图片保存失败"))?;
    sqlx::query("INSERT INTO resource(user_id,type,path,size,original_name,mime_type,status,store_type) VALUES(?,'image',?,?,?,?,1,'local') ON CONFLICT(path) DO NOTHING")
        .bind(owner)
        .bind(&path).bind(body.len() as i64).bind(original_name.as_deref()).bind(mime_type)
        .execute(&state.pool).await.map_err(db_error)?;
    let id: i64 = sqlx::query_scalar("SELECT id FROM resource WHERE path=?")
        .bind(&path)
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    Ok(Json(UploadedImage {
        id,
        path,
        mime_type,
        size: body.len(),
    }))
}
