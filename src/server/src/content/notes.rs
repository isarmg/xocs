use super::{AdminPageQuery, PageResult};
use crate::{ApiResult, AppError, AppState, absent, db_error, invalid};
use axum::{Json, extract::State};
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
    let author = crate::site_author_id(&state.pool).await?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM wei_yan WHERE type='friend' AND user_id=? AND is_public=1",
    )
    .bind(author)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?;
    let items=sqlx::query_as::<_,Note>("SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.like_count,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id WHERE w.type='friend' AND w.user_id=? AND w.is_public=1 ORDER BY w.id DESC LIMIT ? OFFSET ?")
        .bind(author).bind(size).bind((page-1)*size)
        .fetch_all(&state.pool).await.map_err(db_error)?;
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
    content: String,
    #[serde(deserialize_with = "Deserialize::deserialize")]
    image_path: Option<String>,
    is_public: bool,
}

const ADMIN_NOTE_SQL: &str = "SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id";

async fn validate_admin_note(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    input: &AdminNoteInput,
) -> Result<(), AppError> {
    let content = input.content.trim();
    if content.is_empty() || content.chars().count() > 1024 {
        return Err(invalid("动态内容应为 1 至 1024 字"));
    }
    if let Some(path) = &input.image_path {
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
            .bind(path).fetch_one(&mut **transaction).await.map_err(db_error)?;
        if !valid {
            return Err(invalid("动态图片必须选择已上传的本地图片"));
        }
    }
    Ok(())
}

async fn load_admin_note(state: &AppState, id: i64) -> ApiResult<AdminNote> {
    Ok(Json(
        sqlx::query_as::<_, AdminNote>(sqlx::AssertSqlSafe(format!(
            "{ADMIN_NOTE_SQL} WHERE w.id=? AND w.type='friend'"
        )))
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(db_error)?
        .ok_or_else(absent)?,
    ))
}

pub(super) async fn create_admin_note(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<AdminNoteInput>,
) -> ApiResult<AdminNote> {
    let author = crate::site_author_id(&state.pool).await?;
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    validate_admin_note(&mut transaction, &input).await?;
    let id = sqlx::query(
        "INSERT INTO wei_yan(user_id,content,image_path,type,is_public) VALUES(?,?,?,'friend',?)",
    )
    .bind(author)
    .bind(input.content.trim())
    .bind(input.image_path)
    .bind(i64::from(input.is_public))
    .execute(&mut *transaction)
    .await
    .map_err(db_error)?
    .last_insert_rowid();
    transaction.commit().await.map_err(db_error)?;
    load_admin_note(&state, id).await
}

pub(super) async fn update_admin_note(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<AdminNoteInput>,
) -> ApiResult<AdminNote> {
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wei_yan WHERE id=? AND type='friend')")
            .bind(id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(db_error)?;
    if !exists {
        return Err(absent());
    }
    validate_admin_note(&mut transaction, &input).await?;
    sqlx::query(
        "UPDATE wei_yan SET content=?,image_path=?,is_public=? WHERE id=? AND type='friend'",
    )
    .bind(input.content.trim())
    .bind(input.image_path)
    .bind(i64::from(input.is_public))
    .bind(id)
    .execute(&mut *transaction)
    .await
    .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    load_admin_note(&state, id).await
}
pub(super) async fn content_notes(
    State(state): State<AppState>,
    Query(query): Query<AdminPageQuery>,
) -> ApiResult<PageResult<AdminNote>> {
    let (page, size) = query.bounds();
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wei_yan WHERE type='friend'")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let items = sqlx::query_as::<_,AdminNote>("SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id WHERE w.type='friend' ORDER BY w.id DESC LIMIT ? OFFSET ?")
        .bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
pub(super) async fn delete_note(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let changed = sqlx::query("DELETE FROM wei_yan WHERE id=? AND type='friend'")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(serde_json::json!({"deleted":true})))
}
