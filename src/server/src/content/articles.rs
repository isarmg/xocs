use super::{
    PageResult,
    input::{IdempotencyKey, ShortContent},
};
use crate::{ApiResult, AppError, AppState, absent, db_error, invalid};
use axum::{Json, extract::State, http::HeaderMap};
use blog_application::admin_core::AuthenticatedIdentity;
use blog_application::server_cli::{ContractPath as Path, ContractQuery as Query};
use blog_application::{admin_auth::Password, error::HttpStatus};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct ArticleSummary {
    id: i64,
    article_title: String,
    article_cover: Option<String>,
    sort_id: i64,
    label_id: i64,
    sort_name: Option<String>,
    label_name: Option<String>,
    view_count: i64,
    like_count: i64,
    comment_count: i64,
    recommend_status: i64,
    view_status: i64,
    create_time: Option<String>,
    excerpt: Option<String>,
    search_snippet: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Article {
    id: i64,
    user_id: i64,
    article_title: String,
    article_content: String,
    article_cover: Option<String>,
    video_url: Option<String>,
    sort_id: i64,
    label_id: i64,
    view_count: i64,
    like_count: i64,
    comment_count: i64,
    username: Option<String>,
    sort_name: Option<String>,
    label_name: Option<String>,
    recommend_status: i64,
    comment_status: i64,
    view_status: i64,
    password_required: i64,
    tips: Option<String>,
    create_time: Option<String>,
    update_time: Option<String>,
}

const ARTICLE_SUMMARY_SQL: &str = "SELECT id,article_title,article_cover,sort_id,label_id,(SELECT sort_name FROM sort WHERE sort.id=article.sort_id) AS sort_name,(SELECT label_name FROM label WHERE label.id=article.label_id) AS label_name,view_count,like_count,(SELECT COUNT(*) FROM comment WHERE source=article.id AND type='article') AS comment_count,recommend_status,view_status,create_time,CASE WHEN password IS NULL THEN substr(article_content,1,400) ELSE NULL END AS excerpt,CASE WHEN ?2<>'' AND password IS NULL AND article_title NOT LIKE ?3 ESCAPE '\\' THEN substr(article_content,max(1,instr(lower(article_content),lower(?2))-48),160) ELSE NULL END AS search_snippet FROM article";
const ARTICLE_SQL: &str = "SELECT id,user_id,article_title,article_content,article_cover,video_url,sort_id,label_id,view_count,like_count,(SELECT COUNT(*) FROM comment WHERE source=article.id AND type='article') AS comment_count,(SELECT username FROM user WHERE user.id=article.user_id) AS username,(SELECT sort_name FROM sort WHERE sort.id=article.sort_id) AS sort_name,(SELECT label_name FROM label WHERE label.id=article.label_id) AS label_name,recommend_status,comment_status,view_status,CASE WHEN password IS NULL THEN 0 ELSE 1 END AS password_required,tips,create_time,update_time FROM article";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArticleQuery {
    page: Option<i64>,
    size: Option<i64>,
    sort_id: Option<i64>,
    label_id: Option<i64>,
    search: Option<String>,
    recommended: Option<bool>,
}

#[derive(Clone, Copy, Debug)]
enum ArticleAudience {
    Public,
    Administrator,
}

#[derive(Clone, Copy)]
enum SearchMode {
    Browse,
    Search,
}

async fn list_articles(
    pool: &SqlitePool,
    query: ArticleQuery,
    audience: ArticleAudience,
) -> Result<PageResult<ArticleSummary>, AppError> {
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(12).clamp(1, 100);
    let search = query.search.as_deref().unwrap_or_default().trim();
    if search.encode_utf16().count() > 200 {
        return Err(invalid("搜索词过长"));
    }
    let pattern = format!(
        "%{}%",
        search
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    );
    let predicate = match audience {
        ArticleAudience::Public => "deleted=0 AND (view_status=1 OR password IS NOT NULL)",
        ArticleAudience::Administrator => "deleted=0",
    };
    let recommended = query.recommended.map(i64::from);
    let phrase = format!("\"{}\"", search.replace('"', "\"\""));
    let body_match = match audience {
        ArticleAudience::Public => "(password IS NULL AND article_content LIKE ?3 ESCAPE '\\')",
        ArticleAudience::Administrator => "article_content LIKE ?3 ESCAPE '\\'",
    };
    // Bind positions stay fixed, while absent filters no longer hide useful
    // equality constraints from SQLite behind parameter-dependent OR terms.
    let filters = format!(
        "{} AND {} AND {}",
        if query.sort_id.is_some() {
            "sort_id=?1"
        } else {
            "?1 IS NULL"
        },
        if query.label_id.is_some() {
            "label_id=?6"
        } else {
            "?6 IS NULL"
        },
        if recommended.is_some() {
            "recommend_status=?4"
        } else {
            "?4 IS NULL"
        },
    );
    let search_mode = if search.is_empty() {
        SearchMode::Browse
    } else {
        SearchMode::Search
    };
    let filter = match search_mode {
        SearchMode::Browse => format!("{predicate} AND {filters}"),
        SearchMode::Search => {
            let search_index = " AND (length(?2)<3 OR id IN (SELECT rowid FROM article_search WHERE article_search MATCH ?5))";
            format!(
                "{predicate} AND {filters} AND (article_title LIKE ?3 ESCAPE '\\' OR {body_match}){search_index}"
            )
        }
    };
    let (count_sql, list_sql) = article_page_sql(&filter, &filters, audience, search_mode);
    let mut transaction = pool.begin().await.map_err(db_error)?;
    let total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(count_sql))
        .bind(query.sort_id)
        .bind(search)
        .bind(&pattern)
        .bind(recommended)
        .bind(&phrase)
        .bind(query.label_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(db_error)?;
    let items = sqlx::query_as::<_, ArticleSummary>(sqlx::AssertSqlSafe(list_sql))
        .bind(query.sort_id)
        .bind(search)
        .bind(&pattern)
        .bind(recommended)
        .bind(&phrase)
        .bind(query.label_id)
        .bind(size)
        .bind((page - 1) * size)
        .fetch_all(&mut *transaction)
        .await
        .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(PageResult {
        items,
        total,
        page,
        size,
    })
}

fn article_page_sql(
    filter: &str,
    filters: &str,
    audience: ArticleAudience,
    search_mode: SearchMode,
) -> (String, String) {
    let (count, page) = match (audience, search_mode) {
        (ArticleAudience::Public, SearchMode::Browse) => {
            // The disjoint visible/protected branches preserve the public contract.
            // The visible branch can walk article_public_latest in date order.
            let visible = format!("deleted=0 AND view_status=1 AND {filters}");
            // Separate index ranges also avoid scanning every visible row merely
            // to discover that it does not belong to the protected branch.
            let protected_before =
                format!("deleted=0 AND view_status<1 AND password IS NOT NULL AND {filters}");
            let protected_after =
                format!("deleted=0 AND view_status>1 AND password IS NOT NULL AND {filters}");
            (
                format!(
                    "SELECT (SELECT COUNT(*) FROM article WHERE {visible}) + (SELECT COUNT(*) FROM article WHERE {protected_before}) + (SELECT COUNT(*) FROM article WHERE {protected_after})"
                ),
                format!(
                    "SELECT id AS article_id,create_time AS page_created_at,0 AS page_search_rank FROM article WHERE {visible} UNION ALL SELECT id AS article_id,create_time AS page_created_at,0 AS page_search_rank FROM article WHERE {protected_before} UNION ALL SELECT id AS article_id,create_time AS page_created_at,0 AS page_search_rank FROM article WHERE {protected_after} ORDER BY page_created_at DESC,article_id DESC LIMIT ?7 OFFSET ?8"
                ),
            )
        }
        (ArticleAudience::Administrator, SearchMode::Browse)
        | (ArticleAudience::Public, SearchMode::Search)
        | (ArticleAudience::Administrator, SearchMode::Search) => {
            let (rank, order) = match search_mode {
                SearchMode::Search => (
                    "CASE WHEN article_title LIKE ?3 ESCAPE '\\' THEN 0 ELSE 1 END",
                    "page_search_rank,page_created_at DESC,article_id DESC",
                ),
                SearchMode::Browse => ("0", "page_created_at DESC,article_id DESC"),
            };
            (
                format!("SELECT COUNT(*) FROM article WHERE {filter}"),
                format!(
                    "SELECT id AS article_id,create_time AS page_created_at,{rank} AS page_search_rank FROM article WHERE {filter} ORDER BY {order} LIMIT ?7 OFFSET ?8"
                ),
            )
        }
    };
    // Materialize only a page of IDs before reading body excerpts, taxonomy and
    // comment counts. CROSS JOIN fixes the page as the outer loop, so OFFSET
    // and sorting never process full article summaries for discarded rows.
    let summaries =
        ARTICLE_SUMMARY_SQL.replace(" FROM article", " FROM article_page CROSS JOIN article");
    let list = format!(
        "WITH article_page AS MATERIALIZED ({page}) {summaries} WHERE article.id=article_page.article_id ORDER BY article_page.page_search_rank,article_page.page_created_at DESC,article_page.article_id DESC"
    );
    (count, list)
}

pub(super) async fn public_articles(
    State(state): State<AppState>,
    Query(query): Query<ArticleQuery>,
) -> ApiResult<PageResult<ArticleSummary>> {
    Ok(Json(
        list_articles(&state.pool, query, ArticleAudience::Public).await?,
    ))
}
pub(super) async fn admin_articles(
    State(state): State<AppState>,
    Query(query): Query<ArticleQuery>,
) -> ApiResult<PageResult<ArticleSummary>> {
    Ok(Json(
        list_articles(&state.pool, query, ArticleAudience::Administrator).await?,
    ))
}

async fn load_article<'executor, E>(
    executor: E,
    id: i64,
    audience: ArticleAudience,
) -> Result<Article, AppError>
where
    E: sqlx::Executor<'executor, Database = sqlx::Sqlite>,
{
    let condition = match audience {
        ArticleAudience::Administrator => "id=? AND deleted=0",
        ArticleAudience::Public => "id=? AND deleted=0 AND view_status=1 AND password IS NULL",
    };
    sqlx::query_as::<_, Article>(sqlx::AssertSqlSafe(format!(
        "{ARTICLE_SQL} WHERE {condition}"
    )))
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(db_error)?
    .ok_or_else(absent)
}

pub(super) async fn public_article(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Article> {
    let article = load_article(&state.pool, id, ArticleAudience::Public).await?;
    sqlx::query("UPDATE article SET view_count=view_count+1 WHERE id=?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    Ok(Json(article))
}
#[derive(Serialize, sqlx::FromRow)]
pub(super) struct ArticleNews {
    id: i64,
    content: String,
    create_time: Option<String>,
}
const ARTICLE_NEWS_SQL: &str = "SELECT w.id,w.content,w.create_time FROM wei_yan w";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArticleNewsQuery {
    page: Option<i64>,
    size: Option<i64>,
}

pub(super) async fn article_news(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Vec<ArticleNews>> {
    let mut transaction = state.pool.begin().await.map_err(db_error)?;
    let visible: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM article WHERE id=? AND deleted=0 AND view_status=1 AND password IS NULL)",
    )
    .bind(id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(db_error)?;
    if !visible {
        return Err(absent());
    }
    let rows=sqlx::query_as::<_,ArticleNews>(sqlx::AssertSqlSafe(format!("{ARTICLE_NEWS_SQL} WHERE w.type='news' AND w.source=? AND w.is_public=1 ORDER BY w.create_time DESC,w.id DESC LIMIT 100")))
        .bind(id).fetch_all(&mut *transaction).await.map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(rows))
}
pub(super) async fn admin_article_news(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<ArticleNewsQuery>,
) -> ApiResult<PageResult<ArticleNews>> {
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(20).clamp(1, 100);
    let mut transaction = state.pool.begin().await.map_err(db_error)?;
    article_author(&mut transaction, id).await?;
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM wei_yan WHERE type='news' AND source=?")
            .bind(id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(db_error)?;
    let items = sqlx::query_as::<_, ArticleNews>(sqlx::AssertSqlSafe(format!("{ARTICLE_NEWS_SQL} WHERE w.type='news' AND w.source=? ORDER BY w.create_time DESC,w.id DESC LIMIT ? OFFSET ?")))
        .bind(id).bind(size).bind((page - 1) * size).fetch_all(&mut *transaction).await.map_err(db_error)?;
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
pub(super) struct ArticleNewsInput {
    content: ShortContent,
    create_time: Option<String>,
}
pub(super) async fn admin_create_article_news(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    crate::ContractJson(input): crate::ContractJson<ArticleNewsInput>,
) -> ApiResult<ArticleNews> {
    let request_id = IdempotencyKey::from_headers(&headers)?;
    Ok(Json(
        insert_article_news(&state, &identity, id, input, request_id).await?,
    ))
}
pub(super) async fn admin_delete_article_news(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path((article_id, news_id)): Path<(i64, i64)>,
) -> ApiResult<serde_json::Value> {
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let changed = sqlx::query("DELETE FROM wei_yan WHERE id=? AND source=? AND type='news'")
        .bind(news_id)
        .bind(article_id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(serde_json::json!({"deleted":true})))
}

pub(super) async fn admin_update_article_news(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path((article_id, news_id)): Path<(i64, i64)>,
    crate::ContractJson(input): crate::ContractJson<ArticleNewsInput>,
) -> ApiResult<ArticleNews> {
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    sqlx::query_scalar::<_, i64>("SELECT id FROM article WHERE id=? AND deleted=0")
        .bind(article_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(db_error)?
        .ok_or_else(absent)?;
    let time = normalize_news_time(input.create_time)?;
    let changed = sqlx::query("UPDATE wei_yan SET content=?,create_time=COALESCE(?,create_time) WHERE id=? AND source=? AND type='news'")
        .bind(input.content.as_str()).bind(time).bind(news_id).bind(article_id)
        .execute(&mut *transaction).await.map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    let result = sqlx::query_as::<_, ArticleNews>(sqlx::AssertSqlSafe(format!(
        "{ARTICLE_NEWS_SQL} WHERE w.id=? AND w.source=? AND w.type='news'"
    )))
    .bind(news_id)
    .bind(article_id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(result))
}

fn normalize_news_time(raw: Option<String>) -> Result<Option<String>, AppError> {
    use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Timelike, Utc};

    let Some(raw) = raw.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    if raw.len() > 64 {
        return Err(invalid("日期时间无效"));
    }
    let value = raw.trim().replacen(' ', "T", 1);
    // Only absolute calendar input is admitted. SQLite's datetime() also accepts
    // relative values such as "now", which make the same retry hash differently.
    let parsed = DateTime::parse_from_rfc3339(&value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
        .or_else(|| {
            ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M"]
                .into_iter()
                .find_map(|format| NaiveDateTime::parse_from_str(&value, format).ok())
                .map(|value| value.and_utc())
        })
        .or_else(|| {
            NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                .ok()
                .and_then(|value| value.and_hms_opt(0, 0, 0))
                .map(|value| value.and_utc())
        })
        .filter(|value| (0..=9999).contains(&value.year()) && value.nanosecond() < 1_000_000_000)
        .ok_or_else(|| invalid("日期时间无效"))?;
    Ok(Some(parsed.format("%Y-%m-%d %H:%M:%S").to_string()))
}

async fn article_author(connection: &mut sqlx::SqliteConnection, id: i64) -> Result<i64, AppError> {
    sqlx::query_scalar("SELECT user_id FROM article WHERE id=? AND deleted=0")
        .bind(id)
        .fetch_optional(connection)
        .await
        .map_err(db_error)?
        .ok_or_else(absent)
}

async fn insert_article_news(
    state: &AppState,
    identity: &AuthenticatedIdentity,
    id: i64,
    input: ArticleNewsInput,
    request_id: Option<IdempotencyKey<'_>>,
) -> Result<ArticleNews, AppError> {
    let content = input.content.as_str();
    let time = normalize_news_time(input.create_time)?;
    use sha2::{Digest, Sha256};
    // Bind the key to its article and normalized original draft, including time.
    let request_hash = Sha256::digest(
        serde_json::to_vec(&(id, content, &time))
            .expect("invariant: news strings and identifiers serialize to JSON"),
    )
    .to_vec();
    let mut transaction = crate::content::begin_mutation(&state.pool, identity).await?;
    let author = article_author(&mut transaction, id).await?;
    let previous: Option<(Vec<u8>, Option<i64>)> = if let Some(key) = &request_id {
        sqlx::query_as(
            "SELECT request_hash,news_id FROM article_news_submission WHERE request_id=?",
        )
        .bind(key.as_str())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(db_error)?
    } else {
        None
    };
    let news_id = if let Some((hash, news_id)) = previous {
        if hash != request_hash {
            return Err(AppError::Public(
                HttpStatus::Conflict,
                "进展请求标识已用于不同内容",
            ));
        }
        news_id.ok_or(AppError::Public(
            HttpStatus::Conflict,
            "原进展已删除，不会重复发表",
        ))?
    } else {
        let news_id = sqlx::query("INSERT INTO wei_yan(user_id,content,type,source,is_public,create_time) VALUES(?,?,'news',?,1,COALESCE(?,CURRENT_TIMESTAMP))")
            .bind(author).bind(content).bind(id).bind(time).execute(&mut *transaction).await.map_err(db_error)?.last_insert_rowid();
        if let Some(key) = request_id {
            sqlx::query("INSERT INTO article_news_submission(request_id,request_hash,news_id) VALUES(?,?,?)")
                .bind(key.as_str()).bind(request_hash).bind(news_id).execute(&mut *transaction).await.map_err(db_error)?;
        }
        news_id
    };
    let result = sqlx::query_as::<_, ArticleNews>(sqlx::AssertSqlSafe(format!(
        "{ARTICLE_NEWS_SQL} WHERE w.id=? AND w.source=? AND w.type='news'"
    )))
    .bind(news_id)
    .bind(id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(result)
}
#[derive(Serialize, sqlx::FromRow)]
pub(super) struct ArticleAccess {
    password_required: i64,
    tips: Option<String>,
}
pub(super) async fn article_access(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<ArticleAccess> {
    Ok(Json(sqlx::query_as::<_,ArticleAccess>("SELECT CASE WHEN password IS NULL THEN 0 ELSE 1 END AS password_required,tips FROM article WHERE id=? AND deleted=0 AND (view_status=1 OR password IS NOT NULL)")
        .bind(id).fetch_optional(&state.pool).await.map_err(db_error)?.ok_or_else(absent)?))
}
pub(super) async fn admin_article(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Article> {
    Ok(Json(
        load_article(&state.pool, id, ArticleAudience::Administrator).await?,
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct UnlockRequest {
    password: String,
}
pub(super) async fn unlock_article(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<UnlockRequest>,
) -> ApiResult<Article> {
    if input.password.len() > 1024 {
        return Err(invalid("访问密码无效"));
    }
    let stored: String = sqlx::query_scalar(
        "SELECT password FROM article WHERE id=? AND deleted=0 AND password IS NOT NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or_else(absent)?;
    let permit = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        std::sync::Arc::clone(&state.password_slots).acquire_owned(),
    )
    .await
    .map_err(|source| AppError::Busy {
        message: "密码验证暂时繁忙",
        source,
    })?
    .expect("invariant: password semaphore is never closed");
    let password = input.password;
    // The permit remains held by the worker even if the HTTP request is cancelled.
    let (allowed, stored) = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        (
            blog_application::admin_auth::verify_password(&password, &stored),
            stored,
        )
    })
    .await
    .map_err(|source| AppError::Task {
        message: "密码验证失败",
        source,
    })?;
    if !allowed {
        return Err(AppError::Public(HttpStatus::Forbidden, "访问密码错误"));
    }
    // Recheck access after waiting/hashing, and read the body under the same
    // writer boundary so a concurrent password change cannot expose new content.
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let changed = sqlx::query(
        "UPDATE article SET view_count=view_count+1 WHERE id=? AND deleted=0 AND password=?",
    )
    .bind(id)
    .bind(stored)
    .execute(&mut *transaction)
    .await
    .map_err(db_error)?;
    if changed.rows_affected() != 1 {
        return Err(AppError::Public(
            HttpStatus::Forbidden,
            "文章访问权限已变更，请重试",
        ));
    }
    let article = sqlx::query_as::<_, Article>(sqlx::AssertSqlSafe(format!(
        "{ARTICLE_SQL} WHERE id=? AND deleted=0"
    )))
    .bind(id)
    .fetch_one(&mut *transaction)
    .await
    .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(article))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArticleInputDto {
    article_title: String,
    article_content: String,
    article_cover: Option<String>,
    video_url: Option<String>,
    sort_id: i64,
    label_id: i64,
    view_status: bool,
    recommend_status: bool,
    comment_status: bool,
    password: Option<String>,
    tips: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(try_from = "ArticleInputDto")]
pub(super) struct ArticleInput {
    article_title: String,
    article_content: String,
    article_cover: Option<String>,
    video_url: Option<String>,
    sort_id: i64,
    label_id: i64,
    view_status: bool,
    recommend_status: bool,
    comment_status: bool,
    password: Option<String>,
    tips: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum ArticleInputError {
    #[error("文章标题无效")]
    Title,
    #[error("文章内容无效")]
    Content,
    #[error("请选择分类和标签")]
    Taxonomy,
    #[error("公开文章不可设置访问密码")]
    PublicPassword,
}

impl TryFrom<ArticleInputDto> for ArticleInput {
    type Error = ArticleInputError;

    fn try_from(input: ArticleInputDto) -> Result<Self, Self::Error> {
        let ArticleInputDto {
            article_title,
            article_content,
            article_cover,
            video_url,
            sort_id,
            label_id,
            view_status,
            recommend_status,
            comment_status,
            password,
            tips,
        } = input;
        if article_title.trim().is_empty() || article_title.chars().count() > 120 {
            return Err(ArticleInputError::Title);
        }
        if article_content.trim().is_empty() || article_content.len() > 2_000_000 {
            return Err(ArticleInputError::Content);
        }
        if sort_id < 1 || label_id < 1 {
            return Err(ArticleInputError::Taxonomy);
        }
        if view_status && password.is_some() {
            return Err(ArticleInputError::PublicPassword);
        }
        Ok(Self {
            article_title,
            article_content,
            article_cover,
            video_url,
            sort_id,
            label_id,
            view_status,
            recommend_status,
            comment_status,
            password,
            tips,
        })
    }
}

// Serialize taxonomy validation with label moves/deletes and category deletion.
// An article must never commit a label from another (or a missing) category.
async fn validate_article_taxonomy(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    input: &ArticleInput,
) -> Result<(), AppError> {
    let matches: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM label l JOIN sort s ON s.id=l.sort_id WHERE l.id=? AND s.id=?)",
    )
    .bind(input.label_id)
    .bind(input.sort_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(db_error)?;
    if !matches {
        return Err(invalid("标签不属于所选分类，请重新选择分类和标签"));
    }
    Ok(())
}

pub(super) async fn create_article(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    headers: HeaderMap,
    crate::ContractJson(input): crate::ContractJson<ArticleInput>,
) -> ApiResult<Article> {
    if !input.view_status && input.password.is_none() {
        return Err(invalid("加密文章必须设置访问密码"));
    }
    let request_id = IdempotencyKey::from_headers(&headers)?;
    use sha2::{Digest, Sha256};
    let request_hash = Sha256::digest(
        serde_json::to_vec(&input).expect("invariant: article strings and flags serialize to JSON"),
    )
    .to_vec();
    let password = input
        .password
        .as_deref()
        .map(Password::new)
        .transpose()
        .map_err(AppError::InvalidPassword)?;
    let password = hash_article_password(&state, password).await?;
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    if let Some(key) = &request_id {
        let receipt: Option<(Vec<u8>, i64, i64)> = sqlx::query_as("SELECT s.request_hash,s.article_id,a.deleted FROM article_submission s JOIN article a ON a.id=s.article_id WHERE s.request_id=?")
            .bind(key.as_str()).fetch_optional(&mut *transaction).await.map_err(db_error)?;
        if let Some((hash, id, deleted)) = receipt {
            if hash != request_hash || deleted != 0 {
                return Err(AppError::Public(
                    HttpStatus::Conflict,
                    "文章请求已使用或文章已删除",
                ));
            }
            transaction.commit().await.map_err(db_error)?;
            return Ok(Json(
                load_article(&state.pool, id, ArticleAudience::Administrator).await?,
            ));
        }
    }
    let author = crate::site_author_id(&mut *transaction).await?;
    validate_article_taxonomy(&mut transaction, &input).await?;
    let id = sqlx::query("INSERT INTO article(user_id,sort_id,label_id,article_cover,article_title,article_content,video_url,view_status,recommend_status,comment_status,password,tips) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(author)
        .bind(input.sort_id).bind(input.label_id).bind(input.article_cover).bind(input.article_title).bind(input.article_content).bind(input.video_url)
        .bind(i64::from(input.view_status)).bind(i64::from(input.recommend_status)).bind(i64::from(input.comment_status)).bind(password).bind(input.tips)
        .execute(&mut *transaction).await.map_err(db_error)?.last_insert_rowid();
    if let Some(key) = request_id {
        sqlx::query(
            "INSERT INTO article_submission(request_id,request_hash,article_id) VALUES(?,?,?)",
        )
        .bind(key.as_str())
        .bind(request_hash)
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    }
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(
        load_article(&state.pool, id, ArticleAudience::Administrator).await?,
    ))
}

async fn hash_article_password(
    state: &AppState,
    password: Option<Password>,
) -> Result<Option<String>, AppError> {
    let Some(password) = password else {
        return Ok(None);
    };
    let permit = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        std::sync::Arc::clone(&state.password_slots).acquire_owned(),
    )
    .await
    .map_err(|source| AppError::Busy {
        message: "密码计算暂时繁忙",
        source,
    })?
    .expect("invariant: password semaphore is never closed");
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        password.hash()
    })
    .await
    .map_err(|source| AppError::Task {
        message: "密码计算失败",
        source,
    })?
    .map(Some)
    .map_err(AppError::Password)
}

pub(super) async fn update_article(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<ArticleInput>,
) -> ApiResult<Article> {
    let password = input
        .password
        .as_deref()
        .map(Password::new)
        .transpose()
        .map_err(AppError::InvalidPassword)?;
    let password = hash_article_password(&state, password).await?;
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let has_password: bool =
        sqlx::query_scalar("SELECT password IS NOT NULL FROM article WHERE id=? AND deleted=0")
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(db_error)?
            .ok_or_else(absent)?;
    if !input.view_status && input.password.is_none() && !has_password {
        return Err(invalid("加密文章必须设置访问密码"));
    }
    validate_article_taxonomy(&mut transaction, &input).await?;

    sqlx::query("UPDATE article SET sort_id=?,label_id=?,article_cover=?,article_title=?,article_content=?,video_url=?,view_status=?,recommend_status=?,comment_status=?,password=CASE WHEN ?=1 THEN NULL ELSE COALESCE(?,password) END,tips=?,update_time=CURRENT_TIMESTAMP WHERE id=? AND deleted=0")
        .bind(input.sort_id).bind(input.label_id).bind(input.article_cover).bind(input.article_title).bind(input.article_content).bind(input.video_url)
        .bind(i64::from(input.view_status)).bind(i64::from(input.recommend_status)).bind(i64::from(input.comment_status)).bind(i64::from(input.view_status)).bind(password).bind(input.tips).bind(id)
        .execute(&mut *transaction).await.map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(
        load_article(&state.pool, id, ArticleAudience::Administrator).await?,
    ))
}

pub(super) async fn delete_article(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let result = sqlx::query(
        "UPDATE article SET deleted=1,update_time=CURRENT_TIMESTAMP WHERE id=? AND deleted=0",
    )
    .bind(id)
    .execute(&mut *transaction)
    .await
    .map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(absent());
    }
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(serde_json::json!({"deleted":true})))
}

#[path = "../../../../test/server/unit/content/articles/pagination_tests.rs"]
#[cfg(test)]
mod pagination_tests;
