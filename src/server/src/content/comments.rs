use super::PageResult;
use crate::{ApiResult, AppError, AppState, absent, db_error, invalid};
use axum::{Json, extract::State, http::StatusCode};
use blog_application::server_cli::{ContractPath as Path, ContractQuery as Query};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CommentInput {
    request_id: String,
    article_id: i64,
    content: String,
    parent_comment_id: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MessageCommentInput {
    request_id: String,
    content: String,
    parent_comment_id: Option<i64>,
}

pub(super) async fn create_comment(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<CommentInput>,
) -> ApiResult<Comment> {
    create_anonymous_comment(
        &state,
        input.article_id,
        "article",
        MessageCommentInput {
            request_id: input.request_id,
            content: input.content,
            parent_comment_id: input.parent_comment_id,
        },
    )
    .await
}

pub(super) async fn create_message_comment(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<MessageCommentInput>,
) -> ApiResult<Comment> {
    create_anonymous_comment(&state, 0, "message", input).await
}

pub(super) async fn create_love_comment(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<MessageCommentInput>,
) -> ApiResult<Comment> {
    let source = crate::site_author_id(&state.pool).await?;
    create_anonymous_comment(&state, source, "love", input).await
}

fn valid_request_id(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            14 => byte == b'4',
            19 => matches!(byte, b'8' | b'9' | b'a' | b'b'),
            _ => byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'),
        })
}

async fn create_anonymous_comment(
    state: &AppState,
    source: i64,
    kind: &'static str,
    input: MessageCommentInput,
) -> ApiResult<Comment> {
    let content = input.content.trim();
    let parent = input.parent_comment_id.unwrap_or(0);
    if !valid_request_id(&input.request_id) {
        return Err(invalid("评论请求标识必须为小写 UUIDv4"));
    }
    if content.is_empty()
        || content.chars().count() > 1024
        || content.contains('<')
        || content.contains('>')
        || parent < 0
    {
        return Err(invalid("评论内容或回复目标无效"));
    }
    let mut hash = Sha256::new();
    hash.update(source.to_be_bytes());
    hash.update(kind.as_bytes());
    hash.update([0]);
    hash.update(content.as_bytes());
    hash.update(parent.to_be_bytes());
    let request_hash = hash.finalize().to_vec();
    // Serialize receipt lookup and insertion with the comment.
    // A lost response, concurrent retry or process restart cannot create a second row.
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    // Gate and insert share the writer boundary so a queued request observes
    // administrator changes to publication, comments, deletion or protection.
    if kind == "article" {
        let enabled: Option<i64> = sqlx::query_scalar("SELECT comment_status FROM article WHERE id=? AND deleted=0 AND view_status=1 AND password IS NULL")
            .bind(source).fetch_optional(&mut *transaction).await.map_err(db_error)?;
        if enabled != Some(1) {
            return Err(AppError(StatusCode::FORBIDDEN, "文章不允许评论"));
        }
    }
    let previous: Option<(Vec<u8>, Option<i64>)> =
        sqlx::query_as("SELECT request_hash,comment_id FROM comment_submission WHERE request_id=?")
            .bind(&input.request_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(db_error)?;
    if let Some((previous_hash, comment_id)) = previous {
        if previous_hash != request_hash {
            return Err(AppError(StatusCode::CONFLICT, "评论请求标识已用于不同内容"));
        }
        let id = comment_id.ok_or(AppError(StatusCode::CONFLICT, "原评论已删除，不会重复发表"))?;
        let comment = sqlx::query_as::<_, Comment>(sqlx::AssertSqlSafe(format!(
            "{COMMENT_SQL} WHERE c.id=?"
        )))
        .bind(id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(db_error)?;
        transaction.commit().await.map_err(db_error)?;
        return Ok(Json(comment));
    }
    let (floor, parent_user) = if parent > 0 {
        let row: Option<(Option<i64>, Option<i64>, Option<i64>)> = sqlx::query_as("SELECT parent_comment_id,floor_comment_id,user_id FROM comment WHERE id=? AND source=? AND type=?")
            .bind(parent).bind(source).bind(kind).fetch_optional(&mut *transaction).await.map_err(db_error)?;
        let Some((parent_parent, parent_floor, user)) = row else {
            return Err(invalid("回复目标不存在"));
        };
        (
            Some(if parent_parent.unwrap_or(0) == 0 {
                parent
            } else {
                parent_floor.unwrap_or(parent)
            }),
            user,
        )
    } else {
        (None, None)
    };
    let id = sqlx::query("INSERT INTO comment(source,type,parent_comment_id,user_id,floor_comment_id,parent_user_id,comment_content) VALUES(?,?,?,NULL,?,?,?)")
        .bind(source).bind(kind).bind(parent).bind(floor).bind(parent_user).bind(content)
        .execute(&mut *transaction).await.map_err(db_error)?.last_insert_rowid();
    sqlx::query("INSERT INTO comment_submission(request_id,request_hash,comment_id) VALUES(?,?,?)")
        .bind(&input.request_id)
        .bind(request_hash)
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    let comment =
        sqlx::query_as::<_, Comment>(sqlx::AssertSqlSafe(format!("{COMMENT_SQL} WHERE c.id=?")))
            .bind(id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(comment))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CommentsQuery {
    article_id: i64,
    page: Option<i64>,
    size: Option<i64>,
}
#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Comment {
    id: i64,
    user_id: Option<i64>,
    username: Option<String>,
    avatar: Option<String>,
    comment_content: String,
    create_time: Option<String>,
    parent_comment_id: Option<i64>,
    parent_username: Option<String>,
    floor_comment_id: Option<i64>,
    reply_count: i64,
}
const COMMENT_SQL: &str = "SELECT c.id,c.user_id,u.username,u.avatar,c.comment_content,c.create_time,c.parent_comment_id,pu.username AS parent_username,c.floor_comment_id,(SELECT COUNT(*) FROM comment child WHERE child.floor_comment_id=c.id AND child.type=c.type) AS reply_count FROM comment c LEFT JOIN user u ON u.id=c.user_id LEFT JOIN user pu ON pu.id=c.parent_user_id";
async fn visible_comment_article(
    connection: &mut sqlx::SqliteConnection,
    id: i64,
) -> Result<(), AppError> {
    let visible: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM article WHERE id=? AND deleted=0 AND view_status=1 AND password IS NULL",
    )
    .bind(id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(db_error)?;
    if visible.is_none() {
        return Err(absent());
    }
    Ok(())
}
pub(super) async fn comments(
    State(state): State<AppState>,
    Query(query): Query<CommentsQuery>,
) -> ApiResult<PageResult<Comment>> {
    // Authorization and returned comments must describe the same snapshot.
    let mut transaction = state.pool.begin().await.map_err(db_error)?;
    visible_comment_article(&mut transaction, query.article_id).await?;
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(10).clamp(1, 50);
    let total:i64=sqlx::query_scalar("SELECT COUNT(*) FROM comment WHERE source=? AND type='article' AND (parent_comment_id IS NULL OR parent_comment_id=0)")
        .bind(query.article_id).fetch_one(&mut *transaction).await.map_err(db_error)?;
    let items=sqlx::query_as::<_,Comment>(sqlx::AssertSqlSafe(format!("{COMMENT_SQL} WHERE c.source=? AND c.type='article' AND (c.parent_comment_id IS NULL OR c.parent_comment_id=0) ORDER BY c.create_time DESC,c.id DESC LIMIT ? OFFSET ?")))
        .bind(query.article_id).bind(size).bind((page-1)*size).fetch_all(&mut *transaction).await.map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RepliesQuery {
    page: Option<i64>,
    size: Option<i64>,
}
pub(super) async fn comment_replies(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let mut transaction = state.pool.begin().await.map_err(db_error)?;
    let article:Option<i64>=sqlx::query_scalar("SELECT source FROM comment WHERE id=? AND type='article' AND (parent_comment_id IS NULL OR parent_comment_id=0)")
        .bind(id).fetch_optional(&mut *transaction).await.map_err(db_error)?;
    visible_comment_article(&mut transaction, article.ok_or_else(absent)?).await?;
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(5).clamp(1, 50);
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comment WHERE floor_comment_id=? AND type='article'",
    )
    .bind(id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(db_error)?;
    let items=sqlx::query_as::<_,Comment>(sqlx::AssertSqlSafe(format!("{COMMENT_SQL} WHERE c.floor_comment_id=? AND c.type='article' ORDER BY c.create_time,c.id LIMIT ? OFFSET ?")))
        .bind(id).bind(size).bind((page-1)*size).fetch_all(&mut *transaction).await.map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
pub(super) async fn message_comments(
    State(state): State<AppState>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(10).clamp(1, 50);
    let total:i64=sqlx::query_scalar("SELECT COUNT(*) FROM comment WHERE source=0 AND type='message' AND (parent_comment_id IS NULL OR parent_comment_id=0)")
        .fetch_one(&state.pool).await.map_err(db_error)?;
    let items=sqlx::query_as::<_,Comment>(sqlx::AssertSqlSafe(format!("{COMMENT_SQL} WHERE c.source=0 AND c.type='message' AND (c.parent_comment_id IS NULL OR c.parent_comment_id=0) ORDER BY c.create_time DESC,c.id DESC LIMIT ? OFFSET ?")))
        .bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
pub(super) async fn message_comment_replies(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let root:Option<i64>=sqlx::query_scalar("SELECT id FROM comment WHERE id=? AND source=0 AND type='message' AND (parent_comment_id IS NULL OR parent_comment_id=0)")
        .bind(id).fetch_optional(&state.pool).await.map_err(db_error)?;
    if root.is_none() {
        return Err(absent());
    }
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(5).clamp(1, 50);
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comment WHERE floor_comment_id=? AND source=0 AND type='message'",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?;
    let items=sqlx::query_as::<_,Comment>(sqlx::AssertSqlSafe(format!("{COMMENT_SQL} WHERE c.floor_comment_id=? AND c.source=0 AND c.type='message' ORDER BY c.create_time,c.id LIMIT ? OFFSET ?")))
        .bind(id).bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
pub(super) async fn love_comments(
    State(state): State<AppState>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let source = crate::site_author_id(&state.pool).await?;
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(10).clamp(1, 50);
    let total:i64=sqlx::query_scalar("SELECT COUNT(*) FROM comment WHERE source=? AND type='love' AND (parent_comment_id IS NULL OR parent_comment_id=0)")
        .bind(source).fetch_one(&state.pool).await.map_err(db_error)?;
    let items=sqlx::query_as::<_,Comment>(sqlx::AssertSqlSafe(format!("{COMMENT_SQL} WHERE c.source=? AND c.type='love' AND (c.parent_comment_id IS NULL OR c.parent_comment_id=0) ORDER BY c.create_time DESC,c.id DESC LIMIT ? OFFSET ?")))
        .bind(source).bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
pub(super) async fn love_comment_replies(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let source = crate::site_author_id(&state.pool).await?;
    let root:Option<i64>=sqlx::query_scalar("SELECT id FROM comment WHERE id=? AND source=? AND type='love' AND (parent_comment_id IS NULL OR parent_comment_id=0)")
        .bind(id).bind(source).fetch_optional(&state.pool).await.map_err(db_error)?;
    if root.is_none() {
        return Err(absent());
    }
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(5).clamp(1, 50);
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comment WHERE floor_comment_id=? AND source=? AND type='love'",
    )
    .bind(id)
    .bind(source)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?;
    let items=sqlx::query_as::<_,Comment>(sqlx::AssertSqlSafe(format!("{COMMENT_SQL} WHERE c.floor_comment_id=? AND c.source=? AND c.type='love' ORDER BY c.create_time,c.id LIMIT ? OFFSET ?")))
        .bind(id).bind(source).bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
