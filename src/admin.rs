use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use xcss::server_cli::{ContractPath as Path, ContractQuery as Query};

use crate::{ApiResult, AppError, AppState, absent, db_error, invalid};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/content/users", get(users))
        .route(
            "/api/v1/content/users/{id}/status",
            axum::routing::put(change_user_status),
        )
        .route("/api/v1/content/comments", get(comments))
        .route(
            "/api/v1/content/comments/{id}",
            axum::routing::delete(delete_comment),
        )
        .route("/api/v1/content/site", get(site).put(update_site))
        .route("/api/v1/content/labels", get(labels).post(create_label))
        .route(
            "/api/v1/content/labels/{id}",
            axum::routing::put(update_label).delete(delete_label),
        )
        .route("/api/v1/content/statistics", get(statistics))
        .route("/api/v1/content/resources", get(resources))
        .route(
            "/api/v1/content/links",
            get(content_links).post(create_link),
        )
        .route(
            "/api/v1/content/links/{id}",
            axum::routing::put(update_link).delete(delete_link),
        )
        .route(
            "/api/v1/content/links/{id}/status",
            axum::routing::put(change_link_status),
        )
        .route("/api/v1/content/tree-hole", get(content_tree_hole))
        .route(
            "/api/v1/content/notes",
            get(content_notes).post(create_admin_note),
        )
        .route(
            "/api/v1/content/notes/{id}",
            axum::routing::delete(delete_note),
        )
        .route(
            "/api/v1/content/tree-hole/{id}",
            axum::routing::delete(delete_tree_hole),
        )
        .route(
            "/api/v1/content/family",
            get(content_family).post(create_family),
        )
        .route(
            "/api/v1/content/family/{id}",
            axum::routing::put(update_family).delete(delete_family),
        )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageQuery {
    page: Option<i64>,
    size: Option<i64>,
}
#[derive(Serialize)]
struct Page<T> {
    items: Vec<T>,
    total: i64,
    page: i64,
    size: i64,
}
fn page(query: PageQuery) -> (i64, i64) {
    (
        query.page.unwrap_or(1).clamp(1, 100_000),
        query.size.unwrap_or(20).clamp(1, 100),
    )
}

#[derive(Serialize, sqlx::FromRow)]
struct User {
    id: i64,
    username: Option<String>,
    email: Option<String>,
    phone_number: Option<String>,
    user_status: i64,
    user_type: i64,
    create_time: Option<String>,
}
async fn users(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> ApiResult<Page<User>> {
    let (page, size) = page(query);
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user WHERE deleted=0")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let items=sqlx::query_as::<_,User>("SELECT id,username,email,phone_number,user_status,user_type,create_time FROM user WHERE deleted=0 ORDER BY id DESC LIMIT ? OFFSET ?")
        .bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(Page {
        items,
        total,
        page,
        size,
    }))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusInput {
    active: bool,
}
async fn change_user_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<StatusInput>,
) -> ApiResult<serde_json::Value> {
    if !input.active {
        let kind: Option<i64> =
            sqlx::query_scalar("SELECT user_type FROM user WHERE id=? AND deleted=0")
                .bind(id)
                .fetch_optional(&state.pool)
                .await
                .map_err(db_error)?;
        if kind == Some(0) {
            let others:i64=sqlx::query_scalar("SELECT COUNT(*) FROM user WHERE user_type=0 AND user_status=1 AND deleted=0 AND id<>?")
                .bind(id).fetch_one(&state.pool).await.map_err(db_error)?;
            if others == 0 {
                return Err(AppError(StatusCode::CONFLICT, "不能停用唯一的站点作者"));
            }
        }
    }
    let changed = sqlx::query(
        "UPDATE user SET user_status=?,update_time=CURRENT_TIMESTAMP WHERE id=? AND deleted=0",
    )
    .bind(i64::from(input.active))
    .bind(id)
    .execute(&state.pool)
    .await
    .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(serde_json::json!({"active":input.active})))
}

#[derive(Serialize, sqlx::FromRow)]
struct Comment {
    id: i64,
    source: i64,
    comment_type: Option<String>,
    user_id: Option<i64>,
    username: Option<String>,
    comment_content: String,
    create_time: Option<String>,
}
async fn comments(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> ApiResult<Page<Comment>> {
    let (page, size) = page(query);
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comment")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let items=sqlx::query_as::<_,Comment>("SELECT c.id,c.source,c.type AS comment_type,c.user_id,u.username,c.comment_content,c.create_time FROM comment c LEFT JOIN user u ON u.id=c.user_id ORDER BY c.id DESC LIMIT ? OFFSET ?")
        .bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(Page {
        items,
        total,
        page,
        size,
    }))
}
async fn delete_comment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let root: Option<Option<i64>> =
        sqlx::query_scalar("SELECT parent_comment_id FROM comment WHERE id=?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(db_error)?;
    let Some(parent) = root else {
        return Err(absent());
    };
    if parent.unwrap_or(0) == 0 {
        sqlx::query("DELETE FROM comment WHERE id=? OR floor_comment_id=?")
            .bind(id)
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(db_error)?;
    } else {
        sqlx::query("DELETE FROM comment WHERE id=?")
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(db_error)?;
    }
    Ok(Json(serde_json::json!({"deleted":true})))
}

#[derive(Serialize, sqlx::FromRow)]
struct Site {
    web_name: Option<String>,
    web_title: Option<String>,
    notices: Option<String>,
    footer: Option<String>,
    background_image: Option<String>,
    avatar: Option<String>,
    random_avatar: Option<String>,
    random_name: Option<String>,
    random_cover: Option<String>,
    waifu_json: Option<String>,
}
async fn site(State(state): State<AppState>) -> ApiResult<Site> {
    Ok(Json(sqlx::query_as::<_,Site>("SELECT web_name,web_title,notices,footer,background_image,avatar,random_avatar,random_name,random_cover,waifu_json FROM web_info ORDER BY id LIMIT 1")
        .fetch_optional(&state.pool).await.map_err(db_error)?.unwrap_or(Site{web_name:None,web_title:None,notices:None,footer:None,background_image:None,avatar:None,random_avatar:None,random_name:None,random_cover:None,waifu_json:None})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SiteInput {
    web_name: String,
    web_title: Option<String>,
    notices: Option<String>,
    footer: Option<String>,
    background_image: Option<String>,
    avatar: Option<String>,
    random_avatar: Option<String>,
    random_name: Option<String>,
    random_cover: Option<String>,
    waifu_json: Option<String>,
}
async fn update_site(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<SiteInput>,
) -> ApiResult<Site> {
    if input.web_name.trim().is_empty()
        || input.web_name.chars().count() > 100
        || input
            .web_title
            .as_deref()
            .is_some_and(|v| v.chars().count() > 200)
        || input.footer.as_deref().is_some_and(|v| v.len() > 4096)
        || [
            &input.notices,
            &input.random_avatar,
            &input.random_name,
            &input.random_cover,
            &input.waifu_json,
        ]
        .iter()
        .any(|item| item.as_deref().is_some_and(|v| v.len() > 16384))
    {
        return Err(invalid("站点信息无效"));
    }
    let id: Option<i64> = sqlx::query_scalar("SELECT id FROM web_info ORDER BY id LIMIT 1")
        .fetch_optional(&state.pool)
        .await
        .map_err(db_error)?;
    if let Some(id) = id {
        sqlx::query("UPDATE web_info SET web_name=?,web_title=?,notices=?,footer=?,background_image=?,avatar=?,random_avatar=?,random_name=?,random_cover=?,waifu_json=? WHERE id=?")
            .bind(input.web_name).bind(input.web_title).bind(input.notices).bind(input.footer).bind(input.background_image).bind(input.avatar).bind(input.random_avatar).bind(input.random_name).bind(input.random_cover).bind(input.waifu_json).bind(id)
            .execute(&state.pool).await.map_err(db_error)?;
    } else {
        sqlx::query("INSERT INTO web_info(web_name,web_title,notices,footer,background_image,avatar,random_avatar,random_name,random_cover,waifu_json,status) VALUES(?,?,?,?,?,?,?,?,?,?,1)")
            .bind(input.web_name).bind(input.web_title).bind(input.notices).bind(input.footer).bind(input.background_image).bind(input.avatar).bind(input.random_avatar).bind(input.random_name).bind(input.random_cover).bind(input.waifu_json)
            .execute(&state.pool).await.map_err(db_error)?;
    }
    site(State(state)).await
}

#[derive(Serialize, sqlx::FromRow)]
struct Label {
    id: i64,
    sort_id: i64,
    label_name: String,
    label_description: Option<String>,
}
async fn labels(State(state): State<AppState>) -> ApiResult<Vec<Label>> {
    Ok(Json(
        sqlx::query_as::<_, Label>(
            "SELECT id,sort_id,label_name,label_description FROM label ORDER BY sort_id,id",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelInput {
    sort_id: i64,
    name: String,
    description: Option<String>,
}
async fn create_label(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<LabelInput>,
) -> ApiResult<Label> {
    if input.name.trim().is_empty() || input.name.chars().count() > 32 {
        return Err(invalid("标签名称无效"));
    }
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM sort WHERE id=?")
        .bind(input.sort_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(db_error)?;
    if exists.is_none() {
        return Err(invalid("分类不存在"));
    }
    let id = sqlx::query("INSERT INTO label(sort_id,label_name,label_description) VALUES(?,?,?)")
        .bind(input.sort_id)
        .bind(input.name)
        .bind(input.description)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?
        .last_insert_rowid();
    let label =
        sqlx::query_as("SELECT id,sort_id,label_name,label_description FROM label WHERE id=?")
            .bind(id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(label))
}
async fn update_label(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<LabelInput>,
) -> ApiResult<Label> {
    if input.name.trim().is_empty() || input.name.chars().count() > 32 {
        return Err(invalid("标签名称无效"));
    }
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let previous: i64 = sqlx::query_scalar("SELECT sort_id FROM label WHERE id=?")
        .bind(id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(db_error)?
        .ok_or_else(absent)?;
    if previous != input.sort_id {
        let used: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM article WHERE label_id=? AND deleted=0)",
        )
        .bind(id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(db_error)?;
        if used {
            return Err(AppError(
                StatusCode::CONFLICT,
                "标签仍被文章使用，不能更改分类；请先修改文章的分类和标签",
            ));
        }
    }
    let category: Option<i64> = sqlx::query_scalar("SELECT id FROM sort WHERE id=?")
        .bind(input.sort_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(db_error)?;
    if category.is_none() {
        return Err(invalid("分类不存在"));
    }
    let changed =
        sqlx::query("UPDATE label SET sort_id=?,label_name=?,label_description=? WHERE id=?")
            .bind(input.sort_id)
            .bind(input.name.trim())
            .bind(input.description)
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    let label =
        sqlx::query_as("SELECT id,sort_id,label_name,label_description FROM label WHERE id=?")
            .bind(id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(db_error)?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(label))
}
async fn delete_label(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let mut transaction = state
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(db_error)?;
    let used: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM article WHERE label_id=? AND deleted=0")
            .bind(id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(db_error)?;
    if used > 0 {
        return Err(AppError(StatusCode::CONFLICT, "标签中还有文章"));
    }
    let changed = sqlx::query("DELETE FROM label WHERE id=?")
        .bind(id)
        .execute(&mut *transaction)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(serde_json::json!({"deleted":true})))
}

#[derive(Serialize)]
struct Statistics {
    articles: i64,
    comments: i64,
    members: i64,
    categories: i64,
}
async fn statistics(State(state): State<AppState>) -> ApiResult<Statistics> {
    let articles: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM article WHERE deleted=0")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let comments: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comment")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let members: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user WHERE deleted=0")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let categories: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sort")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    Ok(Json(Statistics {
        articles,
        comments,
        members,
        categories,
    }))
}

#[derive(Serialize, sqlx::FromRow)]
struct Resource {
    id: i64,
    original_name: Option<String>,
    path: Option<String>,
    resource_type: Option<String>,
    size: Option<i64>,
    status: Option<i64>,
    create_time: Option<String>,
}
async fn resources(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> ApiResult<Page<Resource>> {
    let (page, size) = page(query);
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let items=sqlx::query_as::<_,Resource>("SELECT id,original_name,path,type AS resource_type,size,status,create_time FROM resource ORDER BY id DESC LIMIT ? OFFSET ?")
        .bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(Page {
        items,
        total,
        page,
        size,
    }))
}

#[derive(Serialize, sqlx::FromRow)]
struct Link {
    id: i64,
    title: Option<String>,
    classify: Option<String>,
    cover: Option<String>,
    url: Option<String>,
    introduction: Option<String>,
    link_type: Option<String>,
    status: Option<i64>,
    create_time: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkQuery {
    kind: Option<String>,
    status: Option<i64>,
    page: Option<i64>,
    size: Option<i64>,
}
async fn content_links(
    State(state): State<AppState>,
    Query(query): Query<LinkQuery>,
) -> ApiResult<Page<Link>> {
    let (page, size) = page(PageQuery {
        page: query.page,
        size: query.size,
    });
    let kind = query.kind.unwrap_or_default();
    if kind.len() > 32 {
        return Err(invalid("资源类型无效"));
    }
    if query.status.is_some_and(|status| !matches!(status, 0 | 1)) {
        return Err(invalid("资源状态无效"));
    }
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM resource_path WHERE (?='' OR type=?) AND (? IS NULL OR status=?)",
    )
    .bind(&kind)
    .bind(&kind)
    .bind(query.status)
    .bind(query.status)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?;
    let items = sqlx::query_as::<_, Link>("SELECT id,title,classify,cover,url,introduction,type AS link_type,status,create_time FROM resource_path WHERE (?='' OR type=?) AND (? IS NULL OR status=?) ORDER BY id DESC LIMIT ? OFFSET ?")
        .bind(&kind).bind(&kind).bind(query.status).bind(query.status).bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(Page {
        items,
        total,
        page,
        size,
    }))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkInput {
    title: String,
    classify: Option<String>,
    cover: Option<String>,
    url: Option<String>,
    introduction: Option<String>,
    link_type: String,
    status: bool,
}
fn validate_link(input: &LinkInput) -> Result<(), AppError> {
    if input.title.trim().is_empty()
        || input.title.chars().count() > 64
        || !matches!(
            input.link_type.as_str(),
            "friendUrl" | "favorites" | "lovePhoto" | "funny"
        )
        || input
            .classify
            .as_deref()
            .is_some_and(|v| v.chars().count() > 32)
        || input
            .introduction
            .as_deref()
            .is_some_and(|v| v.chars().count() > 1024)
        || input.cover.as_deref().is_some_and(|v| {
            v.len() > 2048
                || !(v.starts_with("https://") || v.starts_with("http://") || v.starts_with('/'))
        })
        || input.url.as_deref().is_some_and(|v| {
            v.len() > 2048 || !(v.starts_with("https://") || v.starts_with("http://"))
        })
    {
        return Err(invalid("资源内容无效"));
    }
    Ok(())
}
async fn load_link(state: &AppState, id: i64) -> ApiResult<Link> {
    Ok(Json(sqlx::query_as("SELECT id,title,classify,cover,url,introduction,type AS link_type,status,create_time FROM resource_path WHERE id=?")
        .bind(id).fetch_optional(&state.pool).await.map_err(db_error)?.ok_or_else(absent)?))
}
async fn create_link(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<LinkInput>,
) -> ApiResult<Link> {
    validate_link(&input)?;
    let id = sqlx::query("INSERT INTO resource_path(title,classify,cover,url,introduction,type,status) VALUES(?,?,?,?,?,?,?)")
        .bind(input.title).bind(input.classify).bind(input.cover).bind(input.url).bind(input.introduction).bind(input.link_type).bind(i64::from(input.status))
        .execute(&state.pool).await.map_err(db_error)?.last_insert_rowid();
    load_link(&state, id).await
}
async fn update_link(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<LinkInput>,
) -> ApiResult<Link> {
    validate_link(&input)?;
    let changed = sqlx::query("UPDATE resource_path SET title=?,classify=?,cover=?,url=?,introduction=?,type=?,status=? WHERE id=?")
        .bind(input.title).bind(input.classify).bind(input.cover).bind(input.url).bind(input.introduction).bind(input.link_type).bind(i64::from(input.status)).bind(id)
        .execute(&state.pool).await.map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    load_link(&state, id).await
}
async fn change_link_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<StatusInput>,
) -> ApiResult<Link> {
    let changed = sqlx::query("UPDATE resource_path SET status=? WHERE id=?")
        .bind(i64::from(input.active))
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    load_link(&state, id).await
}
async fn delete_link(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let changed = sqlx::query("DELETE FROM resource_path WHERE id=?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}

#[derive(Serialize, sqlx::FromRow)]
struct TreeHole {
    id: i64,
    user_id: Option<i64>,
    username: Option<String>,
    avatar: Option<String>,
    message: String,
    image_path: Option<String>,
    create_time: Option<String>,
}
async fn content_tree_hole(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> ApiResult<Page<TreeHole>> {
    let (page, size) = page(query);
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tree_hole")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let items = sqlx::query_as::<_, TreeHole>(
        "SELECT t.id,t.user_id,u.username,t.avatar,t.message,t.image_path,t.create_time FROM tree_hole t LEFT JOIN user u ON u.id=t.user_id ORDER BY t.id DESC LIMIT ? OFFSET ?",
    )
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&state.pool)
    .await
    .map_err(db_error)?;
    Ok(Json(Page {
        items,
        total,
        page,
        size,
    }))
}
#[derive(Serialize, sqlx::FromRow)]
struct Note {
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
struct AdminNoteInput {
    content: String,
    is_public: Option<bool>,
}
async fn create_admin_note(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<AdminNoteInput>,
) -> ApiResult<Note> {
    let content = input.content.trim();
    if content.is_empty() || content.chars().count() > 1024 {
        return Err(invalid("微言内容应为 1 至 1024 字"));
    }
    let author = crate::site_author_id(&state.pool).await?;
    let id =
        sqlx::query("INSERT INTO wei_yan(user_id,content,type,is_public) VALUES(?,?,'friend',?)")
            .bind(author)
            .bind(content)
            .bind(i64::from(input.is_public.unwrap_or(true)))
            .execute(&state.pool)
            .await
            .map_err(db_error)?
            .last_insert_rowid();
    Ok(Json(sqlx::query_as::<_,Note>("SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id WHERE w.id=?")
        .bind(id).fetch_one(&state.pool).await.map_err(db_error)?))
}
async fn content_notes(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> ApiResult<Page<Note>> {
    let (page, size) = page(query);
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wei_yan WHERE type='friend'")
        .fetch_one(&state.pool)
        .await
        .map_err(db_error)?;
    let items = sqlx::query_as::<_,Note>("SELECT w.id,w.user_id,u.username,w.content,w.image_path,w.is_public,w.create_time FROM wei_yan w LEFT JOIN user u ON u.id=w.user_id WHERE w.type='friend' ORDER BY w.id DESC LIMIT ? OFFSET ?")
        .bind(size).bind((page-1)*size).fetch_all(&state.pool).await.map_err(db_error)?;
    Ok(Json(Page {
        items,
        total,
        page,
        size,
    }))
}
async fn delete_note(
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
async fn delete_tree_hole(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let changed = sqlx::query("DELETE FROM tree_hole WHERE id=?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(serde_json::json!({"deleted": true})))
}

#[derive(Serialize, sqlx::FromRow)]
struct Family {
    id: i64,
    bg_cover: Option<String>,
    man_cover: Option<String>,
    woman_cover: Option<String>,
    man_name: Option<String>,
    woman_name: Option<String>,
    timing: Option<String>,
    countdown_title: Option<String>,
    countdown_time: Option<String>,
    family_info: Option<String>,
    status: Option<i64>,
}
const FAMILY_SQL: &str = "SELECT id,bg_cover,man_cover,woman_cover,man_name,woman_name,timing,countdown_title,countdown_time,family_info,status FROM family";
async fn content_family(State(state): State<AppState>) -> ApiResult<Vec<Family>> {
    Ok(Json(
        sqlx::query_as::<_, Family>(sqlx::AssertSqlSafe(format!(
            "{FAMILY_SQL} ORDER BY id DESC LIMIT 100"
        )))
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FamilyInput {
    bg_cover: Option<String>,
    man_cover: Option<String>,
    woman_cover: Option<String>,
    man_name: String,
    woman_name: String,
    timing: String,
    countdown_title: Option<String>,
    countdown_time: Option<String>,
    family_info: Option<String>,
    status: bool,
}
fn valid_image(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(|v| {
        v.len() <= 2048
            && (v.starts_with("https://") || v.starts_with("http://") || v.starts_with("/media/"))
    })
}
fn validate_family(input: &FamilyInput) -> Result<(), AppError> {
    if input.man_name.trim().is_empty()
        || input.man_name.chars().count() > 32
        || input.woman_name.trim().is_empty()
        || input.woman_name.chars().count() > 32
        || input.timing.chars().count() > 32
        || input
            .countdown_title
            .as_deref()
            .is_some_and(|v| v.chars().count() > 32)
        || input
            .countdown_time
            .as_deref()
            .is_some_and(|v| v.chars().count() > 32)
        || input
            .family_info
            .as_deref()
            .is_some_and(|v| v.chars().count() > 1024)
        || !valid_image(&input.bg_cover)
        || !valid_image(&input.man_cover)
        || !valid_image(&input.woman_cover)
    {
        return Err(invalid("恋爱笔记内容无效"));
    }
    Ok(())
}
async fn load_family(state: &AppState, id: i64) -> ApiResult<Family> {
    Ok(Json(
        sqlx::query_as::<_, Family>(sqlx::AssertSqlSafe(format!("{FAMILY_SQL} WHERE id=?")))
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(db_error)?
            .ok_or_else(absent)?,
    ))
}
async fn create_family(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<FamilyInput>,
) -> ApiResult<Family> {
    validate_family(&input)?;
    let author = crate::site_author_id(&state.pool).await?;
    let id=sqlx::query("INSERT INTO family(user_id,bg_cover,man_cover,woman_cover,man_name,woman_name,timing,countdown_title,countdown_time,family_info,status) VALUES(?,?,?,?,?,?,?,?,?,?,?)")
        .bind(author)
        .bind(input.bg_cover).bind(input.man_cover).bind(input.woman_cover).bind(input.man_name).bind(input.woman_name).bind(input.timing)
        .bind(input.countdown_title).bind(input.countdown_time).bind(input.family_info).bind(i64::from(input.status))
        .execute(&state.pool).await.map_err(db_error)?.last_insert_rowid();
    load_family(&state, id).await
}
async fn update_family(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<FamilyInput>,
) -> ApiResult<Family> {
    validate_family(&input)?;
    let changed=sqlx::query("UPDATE family SET bg_cover=?,man_cover=?,woman_cover=?,man_name=?,woman_name=?,timing=?,countdown_title=?,countdown_time=?,family_info=?,status=?,update_time=CURRENT_TIMESTAMP WHERE id=?")
        .bind(input.bg_cover).bind(input.man_cover).bind(input.woman_cover).bind(input.man_name).bind(input.woman_name).bind(input.timing)
        .bind(input.countdown_title).bind(input.countdown_time).bind(input.family_info).bind(i64::from(input.status)).bind(id)
        .execute(&state.pool).await.map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    load_family(&state, id).await
}
async fn delete_family(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let changed = sqlx::query("DELETE FROM family WHERE id=?")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    Ok(Json(serde_json::json!({"deleted":true})))
}
