use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    routing::post,
};
use blog_application::server_cli::ContractPath;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqliteConnection, SqlitePool};

use crate::{ApiResult, AppError, AppState, db_error, invalid};

const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/content/upload",
            post(upload).layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES)),
        )
        .route(
            "/api/v1/content/resources/{id}",
            axum::routing::delete(delete_resource),
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
    let owned = state.clone();
    let headers = headers.clone();
    let result = state
        .scope
        .commit_tasks
        .try_spawn(async move {
            let _operation = owned.media_operations.lock().await;
            save_image_inner(&owned, &headers, body, owner, limit).await
        })
        .map_err(|_| AppError(StatusCode::SERVICE_UNAVAILABLE, "服务正在停止"))?
        .await
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "图片保存失败"))?;
    result.map(Json)
}

async fn save_image_inner(
    state: &AppState,
    headers: &HeaderMap,
    body: Bytes,
    owner: i64,
    limit: usize,
) -> Result<UploadedImage, AppError> {
    if body.is_empty() || body.len() > limit {
        return Err(invalid("图片大小无效"));
    }
    let (extension, mime_type) =
        image_type(&body).ok_or_else(|| invalid("仅支持 PNG、JPEG、GIF 和 WebP 图片"))?;
    let original_name = headers
        .get("x-file-name")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            url_encoding::percent::percent_decode_str(v)
                .decode_utf8()
                .ok()
        })
        .filter(|v| v.len() <= 200 && !v.contains('/') && !v.contains('\\'));
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
                    "existing image content does not match its identity"
                );
                Ok(())
            }
            Err(error) => Err(error.into()),
        }
    })
    .await
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
    Ok(UploadedImage {
        id,
        path,
        mime_type,
        size: body.len(),
    })
}

pub(crate) fn is_image_filename(name: &str) -> bool {
    let Some((hash, extension)) = name.split_once('.') else {
        return false;
    };
    hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && matches!(extension, "png" | "jpg" | "gif" | "webp")
}

fn filesystem_error(error: impl std::fmt::Display) -> AppError {
    tracing::error!(%error, "media filesystem operation failed");
    AppError(StatusCode::INTERNAL_SERVER_ERROR, "图片文件操作失败")
}

fn not_found(error: &blog_application::fs_safety::Error) -> bool {
    matches!(error, blog_application::fs_safety::Error::Io(inner) if inner.kind() == std::io::ErrorKind::NotFound)
}

fn decoded_reference(value: &str) -> String {
    let mut decoded = url_encoding::percent::percent_decode_str(value)
        .decode_utf8_lossy()
        .into_owned();
    // HTML attributes may use numeric entities; Markdown and ordinary URL
    // fields may percent-encode the same image path. Compare their decoded form.
    for _ in 0..3 {
        let mut output = String::with_capacity(decoded.len());
        let mut rest = decoded.as_str();
        while let Some(start) = rest.find('&') {
            output.push_str(&rest[..start]);
            let candidate = &rest[start..];
            let Some(end) = candidate.find(';').filter(|end| *end <= 16) else {
                output.push('&');
                rest = &candidate[1..];
                continue;
            };
            let entity = &candidate[1..end];
            let character = match entity {
                "sol" => Some('/'),
                "period" => Some('.'),
                "amp" => Some('&'),
                _ => entity
                    .strip_prefix("#x")
                    .or_else(|| entity.strip_prefix("#X"))
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .and_then(char::from_u32)
                    .or_else(|| {
                        entity
                            .strip_prefix('#')
                            .and_then(|number| number.parse().ok())
                            .and_then(char::from_u32)
                    }),
            };
            if let Some(character) = character {
                output.push(character);
                rest = &candidate[end + 1..];
            } else {
                output.push('&');
                rest = &candidate[1..];
            }
        }
        output.push_str(rest);
        output = url_encoding::percent::percent_decode_str(&output)
            .decode_utf8_lossy()
            .into_owned();
        if output == decoded {
            break;
        }
        decoded = output;
    }
    decoded
}

fn text_contains_reference(value: &str, path: &str) -> bool {
    if value.contains(path) || decoded_reference(value).contains(path) {
        return true;
    }
    // The browser uses this same tokenizer: backslash escapes depend on the
    // Markdown context and must never be stripped from arbitrary stored text.
    // Its destinations have already decoded CommonMark escapes and entities;
    // only URL percent decoding remains, not another HTML-entity pass.
    value.contains('\\')
        && Parser::new_ext(
            value,
            Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
        )
        .any(|event| match event {
            Event::Start(Tag::Image { dest_url, .. } | Tag::Link { dest_url, .. }) => {
                dest_url.contains(path)
                    || url_encoding::percent::percent_decode_str(&dest_url)
                        .decode_utf8_lossy()
                        .contains(path)
            }
            _ => false,
        })
}

fn contains_reference(value: &str, path: &str) -> bool {
    fn json_contains(value: &serde_json::Value, path: &str) -> bool {
        match value {
            serde_json::Value::String(value) => text_contains_reference(value, path),
            serde_json::Value::Array(values) => {
                values.iter().any(|value| json_contains(value, path))
            }
            serde_json::Value::Object(values) => {
                values.values().any(|value| json_contains(value, path))
            }
            _ => false,
        }
    }
    value.split('\0').any(|field| {
        // Each SQL column is a separate document. Parsing their NUL-separated
        // concatenation changes CommonMark reference-definition boundaries.
        text_contains_reference(field, path)
            || serde_json::from_str::<serde_json::Value>(field)
                .is_ok_and(|value| json_contains(&value, path))
    })
}

async fn referenced(connection: &mut SqliteConnection, path: &str) -> Result<bool, AppError> {
    // Inspect all current content, including drafts, so removing an unused
    // upload cannot break a saved article, profile, album, note or letter.
    const QUERIES: &[&str] = &[
        "SELECT id,coalesce(article_cover,'')||char(0)||coalesce(video_url,'')||char(0)||coalesce(article_content,'')||char(0)||coalesce(tips,'') AS value FROM article NOT INDEXED WHERE deleted=0",
        "SELECT id,coalesce(background_image,'')||char(0)||coalesce(avatar,'')||char(0)||coalesce(random_cover,'')||char(0)||coalesce(notices,'') AS value FROM web_info",
        "SELECT id,coalesce(bg_cover,'')||char(0)||coalesce(man_cover,'')||char(0)||coalesce(woman_cover,'')||char(0)||coalesce(family_info,'') AS value FROM family",
        "SELECT id,coalesce(cover,'')||char(0)||coalesce(url,'')||char(0)||coalesce(introduction,'')||char(0)||coalesce(remark,'') AS value FROM resource_path",
        "SELECT id,coalesce(image_path,'')||char(0)||coalesce(content,'') AS value FROM wei_yan",
        "SELECT id,coalesce(image_path,'')||char(0)||coalesce(avatar,'')||char(0)||coalesce(message,'') AS value FROM tree_hole",
        "SELECT id,coalesce(avatar,'') AS value FROM user WHERE deleted=0",
        "SELECT id,coalesce(comment_content,'')||char(0)||coalesce(comment_info,'') AS value FROM comment",
    ];
    for query in QUERIES {
        // SQL filters the usual unencoded case without materializing every
        // article. Encoded references are checked in bounded individual rows.
        // The article feed indexes order by date, so using them here would sort
        // the remaining corpus on every batch. Walk its rowid range instead.
        let mut after = i64::MIN;
        loop {
            let rows = sqlx::query(sqlx::AssertSqlSafe(format!("SELECT id,value FROM ({query}) WHERE id>? AND (instr(value,?)>0 OR instr(value,'%')>0 OR instr(value,'&')>0 OR instr(value,char(92))>0) ORDER BY id LIMIT 8")))
                .bind(after).bind(path).fetch_all(&mut *connection).await.map_err(db_error)?;
            if rows.is_empty() {
                break;
            }
            for row in rows {
                after = row.try_get("id").map_err(db_error)?;
                let value: String = row.try_get("value").map_err(db_error)?;
                if contains_reference(&value, path) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

const DELETE_PREFIX: &str = ".blog-delete-";

async fn reconcile_deletion(
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
                    let pending = directory.read_bounded(&quarantine, MAX_IMAGE_BYTES)?;
                    let published = directory.read_bounded(&original, MAX_IMAGE_BYTES)?;
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
            is_image_filename(name),
            "invalid media deletion journal name"
        );
        reconcile_deletion(pool, media, name)
            .await
            .map_err(|error| anyhow::anyhow!("{}", error.1))?;
    }
    Ok(())
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
                .filter(|path| path.strip_prefix("/media/").is_some_and(is_image_filename))
                .filter(|_| kind.as_deref() == Some("image") && storage.as_deref() == Some("local"))
                .ok_or(AppError(StatusCode::CONFLICT, "仅可删除本站上传的图片"))?;
            if path != input.path {
                return Err(AppError(
                    StatusCode::CONFLICT,
                    "文件记录已经改变，请刷新后重试",
                ));
            }
            if referenced(&mut transaction, &path).await? {
                return Err(AppError(
                    StatusCode::CONFLICT,
                    "图片仍被站内内容引用，请先移除引用",
                ));
            }
            let filename = path
                .strip_prefix("/media/")
                .expect("validated image path")
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
            .map_err(filesystem_error)?;
            if let Err(error) = moved
                && !not_found(&error)
            {
                transaction.rollback().await.map_err(db_error)?;
                reconcile_deletion(&owned.pool, &owned.media, &filename).await?;
                return Err(filesystem_error(error));
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
        })
        .map_err(|_| AppError(StatusCode::SERVICE_UNAVAILABLE, "服务正在停止"))?
        .await
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "图片删除失败"))?
}

#[path = "../../../test/server/unit/media/reference_tests.rs"]
#[cfg(test)]
mod reference_tests;
