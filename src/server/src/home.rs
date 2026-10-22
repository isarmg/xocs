use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};

use crate::{ApiResult, AppState, db_error, invalid};

#[derive(Serialize, sqlx::FromRow)]
struct HomeSection {
    id: i64,
    title: String,
    kind: String,
    sort_id: Option<i64>,
    priority: i64,
    enabled: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SectionInput {
    title: String,
    kind: String,
    sort_id: Option<i64>,
    priority: i64,
    enabled: bool,
}

pub fn public_routes() -> Router<AppState> {
    Router::new().route("/api/v1/home-sections", get(public_sections))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new().route(
        "/api/v1/content/home-sections",
        get(admin_sections).put(save_sections),
    )
}

async fn public_sections(State(state): State<AppState>) -> ApiResult<Vec<HomeSection>> {
    Ok(Json(sqlx::query_as("SELECT id,title,kind,sort_id,priority,enabled FROM home_sections WHERE enabled=1 ORDER BY priority,id")
        .fetch_all(&state.pool).await.map_err(db_error)?))
}

async fn admin_sections(State(state): State<AppState>) -> ApiResult<Vec<HomeSection>> {
    Ok(Json(
        sqlx::query_as(
            "SELECT id,title,kind,sort_id,priority,enabled FROM home_sections ORDER BY priority,id",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(db_error)?,
    ))
}

async fn save_sections(
    State(state): State<AppState>,
    crate::ContractJson(input): crate::ContractJson<Vec<SectionInput>>,
) -> ApiResult<Vec<HomeSection>> {
    if input.is_empty() || input.len() > 12 || !input.iter().any(|item| item.enabled) {
        return Err(invalid("请保留至少一个启用的栏目，最多 12 个"));
    }
    let mut tx = state.pool.begin().await.map_err(db_error)?;
    for item in &input {
        let title = item.title.trim();
        if title.is_empty()
            || title.chars().count() > 32
            || !(-1000..=1000).contains(&item.priority)
            || !matches!(item.kind.as_str(), "latest" | "recommended" | "category")
            || (item.kind == "category") != item.sort_id.is_some()
        {
            return Err(invalid("栏目配置无效"));
        }
        if let Some(id) = item.sort_id {
            let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sort WHERE id=?")
                .bind(id)
                .fetch_one(&mut *tx)
                .await
                .map_err(db_error)?;
            if exists == 0 {
                return Err(invalid("栏目所选分类不存在"));
            }
        }
    }
    sqlx::query("DELETE FROM home_sections")
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    for item in input {
        sqlx::query(
            "INSERT INTO home_sections(title,kind,sort_id,priority,enabled) VALUES(?,?,?,?,?)",
        )
        .bind(item.title.trim().to_owned())
        .bind(item.kind)
        .bind(item.sort_id)
        .bind(item.priority)
        .bind(item.enabled)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    }
    tx.commit().await.map_err(db_error)?;
    admin_sections(State(state)).await
}
