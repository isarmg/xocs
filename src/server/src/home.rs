use axum::{Json, Router, extract::State, routing::get};
use blog_application::admin_core::AuthenticatedIdentity;
use serde::{Deserialize, Serialize};

use crate::{ApiResult, AppState, invalid};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum HomeKind {
    Latest,
    Recommended,
    Category,
}

enum SectionKind {
    Latest,
    Recommended,
    Category(i64),
}

impl SectionKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Latest => "latest",
            Self::Recommended => "recommended",
            Self::Category(_) => "category",
        }
    }

    fn sort_id(&self) -> Option<i64> {
        match *self {
            Self::Latest | Self::Recommended => None,
            Self::Category(id) => Some(id),
        }
    }
}

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
    kind: HomeKind,
    sort_id: Option<i64>,
    priority: i64,
    enabled: bool,
}

struct Section {
    title: String,
    kind: SectionKind,
    priority: i64,
    enabled: bool,
}

impl TryFrom<SectionInput> for Section {
    type Error = crate::AppError;

    fn try_from(input: SectionInput) -> Result<Self, Self::Error> {
        let title = input.title.trim();
        if title.is_empty()
            || title.chars().count() > 32
            || !(-1000..=1000).contains(&input.priority)
        {
            return Err(invalid("栏目配置无效"));
        }
        let kind = match (input.kind, input.sort_id) {
            (HomeKind::Latest, None) => SectionKind::Latest,
            (HomeKind::Recommended, None) => SectionKind::Recommended,
            (HomeKind::Category, Some(id)) => SectionKind::Category(id),
            (HomeKind::Latest | HomeKind::Recommended, Some(_)) | (HomeKind::Category, None) => {
                return Err(invalid("栏目配置无效"));
            }
        };
        Ok(Self {
            title: title.to_owned(),
            kind,
            priority: input.priority,
            enabled: input.enabled,
        })
    }
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
        .fetch_all(&state.pool).await?))
}

async fn admin_sections(State(state): State<AppState>) -> ApiResult<Vec<HomeSection>> {
    Ok(Json(
        sqlx::query_as(
            "SELECT id,title,kind,sort_id,priority,enabled FROM home_sections ORDER BY priority,id",
        )
        .fetch_all(&state.pool)
        .await?,
    ))
}

async fn save_sections(
    State(state): State<AppState>,
    axum::Extension(identity): axum::Extension<AuthenticatedIdentity>,
    crate::ContractJson(input): crate::ContractJson<Vec<SectionInput>>,
) -> ApiResult<Vec<HomeSection>> {
    if input.is_empty() || input.len() > 12 || !input.iter().any(|item| item.enabled) {
        return Err(invalid("请保留至少一个启用的栏目，最多 12 个"));
    }
    let sections = input
        .into_iter()
        .map(Section::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let mut tx = crate::content::begin_mutation(&state.pool, &identity).await?;
    for item in &sections {
        if let SectionKind::Category(id) = item.kind {
            let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sort WHERE id=?")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
            if exists == 0 {
                return Err(invalid("栏目所选分类不存在"));
            }
        }
    }
    sqlx::query("DELETE FROM home_sections")
        .execute(&mut *tx)
        .await?;
    for item in sections {
        sqlx::query(
            "INSERT INTO home_sections(title,kind,sort_id,priority,enabled) VALUES(?,?,?,?,?)",
        )
        .bind(item.title)
        .bind(item.kind.as_str())
        .bind(item.kind.sort_id())
        .bind(item.priority)
        .bind(item.enabled)
        .execute(&mut *tx)
        .await?;
    }
    let sections = sqlx::query_as(
        "SELECT id,title,kind,sort_id,priority,enabled FROM home_sections ORDER BY priority,id",
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(sections))
}
