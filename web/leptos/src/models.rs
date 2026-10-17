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

/// Decode only after the same strict response contract used by dynamic forms.
#[cfg(test)]
pub fn decode<T: serde::de::DeserializeOwned>(
    path: &str,
    method: &str,
    value: serde_json::Value,
) -> Result<T, ()> {
    if !crate::contracts::validate(path, method, &value)
        || method == "GET" && !crate::contracts::matches_page(path, &value)
    {
        return Err(());
    }
    serde_json::from_value(value).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn typed_site_keeps_required_nullable_fields_and_ignores_extensions() {
        let mut site = json!({"web_name":null,"web_title":"Title","notices":null,
            "background_image":null,"avatar":null,"random_cover":null,"future":true});
        let decoded: Site = decode("/api/v1/site", "GET", site.clone()).unwrap();
        assert_eq!(decoded.web_title.as_deref(), Some("Title"));
        assert!(decoded.web_name.is_none());
        site.as_object_mut().unwrap().remove("avatar");
        assert!(decode::<Site>("/api/v1/site", "GET", site).is_err());
    }

    #[test]
    fn typed_pages_reject_wrong_page_and_unsafe_integers() {
        let page = json!({"items":[],"total":0,"page":2,"size":20});
        assert!(
            decode::<Page<ArticleSummary>>("/api/v1/articles?page=2&size=20", "GET", page.clone())
                .is_ok()
        );
        assert!(
            decode::<Page<ArticleSummary>>("/api/v1/articles?page=1&size=20", "GET", page).is_err()
        );
        assert!(
            decode::<SiteStats>(
                "/api/v1/site/stats",
                "GET",
                json!({"article_count":9007199254740992u64,"view_count":0})
            )
            .is_err()
        );
    }

    #[test]
    fn typed_flags_preserve_numeric_wire_contract() {
        let access: ArticleAccess = decode(
            "/api/v1/articles/1/access",
            "GET",
            json!({"password_required":1,"tips":null}),
        )
        .unwrap();
        assert!(access.password_required);
        for flag in [json!(true), json!(2), json!("1")] {
            assert!(
                decode::<ArticleAccess>(
                    "/api/v1/articles/1/access",
                    "GET",
                    json!({"password_required":flag,"tips":null})
                )
                .is_err()
            );
        }
    }

    #[test]
    fn typed_article_draft_preserves_nullable_write_semantics() {
        let draft = ArticleDraft {
            article_title: "Story".into(),
            article_content: "Markdown".into(),
            article_cover: Some(String::new()),
            password: Some(String::new()),
            tips: Some("hint".into()),
            ..Default::default()
        };
        let body = serde_json::to_value(draft.for_submission()).unwrap();
        assert_eq!(
            body,
            json!({"article_title":"Story","article_content":"Markdown",
            "article_cover":null,"video_url":null,"sort_id":0,"label_id":0,
            "view_status":true,"recommend_status":false,"comment_status":true,
            "password":null,"tips":"hint"})
        );
    }

    #[test]
    fn detail_decodes_without_summary_only_fields_and_hydrates_no_password() {
        let article: Article = decode(
            "/api/v1/content/articles/7",
            "GET",
            json!({
                "id":7,"user_id":1,"article_title":"Story","article_content":"## Body",
                "article_cover":null,"video_url":null,"sort_id":1,"label_id":2,
                "sort_name":"Category","label_name":"Tag","username":null,
                "create_time":null,"update_time":null,"tips":"hint",
                "recommend_status":0,"view_status":0,"comment_status":1,
                "password_required":1,"view_count":0,"like_count":0,"comment_count":0
            }),
        )
        .unwrap();
        assert_eq!(article.id, 7);
        assert!(article.excerpt.is_none());
        assert!(article.search_snippet.is_none());
        let draft = ArticleDraft::from(&article);
        assert!(draft.password.is_none());
        assert!(!draft.view_status);
        assert!(draft.comment_status);
        assert_eq!(draft.tips.as_deref(), Some("hint"));
    }
}
