use crate::{ApiResult, AppError, AppState, absent, db_error, invalid};
use axum::{Json, extract::State};
use blog_application::admin_core::AuthenticatedIdentity;
use blog_application::error::HttpStatus;
use blog_application::server_cli::ContractPath as Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct Family {
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
}
pub(super) async fn family(State(state): State<AppState>) -> ApiResult<Vec<Family>> {
    let author = crate::site_author_id(&state.pool).await?;
    Ok(Json(sqlx::query_as::<_,Family>("SELECT id,bg_cover,man_cover,woman_cover,man_name,woman_name,timing,countdown_title,countdown_time,family_info FROM family WHERE user_id=? AND status=1")
        .bind(author).fetch_all(&state.pool).await.map_err(db_error)?))
}

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct AdminFamily {
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
const FAMILY_OWNER: &str = "user_id=?";
pub(super) async fn content_family(State(state): State<AppState>) -> ApiResult<Vec<AdminFamily>> {
    let author = crate::site_author_id(&state.pool).await?;
    Ok(Json(
        sqlx::query_as::<_, AdminFamily>(sqlx::AssertSqlSafe(format!(
            "{FAMILY_SQL} WHERE {FAMILY_OWNER}"
        )))
        .bind(author)
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FamilyInput {
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
async fn load_family(
    connection: &mut sqlx::SqliteConnection,
    id: i64,
    author: i64,
) -> Result<AdminFamily, AppError> {
    sqlx::query_as::<_, AdminFamily>(sqlx::AssertSqlSafe(format!(
        "{FAMILY_SQL} WHERE id=? AND {FAMILY_OWNER}"
    )))
    .bind(id)
    .bind(author)
    .fetch_optional(connection)
    .await
    .map_err(db_error)?
    .ok_or_else(absent)
}
pub(super) async fn create_family(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    crate::ContractJson(input): crate::ContractJson<FamilyInput>,
) -> ApiResult<AdminFamily> {
    validate_family(&input)?;
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let author = crate::site_author_id(&mut *transaction).await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM family WHERE user_id=?)")
        .bind(author)
        .fetch_one(&mut *transaction)
        .await
        .map_err(db_error)?;
    if exists {
        return Err(AppError::Public(
            HttpStatus::Conflict,
            "家已创建，请编辑现有资料",
        ));
    }
    let id=sqlx::query("INSERT INTO family(user_id,bg_cover,man_cover,woman_cover,man_name,woman_name,timing,countdown_title,countdown_time,family_info,status) VALUES(?,?,?,?,?,?,?,?,?,?,?)")
        .bind(author)
        .bind(input.bg_cover).bind(input.man_cover).bind(input.woman_cover).bind(input.man_name).bind(input.woman_name).bind(input.timing)
        .bind(input.countdown_title).bind(input.countdown_time).bind(input.family_info).bind(i64::from(input.status))
        .execute(&mut *transaction).await.map_err(db_error)?.last_insert_rowid();
    let family = load_family(&mut transaction, id, author).await?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(family))
}
pub(super) async fn update_family(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
    crate::ContractJson(input): crate::ContractJson<FamilyInput>,
) -> ApiResult<AdminFamily> {
    validate_family(&input)?;
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let author = crate::site_author_id(&mut *transaction).await?;
    let changed=sqlx::query(sqlx::AssertSqlSafe(format!("UPDATE family SET bg_cover=?,man_cover=?,woman_cover=?,man_name=?,woman_name=?,timing=?,countdown_title=?,countdown_time=?,family_info=?,status=?,update_time=CURRENT_TIMESTAMP WHERE id=? AND {FAMILY_OWNER}")))
        .bind(input.bg_cover).bind(input.man_cover).bind(input.woman_cover).bind(input.man_name).bind(input.woman_name).bind(input.timing)
        .bind(input.countdown_title).bind(input.countdown_time).bind(input.family_info).bind(i64::from(input.status)).bind(id).bind(author)
        .execute(&mut *transaction).await.map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    let family = load_family(&mut transaction, id, author).await?;
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(family))
}
pub(super) async fn delete_family(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    Path(id): Path<i64>,
) -> ApiResult<serde_json::Value> {
    let author = crate::site_author_id(&state.pool).await?;
    let mut transaction = crate::content::begin_mutation(&state.pool, &identity).await?;
    let changed = sqlx::query(sqlx::AssertSqlSafe(format!(
        "DELETE FROM family WHERE id=? AND {FAMILY_OWNER}"
    )))
    .bind(id)
    .bind(author)
    .execute(&mut *transaction)
    .await
    .map_err(db_error)?;
    if changed.rows_affected() == 0 {
        return Err(absent());
    }
    transaction.commit().await.map_err(db_error)?;
    Ok(Json(serde_json::json!({"deleted":true})))
}
