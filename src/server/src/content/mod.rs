use crate::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

mod articles;
mod catalog;
mod comments;
mod family;
mod guestbook;
mod letter;
mod links;
mod notes;
mod resources;
mod site;
mod statistics;

#[derive(Serialize)]
struct PageResult<T> {
    items: Vec<T>,
    total: i64,
    page: i64,
    size: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdminPageQuery {
    page: Option<i64>,
    size: Option<i64>,
}
impl AdminPageQuery {
    fn bounds(self) -> (i64, i64) {
        (
            self.page.unwrap_or(1).clamp(1, 100_000),
            self.size.unwrap_or(20).clamp(1, 100),
        )
    }
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/site", get(site::site_info))
        .route("/api/v1/letter", get(letter::public_letter))
        .route("/api/v1/articles", get(articles::public_articles))
        .route("/api/v1/articles/{id}", get(articles::public_article))
        .route(
            "/api/v1/articles/{id}/access",
            get(articles::article_access),
        )
        .route(
            "/api/v1/articles/{id}/unlock",
            post(articles::unlock_article),
        )
        .route("/api/v1/articles/{id}/news", get(articles::article_news))
        .route("/api/v1/categories", get(catalog::categories))
        .route("/api/v1/site/stats", get(catalog::public_site_stats))
        .route("/api/v1/labels", get(catalog::public_labels))
        .route(
            "/api/v1/comments",
            get(comments::comments).post(comments::create_comment),
        )
        .route(
            "/api/v1/comments/{id}/replies",
            get(comments::comment_replies),
        )
        .route(
            "/api/v1/message-comments",
            get(comments::message_comments).post(comments::create_message_comment),
        )
        .route(
            "/api/v1/message-comments/{id}/replies",
            get(comments::message_comment_replies),
        )
        .route(
            "/api/v1/love-comments",
            get(comments::love_comments).post(comments::create_love_comment),
        )
        .route(
            "/api/v1/love-comments/{id}/replies",
            get(comments::love_comment_replies),
        )
        .route("/api/v1/notes/page", get(notes::note_page))
        .route("/api/v1/tree-hole", get(guestbook::tree_hole))
        .route(
            "/api/v1/tree-hole/guest",
            post(guestbook::create_guest_tree_hole),
        )
        .route("/api/v1/links/page", get(links::link_page))
        .route("/api/v1/links/classes", get(links::link_classes))
        .route("/api/v1/family", get(family::family))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/content/letter",
            get(letter::admin_letter).put(letter::save_letter),
        )
        .route(
            "/api/v1/content/articles",
            get(articles::admin_articles).post(articles::create_article),
        )
        .route(
            "/api/v1/content/articles/{id}",
            get(articles::admin_article)
                .put(articles::update_article)
                .delete(articles::delete_article),
        )
        .route(
            "/api/v1/content/articles/{id}/news",
            get(articles::admin_article_news).post(articles::admin_create_article_news),
        )
        .route(
            "/api/v1/content/articles/{id}/news/{news_id}",
            axum::routing::put(articles::admin_update_article_news)
                .delete(articles::admin_delete_article_news),
        )
        .route("/api/v1/content/categories", post(catalog::create_category))
        .route(
            "/api/v1/content/categories/{id}",
            axum::routing::put(catalog::update_category).delete(catalog::delete_category),
        )
        .route("/api/v1/content/comments", get(comments::admin_comments))
        .route(
            "/api/v1/content/comments/{id}",
            axum::routing::delete(comments::delete_comment),
        )
        .route(
            "/api/v1/content/site",
            get(site::admin_site).put(site::update_site),
        )
        .route(
            "/api/v1/content/labels",
            get(catalog::admin_labels).post(catalog::create_label),
        )
        .route(
            "/api/v1/content/labels/{id}",
            axum::routing::put(catalog::update_label).delete(catalog::delete_label),
        )
        .route("/api/v1/content/statistics", get(statistics::statistics))
        .route("/api/v1/content/resources", get(resources::resources))
        .route(
            "/api/v1/content/links",
            get(links::content_links).post(links::create_link),
        )
        .route(
            "/api/v1/content/links/{id}",
            axum::routing::put(links::update_link).delete(links::delete_link),
        )
        .route(
            "/api/v1/content/links/{id}/status",
            axum::routing::put(links::change_link_status),
        )
        .route(
            "/api/v1/content/tree-hole",
            get(guestbook::content_tree_hole),
        )
        .route(
            "/api/v1/content/notes",
            get(notes::content_notes).post(notes::create_admin_note),
        )
        .route(
            "/api/v1/content/notes/{id}",
            axum::routing::put(notes::update_admin_note).delete(notes::delete_note),
        )
        .route(
            "/api/v1/content/tree-hole/{id}",
            axum::routing::delete(guestbook::delete_tree_hole),
        )
        .route(
            "/api/v1/content/family",
            get(family::content_family).post(family::create_family),
        )
        .route(
            "/api/v1/content/family/{id}",
            axum::routing::put(family::update_family).delete(family::delete_family),
        )
}
