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
const MAX_VIDEO_BYTES: usize = 50 * 1024 * 1024;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/content/upload",
            post(upload).layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES)),
        )
        .route(
            "/api/v1/content/upload-video",
            post(upload_video).layer(DefaultBodyLimit::max(MAX_VIDEO_BYTES)),
        )
        .route(
            "/api/v1/content/resources/{id}",
            axum::routing::delete(delete_resource),
        )
}

#[derive(Serialize)]
struct UploadedMedia {
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

// Parse only bounded container metadata. The browser remains responsible for
// codec support; an image, audio-only container or renamed document is not a
// video upload, regardless of the request MIME type or filename.
fn video_type(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.get(4..8) == Some(b"ftyp") && mp4_video(bytes) {
        Some(("mp4", "video/mp4"))
    } else if bytes.starts_with(b"\x1a\x45\xdf\xa3") && webm_video(bytes) {
        Some(("webm", "video/webm"))
    } else {
        None
    }
}

fn mp4_box<'a>(bytes: &'a [u8], budget: &mut usize) -> Option<(&'a [u8], &'a [u8], &'a [u8])> {
    *budget = budget.checked_sub(1)?;
    let size = u32::from_be_bytes(bytes.get(..4)?.try_into().ok()?);
    let kind = bytes.get(4..8)?;
    let (header, size) = match size {
        0 => (8, bytes.len()),
        1 => (
            16,
            usize::try_from(u64::from_be_bytes(bytes.get(8..16)?.try_into().ok()?)).ok()?,
        ),
        size => (8, size as usize),
    };
    if size < header {
        return None;
    }
    Some((kind, bytes.get(header..size)?, bytes.get(size..)?))
}

fn mp4_track(bytes: &[u8], depth: u8, budget: &mut usize) -> Option<bool> {
    let mut remaining = bytes;
    let mut video = false;
    while !remaining.is_empty() {
        let (kind, payload, rest) = mp4_box(remaining, budget)?;
        remaining = rest;
        match (depth, kind) {
            (0, b"trak") | (1, b"mdia") => video |= mp4_track(payload, depth + 1, budget)?,
            (2, b"hdlr") => {
                if payload.len() < 24 {
                    return None;
                }
                video |= payload.get(8..12) == Some(b"vide");
            }
            _ => {}
        }
    }
    Some(video)
}

fn mp4_video(bytes: &[u8]) -> bool {
    fn parse(bytes: &[u8]) -> Option<bool> {
        let mut budget = 65_536;
        let (kind, header, mut remaining) = mp4_box(bytes, &mut budget)?;
        if kind != b"ftyp" || header.len() < 8 || !header.len().is_multiple_of(4) {
            return None;
        }
        let compatible = |brand: &[u8]| {
            matches!(
                brand,
                b"isom"
                    | b"iso2"
                    | b"iso3"
                    | b"iso4"
                    | b"iso5"
                    | b"iso6"
                    | b"iso7"
                    | b"iso8"
                    | b"iso9"
                    | b"mp41"
                    | b"mp42"
                    | b"avc1"
                    | b"dash"
                    | b"MSNV"
                    | b"M4V "
                    | b"M4VH"
                    | b"M4VP"
            )
        };
        if !compatible(&header[..4])
            && !header[8..]
                .as_chunks::<4>()
                .0
                .iter()
                .any(|brand| compatible(brand))
        {
            return None;
        }
        let mut video = false;
        let mut data = false;
        while !remaining.is_empty() {
            let (kind, payload, rest) = mp4_box(remaining, &mut budget)?;
            remaining = rest;
            match kind {
                b"moov" => video |= mp4_track(payload, 0, &mut budget)?,
                b"mdat" => data |= !payload.is_empty(),
                _ => {}
            }
        }
        Some(video && data)
    }
    parse(bytes) == Some(true)
}

fn ebml_vint(bytes: &[u8], id: bool) -> Option<(u64, usize, bool)> {
    let first = *bytes.first()?;
    let width = first.leading_zeros() as usize + 1;
    if width > if id { 4 } else { 8 } {
        return None;
    }
    let mut value = if id {
        first as u64
    } else {
        u64::from(first) & (0xff_u64 >> width)
    };
    for byte in bytes.get(1..width)? {
        value = (value << 8) | u64::from(*byte);
    }
    let unknown = !id && value == (1_u64 << (width * 7)) - 1;
    Some((value, width, unknown))
}

fn ebml_element<'a>(bytes: &'a [u8], budget: &mut usize) -> Option<(u64, &'a [u8], &'a [u8])> {
    *budget = budget.checked_sub(1)?;
    let (id, id_width, _) = ebml_vint(bytes, true)?;
    let (size, size_width, unknown) = ebml_vint(bytes.get(id_width..)?, false)?;
    let header = id_width + size_width;
    let size = if unknown {
        // Streaming WebM permits unknown Segment/Cluster lengths; these extend
        // to the parent boundary and cannot reinterpret arbitrary child data.
        if !matches!(id, 0x1853_8067 | 0x1f43_b675) {
            return None;
        }
        bytes.len().checked_sub(header)?
    } else {
        usize::try_from(size).ok()?
    };
    let end = header.checked_add(size)?;
    Some((id, bytes.get(header..end)?, bytes.get(end..)?))
}

fn webm_video(bytes: &[u8]) -> bool {
    fn parse(bytes: &[u8]) -> Option<bool> {
        let mut budget = 65_536;
        let (kind, header, mut remaining) = ebml_element(bytes, &mut budget)?;
        if kind != 0x1a45_dfa3 || header.len() > 4096 {
            return None;
        }
        let mut fields = header;
        let mut webm = false;
        while !fields.is_empty() {
            let (id, value, rest) = ebml_element(fields, &mut budget)?;
            fields = rest;
            if id == 0x4282 {
                webm = value == b"webm";
            }
        }
        if !webm {
            return None;
        }
        // Void/CRC metadata may precede the Segment, but no other document is
        // accepted alongside the video container.
        let segment = loop {
            let (id, value, rest) = ebml_element(remaining, &mut budget)?;
            remaining = rest;
            match id {
                0x1853_8067 if remaining.is_empty() => break value,
                0xec | 0xbf => {}
                _ => return None,
            }
        };
        let mut children = segment;
        let mut video = false;
        let mut data = false;
        while !children.is_empty() {
            let (id, value, rest) = ebml_element(children, &mut budget)?;
            children = rest;
            if id == 0x1f43_b675 {
                data |= !value.is_empty();
            } else if id == 0x1654_ae6b {
                let mut tracks = value;
                while !tracks.is_empty() {
                    let (id, value, rest) = ebml_element(tracks, &mut budget)?;
                    tracks = rest;
                    if id != 0xae {
                        continue;
                    }
                    let mut entry = value;
                    while !entry.is_empty() {
                        let (id, value, rest) = ebml_element(entry, &mut budget)?;
                        entry = rest;
                        if id == 0x83 {
                            if value.is_empty() || value.len() > 8 {
                                return None;
                            }
                            video |= value
                                .iter()
                                .fold(0_u64, |n, byte| (n << 8) | u64::from(*byte))
                                == 1;
                        }
                    }
                }
            }
        }
        Some(video && data)
    }
    parse(bytes) == Some(true)
}

#[derive(Clone, Copy)]
enum UploadKind {
    Image,
    Video,
}

impl UploadKind {
    fn limit(self) -> usize {
        match self {
            Self::Image => MAX_IMAGE_BYTES,
            Self::Video => MAX_VIDEO_BYTES,
        }
    }

    fn resource_type(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
        }
    }

    fn detect(self, bytes: &[u8]) -> Result<(&'static str, &'static str), AppError> {
        match self {
            Self::Image => {
                image_type(bytes).ok_or_else(|| invalid("仅支持 PNG、JPEG、GIF 和 WebP 图片"))
            }
            Self::Video => {
                video_type(bytes).ok_or_else(|| invalid("仅支持包含视频轨道的 MP4 和 WebM 视频"))
            }
        }
    }
}

async fn upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<UploadedMedia> {
    let author = crate::site_author_id(&state.pool).await?;
    save_media(&state, &headers, body, author, UploadKind::Image).await
}

async fn upload_video(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<UploadedMedia> {
    let author = crate::site_author_id(&state.pool).await?;
    save_media(&state, &headers, body, author, UploadKind::Video).await
}

async fn save_media(
    state: &AppState,
    headers: &HeaderMap,
    body: Bytes,
    owner: i64,
    kind: UploadKind,
) -> ApiResult<UploadedMedia> {
    let owned = state.clone();
    let headers = headers.clone();
    let result = state
        .scope
        .commit_tasks
        .try_spawn(async move {
            let _operation = owned.media_operations.lock().await;
            save_media_inner(&owned, &headers, body, owner, kind).await
        })
        .map_err(|_| AppError(StatusCode::SERVICE_UNAVAILABLE, "服务正在停止"))?
        .await
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "媒体保存失败"))?;
    result.map(Json)
}

async fn save_media_inner(
    state: &AppState,
    headers: &HeaderMap,
    body: Bytes,
    owner: i64,
    kind: UploadKind,
) -> Result<UploadedMedia, AppError> {
    let limit = kind.limit();
    if body.is_empty() || body.len() > limit {
        return Err(invalid("媒体大小无效"));
    }
    let (extension, mime_type) = kind.detect(&body)?;
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

fn media_filename_kind(name: &str) -> Option<&'static str> {
    let (hash, extension) = name.split_once('.')?;
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    match extension {
        "png" | "jpg" | "gif" | "webp" => Some("image"),
        "mp4" | "webm" => Some("video"),
        _ => None,
    }
}

pub(crate) fn is_media_filename(name: &str) -> bool {
    media_filename_kind(name).is_some()
}

fn filesystem_error(error: impl std::fmt::Display) -> AppError {
    tracing::error!(%error, "media filesystem operation failed");
    AppError(StatusCode::INTERNAL_SERVER_ERROR, "媒体文件操作失败")
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
        "SELECT id,coalesce(background_image,'')||char(0)||coalesce(avatar,'')||char(0)||coalesce(random_cover,'')||char(0)||coalesce(notices,'')||char(0)||coalesce(media,'') AS value FROM web_info",
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
                    let pending = directory.read_bounded(&quarantine, MAX_VIDEO_BYTES)?;
                    let published = directory.read_bounded(&original, MAX_VIDEO_BYTES)?;
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
                        .and_then(media_filename_kind)
                        .is_some_and(|expected| kind.as_deref() == Some(expected))
                })
                .filter(|_| storage.as_deref() == Some("local"))
                .ok_or(AppError(
                    StatusCode::CONFLICT,
                    "仅可删除本站上传的图片或视频",
                ))?;
            if path != input.path {
                return Err(AppError(
                    StatusCode::CONFLICT,
                    "文件记录已经改变，请刷新后重试",
                ));
            }
            if referenced(&mut transaction, &path).await? {
                return Err(AppError(
                    StatusCode::CONFLICT,
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
        .map_err(|_| AppError(StatusCode::INTERNAL_SERVER_ERROR, "媒体删除失败"))?
}

#[path = "../../../test/server/unit/media/reference_tests.rs"]
#[cfg(test)]
mod reference_tests;

#[cfg(test)]
#[path = "../../../test/server/unit/media/video_types.rs"]
mod video_types;
