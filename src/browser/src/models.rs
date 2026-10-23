//! Typed read models consumed by the public article pages and administration.
//!
//! The endpoint contract is checked before deserialization. In particular,
//! nullable-but-required JSON fields, safe integers and page identity retain
//! their original validation; `Option` never makes a missing field acceptable.
//! These are read projections: unused wire fields stay validated at the boundary
//! and are not retained or decoded a second time into reactive application state.
use serde::{Deserialize, Deserializer, Serialize};

fn binary_flag<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    match i64::deserialize(d)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(serde::de::Error::custom("expected a binary integer flag")),
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub size: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Site {
    pub web_name: Option<String>,
    pub web_title: Option<String>,
    pub notices: Option<String>,
    pub background_image: Option<String>,
    pub avatar: Option<String>,
    pub random_cover: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SiteStats {
    pub article_count: i64,
    pub view_count: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdminStats {
    pub articles: i64,
    pub comments: i64,
    pub members: i64,
    pub categories: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Category {
    pub id: i64,
    pub sort_name: String,
    pub sort_description: Option<String>,
    pub article_count: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Label {
    pub id: i64,
    pub sort_id: i64,
    pub label_name: String,
    // The administrative endpoint intentionally omits this public-only field.
    pub article_count: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ArticleSummary {
    pub id: i64,
    pub article_title: String,
    pub article_cover: Option<String>,
    pub sort_id: i64,
    pub label_id: i64,
    pub sort_name: Option<String>,
    pub label_name: Option<String>,
    pub create_time: Option<String>,
    #[serde(deserialize_with = "binary_flag")]
    pub recommend_status: bool,
    #[serde(deserialize_with = "binary_flag")]
    pub view_status: bool,
    pub view_count: i64,
    pub like_count: i64,
    pub comment_count: i64,
    pub excerpt: Option<String>,
    pub search_snippet: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Article {
    #[serde(flatten)]
    pub summary: ArticleSummary,
    pub article_content: String,
    #[serde(deserialize_with = "binary_flag")]
    pub comment_status: bool,
    #[serde(deserialize_with = "binary_flag")]
    pub password_required: bool,
    pub video_url: Option<String>,
    pub username: Option<String>,
    pub tips: Option<String>,
    pub update_time: Option<String>,
}
impl std::ops::Deref for Article {
    type Target = ArticleSummary;
    fn deref(&self) -> &Self::Target {
        &self.summary
    }
}

/// Editor state and write payload have the same explicit wire fields. Passwords
/// are never hydrated from a response; null keeps an existing password intact.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ArticleDraft {
    pub article_title: String,
    pub article_content: String,
    pub article_cover: Option<String>,
    pub video_url: Option<String>,
    pub sort_id: i64,
    pub label_id: i64,
    pub view_status: bool,
    pub recommend_status: bool,
    pub comment_status: bool,
    pub password: Option<String>,
    pub tips: Option<String>,
}
impl Default for ArticleDraft {
    fn default() -> Self {
        Self {
            article_title: String::new(),
            article_content: String::new(),
            article_cover: None,
            video_url: None,
            sort_id: 0,
            label_id: 0,
            view_status: true,
            recommend_status: false,
            comment_status: true,
            password: None,
            tips: None,
        }
    }
}
impl From<&Article> for ArticleDraft {
    fn from(article: &Article) -> Self {
        Self {
            article_title: article.article_title.clone(),
            article_content: article.article_content.clone(),
            article_cover: article.article_cover.clone(),
            video_url: article.video_url.clone(),
            sort_id: article.sort_id,
            label_id: article.label_id,
            view_status: article.view_status,
            recommend_status: article.recommend_status,
            comment_status: article.comment_status,
            password: None,
            tips: article.tips.clone(),
        }
    }
}
impl ArticleDraft {
    pub fn for_submission(mut self) -> Self {
        for field in [
            &mut self.article_cover,
            &mut self.video_url,
            &mut self.password,
            &mut self.tips,
        ] {
            if field.as_ref().is_some_and(String::is_empty) {
                *field = None;
            }
        }
        self
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ArticleAccess {
    #[serde(deserialize_with = "binary_flag")]
    pub password_required: bool,
    pub tips: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HomeKind {
    Latest,
    Recommended,
    Category,
}
#[derive(Clone, Debug, Deserialize)]
pub struct HomeSection {
    pub id: i64,
    pub title: String,
    pub kind: HomeKind,
    pub sort_id: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct News {
    pub id: i64,
    pub content: String,
    pub create_time: Option<String>,
}

#[cfg(test)]
include!("../../../test/browser/unit/models/decode.rs");

#[path = "../../../test/browser/unit/models/tests.rs"]
#[cfg(test)]
mod tests;
