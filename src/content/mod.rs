use crate::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use serde::Serialize;

mod articles;
mod catalog;
mod comments;
mod family;
mod guestbook;
mod links;
mod notes;
mod site;

#[derive(Serialize)]
struct PageResult<T> {
    items: Vec<T>,
    total: i64,
    page: i64,
    size: i64,
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/site", get(site::site_info))
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
        .route("/api/v1/links", get(links::links))
        .route("/api/v1/links/page", get(links::link_page))
        .route("/api/v1/links/classes", get(links::link_classes))
        .route("/api/v1/family", get(family::family))
}

pub fn admin_routes() -> Router<AppState> {
    Router::new()
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
            axum::routing::delete(articles::admin_delete_article_news),
        )
        .route("/api/v1/content/categories", post(catalog::create_category))
        .route(
            "/api/v1/content/categories/{id}",
            axum::routing::put(catalog::update_category).delete(catalog::delete_category),
        )
}
