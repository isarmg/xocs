use super::{AdminPageQuery, PageResult};
use crate::{ApiResult, AppError, AppState, absent, invalid};
use axum::{Json, extract::State};
use blog_application::admin_core::AuthenticatedIdentity;
use blog_application::error::HttpStatus;
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
        CommentTarget::Article(input.article_id),
        CommentSubmission::parse(MessageCommentInput {
            request_id: input.request_id,
            content: input.content,
            parent_comment_id: input.parent_comment_id,
        })?,
    )
    .await
}

pub(super) async fn create_message_comment(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<MessageCommentInput>,
) -> ApiResult<Comment> {
    create_anonymous_comment(
        &state,
        CommentTarget::Message,
        CommentSubmission::parse(input)?,
    )
    .await
}

pub(super) async fn create_love_comment(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<MessageCommentInput>,
) -> ApiResult<Comment> {
    let source = crate::site_author_id(&state.pool).await?;
    create_anonymous_comment(
        &state,
        CommentTarget::Love(source),
        CommentSubmission::parse(input)?,
    )
    .await
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

enum CommentTarget {
    Article(i64),
    Message,
    Love(i64),
}

impl CommentTarget {
    fn source(&self) -> i64 {
        match *self {
            Self::Article(id) | Self::Love(id) => id,
            Self::Message => 0,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Article(_) => "article",
            Self::Message => "message",
            Self::Love(_) => "love",
        }
    }
}

struct CommentSubmission {
    request_id: String,
    content: String,
    parent: i64,
}

impl CommentSubmission {
    fn parse(input: MessageCommentInput) -> Result<Self, AppError> {
        if !valid_request_id(&input.request_id) {
            return Err(invalid("评论请求标识必须为小写 UUIDv4"));
        }
        let content = input.content.trim();
        let parent = input.parent_comment_id.unwrap_or(0);
        if content.is_empty() || content.chars().count() > 1024 || parent < 0 {
            return Err(invalid("评论内容或回复目标无效"));
        }
        Ok(Self {
            request_id: input.request_id,
            content: content.to_owned(),
            parent,
        })
    }
}

async fn create_anonymous_comment(
    state: &AppState,
    target: CommentTarget,
    input: CommentSubmission,
) -> ApiResult<Comment> {
    let source = target.source();
    let kind = target.as_str();
    let content = &input.content;
    let parent = input.parent;
    let mut hash = Sha256::new();
    hash.update(source.to_be_bytes());
    hash.update(kind.as_bytes());
    hash.update([0]);
    hash.update(content.as_bytes());
    hash.update(parent.to_be_bytes());
    let request_hash = hash.finalize().to_vec();
    // Serialize receipt lookup and insertion with the comment.
    // A lost response, concurrent retry or process restart cannot create a second row.
    let mut transaction = state.pool.begin_with("BEGIN IMMEDIATE").await?;
    // Gate and insert share the writer boundary so a queued request observes
    // administrator changes to publication, comments, deletion or protection.
    if let CommentTarget::Article(_) = target {
        let enabled: Option<i64> = sqlx::query_scalar("SELECT comment_status FROM article WHERE id=? AND deleted=0 AND view_status=1 AND password IS NULL")
            .bind(source).fetch_optional(&mut *transaction).await?;
        if enabled != Some(1) {
            return Err(AppError::Public(HttpStatus::Forbidden, "文章不允许评论"));
        }
    }
    let previous: Option<(Vec<u8>, Option<i64>)> =
        sqlx::query_as("SELECT request_hash,comment_id FROM comment_submission WHERE request_id=?")
            .bind(&input.request_id)
            .fetch_optional(&mut *transaction)
            .await?;
    if let Some((previous_hash, comment_id)) = previous {
        if previous_hash != request_hash {
            return Err(AppError::Public(
                HttpStatus::Conflict,
                "评论请求标识已用于不同内容",
            ));
        }
        let id = comment_id.ok_or(AppError::Public(
            HttpStatus::Conflict,
            "原评论已删除，不会重复发表",
        ))?;
        let comment = sqlx::query_as::<_, Comment>(sqlx::AssertSqlSafe(format!(
            "{COMMENT_SQL} WHERE c.id=?"
        )))
        .bind(id)
        .fetch_one(&mut *transaction)
        .await?;
        transaction.commit().await?;
        return Ok(Json(comment));
    }
    let (floor, parent_user) = if parent > 0 {
        let row: Option<(Option<i64>, Option<i64>, Option<i64>)> = sqlx::query_as("SELECT parent_comment_id,floor_comment_id,user_id FROM comment WHERE id=? AND source=? AND type=?")
            .bind(parent).bind(source).bind(kind).fetch_optional(&mut *transaction).await?;
        let Some((parent_parent, parent_floor, user)) = row else {
            return Err(invalid("回复目标不存在"));
        };
        (
            Some(if parent_parent.unwrap_or(0) == 0 {
                parent
            } else {
                parent_floor.ok_or(AppError::Public(
                    HttpStatus::Conflict,
                    "回复目标的楼层信息无效",
                ))?
            }),
            user,
        )
    } else {
        (None, None)
    };
    let id = sqlx::query("INSERT INTO comment(source,type,parent_comment_id,user_id,floor_comment_id,parent_user_id,comment_content) VALUES(?,?,?,NULL,?,?,?)")
        .bind(source).bind(kind).bind(parent).bind(floor).bind(parent_user).bind(content)
        .execute(&mut *transaction).await?.last_insert_rowid();
    sqlx::query("INSERT INTO comment_submission(request_id,request_hash,comment_id) VALUES(?,?,?)")
        .bind(&input.request_id)
        .bind(request_hash)
        .bind(id)
        .execute(&mut *transaction)
        .await?;
    let comment =
        sqlx::query_as::<_, Comment>(sqlx::AssertSqlSafe(format!("{COMMENT_SQL} WHERE c.id=?")))
            .bind(id)
            .fetch_one(&mut *transaction)
            .await?;
    transaction.commit().await?;
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
const COMMENT_SQL: &str = "SELECT c.id,c.user_id,u.username,u.avatar,c.comment_content,c.create_time,
    c.parent_comment_id,pu.username AS parent_username,c.floor_comment_id,
    (SELECT COUNT(*) FROM comment child WHERE child.floor_comment_id=c.id AND child.type=c.type) AS reply_count
    FROM comment c LEFT JOIN user u ON u.id=c.user_id LEFT JOIN user pu ON pu.id=c.parent_user_id";
async fn visible_comment_article(
    connection: &mut sqlx::SqliteConnection,
    id: i64,
) -> Result<(), AppError> {
    let visible: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM article WHERE id=? AND deleted=0 AND view_status=1 AND password IS NULL",
    )
    .bind(id)
    .fetch_optional(&mut *connection)
    .await?;
    if visible.is_none() {
        return Err(absent());
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RepliesQuery {
    page: Option<i64>,
    size: Option<i64>,
}

struct CommentPagination {
    page: i64,
    size: i64,
}

impl RepliesQuery {
    fn bounds(self, default_size: i64) -> CommentPagination {
        CommentPagination {
            page: self.page.unwrap_or(1).clamp(1, 100_000),
            size: self.size.unwrap_or(default_size).clamp(1, 50),
        }
    }
}

async fn read_root_comments(
    connection: &mut sqlx::SqliteConnection,
    target: CommentTarget,
    pagination: CommentPagination,
) -> Result<PageResult<Comment>, AppError> {
    let CommentPagination { page, size } = pagination;
    let total = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comment WHERE source=? AND type=?
            AND (parent_comment_id IS NULL OR parent_comment_id=0)",
    )
    .bind(target.source())
    .bind(target.as_str())
    .fetch_one(&mut *connection)
    .await?;
    let items = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{COMMENT_SQL} WHERE c.source=? AND c.type=?
            AND (c.parent_comment_id IS NULL OR c.parent_comment_id=0)
            ORDER BY c.create_time DESC,c.id DESC LIMIT ? OFFSET ?"
    )))
    .bind(target.source())
    .bind(target.as_str())
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(connection)
    .await?;
    Ok(PageResult {
        items,
        total,
        page,
        size,
    })
}

async fn read_comment_replies(
    connection: &mut sqlx::SqliteConnection,
    target: CommentTarget,
    id: i64,
    pagination: CommentPagination,
) -> Result<PageResult<Comment>, AppError> {
    let CommentPagination { page, size } = pagination;
    let total = sqlx::query_scalar(
        "SELECT COUNT(*) FROM comment WHERE floor_comment_id=? AND source=? AND type=?",
    )
    .bind(id)
    .bind(target.source())
    .bind(target.as_str())
    .fetch_one(&mut *connection)
    .await?;
    let items = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{COMMENT_SQL} WHERE c.floor_comment_id=? AND c.source=? AND c.type=?
            ORDER BY c.create_time,c.id LIMIT ? OFFSET ?"
    )))
    .bind(id)
    .bind(target.source())
    .bind(target.as_str())
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(connection)
    .await?;
    Ok(PageResult {
        items,
        total,
        page,
        size,
    })
}

pub(super) async fn comments(
    State(state): State<AppState>,
    Query(query): Query<CommentsQuery>,
) -> ApiResult<PageResult<Comment>> {
    // Authorization and returned comments must describe the same snapshot.
    let mut transaction = state.pool.begin().await?;
    visible_comment_article(&mut transaction, query.article_id).await?;
    let pagination = RepliesQuery {
        page: query.page,
        size: query.size,
    }
    .bounds(10);
    let result = read_root_comments(
        &mut transaction,
        CommentTarget::Article(query.article_id),
        pagination,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(result))
}

pub(super) async fn comment_replies(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let mut transaction = state.pool.begin().await?;
    let article = sqlx::query_scalar(
        "SELECT source FROM comment WHERE id=? AND type='article'
            AND (parent_comment_id IS NULL OR parent_comment_id=0)",
    )
    .bind(id)
    .fetch_optional(&mut *transaction)
    .await?;
    let article = article.ok_or_else(absent)?;
    visible_comment_article(&mut transaction, article).await?;
    let result = read_comment_replies(
        &mut transaction,
        CommentTarget::Article(article),
        id,
        query.bounds(5),
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(result))
}

pub(super) async fn message_comments(
    State(state): State<AppState>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let mut transaction = state.pool.begin().await?;
    let result =
        read_root_comments(&mut transaction, CommentTarget::Message, query.bounds(10)).await?;
    transaction.commit().await?;
    Ok(Json(result))
}

pub(super) async fn message_comment_replies(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let mut transaction = state.pool.begin().await?;
    let root: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM comment WHERE id=? AND source=0 AND type='message'
            AND (parent_comment_id IS NULL OR parent_comment_id=0)",
    )
    .bind(id)
    .fetch_optional(&mut *transaction)
    .await?;
    if root.is_none() {
        return Err(absent());
    }
    let result = read_comment_replies(
        &mut transaction,
        CommentTarget::Message,
        id,
        query.bounds(5),
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(result))
}

pub(super) async fn love_comments(
    State(state): State<AppState>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let mut transaction = state.pool.begin().await?;
    let source = crate::site_author_id(&mut *transaction).await?;
    let result = read_root_comments(
        &mut transaction,
        CommentTarget::Love(source),
        query.bounds(10),
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(result))
}

pub(super) async fn love_comment_replies(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<RepliesQuery>,
) -> ApiResult<PageResult<Comment>> {
    let mut transaction = state.pool.begin().await?;
    let source = crate::site_author_id(&mut *transaction).await?;
    let root: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM comment WHERE id=? AND source=? AND type='love'
            AND (parent_comment_id IS NULL OR parent_comment_id=0)",
    )
    .bind(id)
    .bind(source)
    .fetch_optional(&mut *transaction)
    .await?;
    if root.is_none() {
        return Err(absent());
    }
    let result = read_comment_replies(
        &mut transaction,
        CommentTarget::Love(source),
        id,
        query.bounds(5),
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(result))
}

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct AdminComment {
    id: i64,
    source: i64,
    comment_type: Option<String>,
    user_id: Option<i64>,
    username: Option<String>,
    comment_content: String,
    create_time: Option<String>,
}
pub(super) async fn admin_comments(
    State(state): State<AppState>,
    Query(query): Query<AdminPageQuery>,
) -> ApiResult<PageResult<AdminComment>> {
    let (page, size) = query.bounds();
    let mut transaction = state.pool.begin().await?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comment")
        .fetch_one(&mut *transaction)
        .await?;
    let items=sqlx::query_as::<_,AdminComment>("SELECT c.id,c.source,c.type AS comment_type,c.user_id,u.username,c.comment_content,c.create_time FROM comment c LEFT JOIN user u ON u.id=c.user_id ORDER BY c.id DESC LIMIT ? OFFSET ?")
        .bind(size).bind((page-1)*size).fetch_all(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(PageResult {
        items,
        total,
        page,
        size,
    }))
}
pub(super) async fn delete_comment(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let root: Option<Option<i64>> =
        sqlx::query_scalar("SELECT parent_comment_id FROM comment WHERE id=?")
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await?;
    let Some(parent) = root else {
        return Err(absent());
    };
    if parent.unwrap_or(0) == 0 {
        sqlx::query("DELETE FROM comment WHERE id=? OR floor_comment_id=?")
            .bind(id)
            .bind(id)
            .execute(&mut *transaction)
            .await?;
    } else {
        sqlx::query("DELETE FROM comment WHERE id=?")
            .bind(id)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;
    Ok(Json(serde_json::json!({"deleted":true})))
}
