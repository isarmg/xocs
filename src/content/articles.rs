use super::PageResult;
use crate::rate_limit;
use crate::{ApiResult, AppError, AppState, absent, db_error, invalid};
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use xcss::server_cli::{ContractPath as Path, ContractQuery as Query};

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

async fn list_articles(
    pool: &SqlitePool,
    query: ArticleQuery,
    public: bool,
) -> Result<PageResult<ArticleSummary>, AppError> {
    let page = query.page.unwrap_or(1).clamp(1, 100_000);
    let size = query.size.unwrap_or(12).clamp(1, 100);
    let search = query.search.unwrap_or_default().trim().to_owned();
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
    let predicate = if public {
        "deleted=0 AND (view_status=1 OR password IS NOT NULL)"
    } else {
        "deleted=0"
    };
    let recommended = query.recommended.map(i64::from);
    let phrase = format!("\"{}\"", search.replace('"', "\"\""));
    let body_match = if public {
        "(password IS NULL AND article_content LIKE ?3 ESCAPE '\\')"
    } else {
        "article_content LIKE ?3 ESCAPE '\\'"
    };
    let filter = format!(
        "{predicate} AND (?1 IS NULL OR sort_id=?1) AND (?6 IS NULL OR label_id=?6) AND (?4 IS NULL OR recommend_status=?4) AND (?2='' OR ((article_title LIKE ?3 ESCAPE '\\' OR {body_match}) AND (length(?2)<3 OR id IN (SELECT rowid FROM article_search WHERE article_search MATCH ?5))))"
    );
    let count_sql = format!("SELECT COUNT(*) FROM article WHERE {filter}");
    let total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(count_sql))
        .bind(query.sort_id)
        .bind(&search)
        .bind(&pattern)
        .bind(recommended)
        .bind(&phrase)
        .bind(query.label_id)
        .fetch_one(pool)
        .await
        .map_err(db_error)?;
    let list_sql = format!(
        "{ARTICLE_SUMMARY_SQL} WHERE {filter} ORDER BY CASE WHEN ?2<>'' AND article_title LIKE ?3 ESCAPE '\\' THEN 0 ELSE 1 END,create_time DESC,id DESC LIMIT ?7 OFFSET ?8"
    );
    let items = sqlx::query_as::<_, ArticleSummary>(sqlx::AssertSqlSafe(list_sql))
        .bind(query.sort_id)
        .bind(&search)
        .bind(&pattern)
        .bind(recommended)
        .bind(&phrase)
        .bind(query.label_id)
        .bind(size)
        .bind((page - 1) * size)
        .fetch_all(pool)
        .await
        .map_err(db_error)?;
    Ok(PageResult {
        items,
        total,
        page,
        size,
    })
}

pub(super) async fn public_articles(
    State(state): State<AppState>,
    Query(query): Query<ArticleQuery>,
) -> ApiResult<PageResult<ArticleSummary>> {
    Ok(Json(list_articles(&state.pool, query, true).await?))
}
pub(super) async fn admin_articles(
    State(state): State<AppState>,
    Query(query): Query<ArticleQuery>,
) -> ApiResult<PageResult<ArticleSummary>> {
    Ok(Json(list_articles(&state.pool, query, false).await?))
}

async fn load_article(pool: &SqlitePool, id: i64, admin: bool) -> Result<Article, AppError> {
    let condition = if admin {
        "id=? AND deleted=0"
    } else {
        "id=? AND deleted=0 AND view_status=1 AND password IS NULL"
    };
    sqlx::query_as::<_, Article>(sqlx::AssertSqlSafe(format!(
        "{ARTICLE_SQL} WHERE {condition}"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?
    .ok_or_else(absent)
}

pub(super) async fn public_article(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Article> {
    let article = load_article(&state.pool, id, false).await?;
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
pub(super) async fn article_news(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Vec<ArticleNews>> {
    load_article(&state.pool, id, false).await?;
    let rows=sqlx::query_as::<_,ArticleNews>(sqlx::AssertSqlSafe(format!("{ARTICLE_NEWS_SQL} WHERE w.type='news' AND w.source=? AND w.is_public=1 ORDER BY w.create_time DESC,w.id DESC LIMIT 100")))
        .bind(id).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(rows))
}
pub(super) async fn admin_article_news(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<Vec<ArticleNews>> {
    load_article(&state.pool, id, true).await?;
    let rows=sqlx::query_as::<_,ArticleNews>(sqlx::AssertSqlSafe(format!("{ARTICLE_NEWS_SQL} WHERE w.type='news' AND w.source=? ORDER BY w.create_time DESC,w.id DESC LIMIT 100")))
        .bind(id).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(rows))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArticleNewsInput {
    content: String,
    create_time: Option<String>,
}
pub(super) async fn admin_create_article_news(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<ArticleNewsInput>,
) -> ApiResult<ArticleNews> {
    let article = load_article(&state.pool, id, true).await?;
    Ok(Json(
        insert_article_news(&state, id, article.user_id, input).await?,
    ))
}
pub(super) async fn admin_delete_article_news(
    State(state): State<AppState>,
    Path((article_id, news_id)): Path<(i64, i64)>,
) -> ApiResult<serde_json::Value> {
    let changed = sqlx::query("DELETE FROM wei_yan WHERE id=? AND source=? AND type='news'")
        .bind(news_id)
        .bind(article_id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(serde_json::json!({"deleted":true})))
}
async fn insert_article_news(
    state: &AppState,
    id: i64,
    author: i64,
    input: ArticleNewsInput,
) -> Result<ArticleNews, AppError> {
    let content = input.content.trim();
    if content.is_empty() || content.chars().count() > 1024 {
        return Err(invalid("进展内容应为 1 至 1024 字"));
    }
    let time = if let Some(raw) = input.create_time.filter(|value| !value.trim().is_empty()) {
        let parsed: Option<String> = sqlx::query_scalar("SELECT datetime(?)")
            .bind(&raw)
            .fetch_one(&state.pool)
            .await
            .map_err(db_error)?;
        Some(parsed.ok_or_else(|| invalid("日期时间无效"))?)
    } else {
        None
    };
    let id_new=sqlx::query("INSERT INTO wei_yan(user_id,content,type,source,is_public,create_time) VALUES(?,?,'news',?,1,COALESCE(?,CURRENT_TIMESTAMP))")
        .bind(author).bind(content).bind(id).bind(time).execute(&state.pool).await.map_err(db_error)?.last_insert_rowid();
    sqlx::query_as::<_, ArticleNews>(sqlx::AssertSqlSafe(format!(
        "{ARTICLE_NEWS_SQL} WHERE w.id=?"
    )))
    .bind(id_new)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)
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
    Ok(Json(load_article(&state.pool, id, true).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct UnlockRequest {
    password: String,
}
pub(super) async fn unlock_article(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<UnlockRequest>,
) -> ApiResult<Article> {
    if input.password.len() > 1024 {
        return Err(invalid("访问密码无效"));
    }
    let stored: Option<String> = sqlx::query_scalar(
        "SELECT password FROM article WHERE id=? AND deleted=0 AND password IS NOT NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(db_error)?
    .ok_or_else(absent)?;
    let key = rate_limit::attempt_key("article", &peer.ip().to_string(), &id.to_string());
    rate_limit::record_attempt(&state.pool, &key, 10)
        .await
        .map_err(|error| {
            if error.0 == StatusCode::TOO_MANY_REQUESTS {
                AppError(
                    StatusCode::TOO_MANY_REQUESTS,
                    "访问密码尝试过多，请稍后再试",
                )
            } else {
                error
            }
        })?;
    let allowed = stored
        .as_deref()
        .is_none_or(|hash| xcss::admin_auth::verify_password(&input.password, hash));
    if !allowed {
        return Err(AppError(StatusCode::FORBIDDEN, "访问密码错误"));
    }
    sqlx::query("DELETE FROM member_login_failures WHERE failure_key=?")
        .bind(&key)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    sqlx::query("UPDATE article SET view_count=view_count+1 WHERE id=?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    Ok(Json(load_article(&state.pool, id, true).await?))
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
fn validate_article(input: &ArticleInput, has_existing_password: bool) -> Result<(), AppError> {
    if input.article_title.trim().is_empty() || input.article_title.chars().count() > 120 {
        return Err(invalid("文章标题无效"));
    }
    if input.article_content.trim().is_empty() || input.article_content.len() > 2_000_000 {
        return Err(invalid("文章内容无效"));
    }
    if input.sort_id < 1 || input.label_id < 1 {
        return Err(invalid("请选择分类和标签"));
    }
    if !input.view_status && input.password.is_none() && !has_existing_password {
        return Err(invalid("加密文章必须设置访问密码"));
    }
    if input.view_status && input.password.is_some() {
        return Err(invalid("公开文章不可设置访问密码"));
    }
    if let Some(password) = &input.password {
        xcss::admin_auth::validate_password(password)
            .map_err(|_| invalid("文章密码至少需要 12 字节"))?;
    }
    Ok(())
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
    headers: HeaderMap,
    crate::ContractJson(input): crate::ContractJson<ArticleInput>,
) -> ApiResult<Article> {
    validate_article(&input, false)?;
    let request_id = headers
        .get("Idempotency-Key")
        .map(|value| {
            value
                .to_str()
                .ok()
                .filter(|key| {
                    !key.is_empty()
                        && key.len() <= 128
                        && key.bytes().all(|byte| byte.is_ascii_graphic())
                })
                .map(str::to_owned)
                .ok_or_else(|| invalid("文章请求标识无效"))
        })
        .transpose()?;
    use sha2::{Digest, Sha256};
    let request_hash =
        Sha256::digest(serde_json::to_vec(&input).map_err(|_| invalid("文章内容无效"))?).to_vec();
    let password = input
        .password
        .as_deref()
        .map(xcss::admin_auth::hash_password)
        .transpose()
        .map_err(|_| invalid("文章密码无效"))?;
    let author = crate::site_author_id(&state.pool).await?;
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    if let Some(key) = &request_id {
        let receipt: Option<(Vec<u8>, i64, i64)> = sqlx::query_as("SELECT s.request_hash,s.article_id,a.deleted FROM article_submission s JOIN article a ON a.id=s.article_id WHERE s.request_id=?")
            .bind(key).fetch_optional(&mut *transaction).await.map_err(db_error)?;
        if let Some((hash, id, deleted)) = receipt {
            if hash != request_hash || deleted != 0 {
                return Err(AppError(StatusCode::CONFLICT, "文章请求已使用或文章已删除"));
            }
            transaction.commit().await.map_err(db_error)?;
            return Ok(Json(load_article(&state.pool, id, true).await?));
        }
    }
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
        .bind(key)
        .bind(request_hash)
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    }
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(load_article(&state.pool, id, true).await?))
}

pub(super) async fn update_article(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<ArticleInput>,
) -> ApiResult<Article> {
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let old_password: Option<Option<String>> =
        sqlx::query_scalar("SELECT password FROM article WHERE id=? AND deleted=0")
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(db_error)?;
    let Some(old_password) = old_password else {
        return Err(absent());
    };
    validate_article(&input, old_password.is_some())?;
    validate_article_taxonomy(&mut transaction, &input).await?;
    let password = input
        .password
        .as_deref()
        .map(xcss::admin_auth::hash_password)
        .transpose()
        .map_err(|_| invalid("文章密码无效"))?;
    let result = sqlx::query("UPDATE article SET sort_id=?,label_id=?,article_cover=?,article_title=?,article_content=?,video_url=?,view_status=?,recommend_status=?,comment_status=?,password=CASE WHEN ?=1 THEN NULL ELSE COALESCE(?,password) END,tips=?,update_time=CURRENT_TIMESTAMP WHERE id=? AND deleted=0")
        .bind(input.sort_id).bind(input.label_id).bind(input.article_cover).bind(input.article_title).bind(input.article_content).bind(input.video_url)
        .bind(i64::from(input.view_status)).bind(i64::from(input.recommend_status)).bind(i64::from(input.comment_status)).bind(i64::from(input.view_status)).bind(password).bind(input.tips).bind(id)
        .execute(&mut *transaction).await.map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(absent());
    }
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(load_article(&state.pool, id, true).await?))
}

pub(super) async fn delete_article(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let result = sqlx::query(
        "UPDATE article SET deleted=1,update_time=CURRENT_TIMESTAMP WHERE id=? AND deleted=0",
    )
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(db_error)?;
    if result.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(serde_json::json!({"deleted":true})))
}
