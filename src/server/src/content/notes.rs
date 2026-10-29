use super::{
    AdminPageQuery, PageResult,
    input::{IdempotencyKey, ShortContent},
};
use crate::{ApiResult, AppError, AppState, absent, invalid};
use axum::{Json, extract::State, http::HeaderMap};
use blog_application::admin_core::AuthenticatedIdentity;
use blog_application::error::HttpStatus;
use blog_application::server_cli::{ContractPath as Path, ContractQuery as Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Note {
    id: i64,
    user_id: Option<i64>,
    username: Option<String>,
    content: String,
    image_path: Option<String>,
    like_count: i64,
    is_public: i64,
    create_time: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NotePageQuery {
    page: Option<i64>,
    size: Option<i64>,
}
async fn list_note_page(
    state: &AppState,
    query: NotePageQuery,
) -> Result<PageResult<Note>, AppError> {
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(10).clamp(1, 50);
    let mut transaction = state.pool.begin().await?;
    let author = crate::site_author_id(&mut *transaction).await?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM wei_yan WHERE type='friend' AND user_id=? AND is_public=1",
    )
    .bind(author)
    .fetch_one(&mut *transaction)
    .await?;
    let items=sqlx::query_as::<_,Note>("SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.like_count,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id WHERE w.type='friend' AND w.user_id=? AND w.is_public=1 ORDER BY w.id DESC LIMIT ? OFFSET ?")
        .bind(author).bind(size).bind((page-1)*size)
        .fetch_all(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(PageResult {
        items,
        total,
        page,
        size,
    })
}
pub(super) async fn note_page(
    State(state): State<AppState>,
    Query(query): Query<NotePageQuery>,
) -> ApiResult<PageResult<Note>> {
    Ok(Json(list_note_page(&state, query).await?))
}

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct AdminNote {
    id: i64,
    user_id: Option<i64>,
    username: Option<String>,
    content: String,
    image_path: Option<String>,
    is_public: i64,
    create_time: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdminNoteInput {
    content: ShortContent,
    #[serde(deserialize_with = "Deserialize::deserialize")]
    image_path: Option<String>,
    is_public: bool,
}

const ADMIN_NOTE_SQL: &str = "SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id";

async fn validate_note_image(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    image_path: &Option<String>,
) -> Result<(), AppError> {
    if let Some(path) = image_path {
        let local_image = path
            .strip_prefix("/media/")
            .and_then(|name| name.rsplit_once('.'))
            .is_some_and(|(hash, extension)| {
                hash.len() == 64
                    && hash
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
                    && matches!(extension, "png" | "jpg" | "gif" | "webp")
            });
        if !local_image {
            return Err(invalid("动态图片必须选择已上传的本地图片"));
        }
        let valid: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM resource WHERE path=? AND type='image' AND store_type='local' AND status=1 AND mime_type IN ('image/png','image/jpeg','image/gif','image/webp'))")
            .bind(path).fetch_one(&mut **transaction).await?;
        if !valid {
            return Err(invalid("动态图片必须选择已上传的本地图片"));
        }
    }
    Ok(())
}

async fn load_admin_note(
    connection: &mut sqlx::SqliteConnection,
    id: i64,
) -> Result<AdminNote, AppError> {
    sqlx::query_as::<_, AdminNote>(sqlx::AssertSqlSafe(format!(
        "{ADMIN_NOTE_SQL} WHERE w.id=? AND w.type='friend'"
    )))
    .bind(id)
    .fetch_optional(connection)
    .await?
    .ok_or_else(absent)
}

pub(super) async fn create_admin_note(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    headers: HeaderMap,
    crate::ContractJson(input): crate::ContractJson<AdminNoteInput>,
) -> ApiResult<AdminNote> {
    let request_id = IdempotencyKey::from_headers(&headers)?;
    use sha2::{Digest, Sha256};
    // Bind the initial normalized draft, independently of later edits to its row.
    let request_hash = Sha256::digest(
        serde_json::to_vec(&(input.content.as_str(), &input.image_path, input.is_public))
            .expect("invariant: note strings and flags serialize to JSON"),
    )
    .to_vec();
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    if let Some(key) = &request_id {
        let receipt: Option<(Vec<u8>, Option<i64>)> =
            sqlx::query_as("SELECT request_hash,note_id FROM note_submission WHERE request_id=?")
                .bind(key.as_str())
                .fetch_optional(&mut *transaction)
                .await?;
        if let Some((hash, id)) = receipt {
            if hash != request_hash {
                return Err(AppError::Public(
                    HttpStatus::Conflict,
                    "动态请求标识已用于不同内容",
                ));
            }
            let id = id.ok_or(AppError::Public(
                HttpStatus::Conflict,
                "原动态已删除，不会重复发表",
            ))?;
            // A removed original image need not remain available after an edit.
            // Recovery returns the current row without revalidating old uploads.
            let note = load_admin_note(&mut transaction, id).await?;
            transaction.commit().await?;
            return Ok(Json(note));
        }
    }
    let author = crate::site_author_id(&mut *transaction).await?;
    validate_note_image(&mut transaction, &input.image_path).await?;
    let id = sqlx::query(
        "INSERT INTO wei_yan(user_id,content,image_path,type,is_public) VALUES(?,?,?,'friend',?)",
    )
    .bind(author)
    .bind(input.content.as_str())
    .bind(input.image_path)
    .bind(i64::from(input.is_public))
    .execute(&mut *transaction)
    .await?
    .last_insert_rowid();
    if let Some(key) = request_id {
        sqlx::query("INSERT INTO note_submission(request_id,request_hash,note_id) VALUES(?,?,?)")
            .bind(key.as_str())
            .bind(request_hash)
            .bind(id)
            .execute(&mut *transaction)
            .await?;
    }
    let note = load_admin_note(&mut transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(note))
}

pub(super) async fn update_admin_note(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<AdminNoteInput>,
) -> ApiResult<AdminNote> {
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wei_yan WHERE id=? AND type='friend')")
            .bind(id)
            .fetch_one(&mut *transaction)
            .await?;
    if !exists {
        return Err(absent());
    }
    validate_note_image(&mut transaction, &input.image_path).await?;
    sqlx::query(
        "UPDATE wei_yan SET content=?,image_path=?,is_public=? WHERE id=? AND type='friend'",
    )
    .bind(input.content.as_str())
    .bind(input.image_path)
    .bind(i64::from(input.is_public))
    .bind(id)
    .execute(&mut *transaction)
    .await?;
    let note = load_admin_note(&mut transaction, id).await?;
    transaction.commit().await?;
    Ok(Json(note))
}
pub(super) async fn content_notes(
    State(state): State<AppState>,
    Query(query): Query<AdminPageQuery>,
) -> ApiResult<PageResult<AdminNote>> {
    let (page, size) = query.bounds();
    let mut transaction = state.pool.begin().await?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wei_yan WHERE type='friend'")
        .fetch_one(&mut *transaction)
        .await?;
    let items = sqlx::query_as::<_,AdminNote>("SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id WHERE w.type='friend' ORDER BY w.id DESC LIMIT ? OFFSET ?")
        .bind(size).bind((page-1)*size).fetch_all(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
pub(super) async fn delete_note(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let changed = sqlx::query("DELETE FROM wei_yan WHERE id=? AND type='friend'")
        .bind(id)
        .execute(&mut *transaction)
        .await?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    transaction.commit().await?;
    Ok(Json(serde_json::json!({"deleted":true})))
}
