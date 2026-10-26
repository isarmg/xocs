use super::{Group, Page, Route};
use crate::models::ArticleId;
use std::collections::HashSet;

#[test]
fn every_navigation_page_has_a_unique_hash_and_round_trips() {
    let mut hashes = HashSet::new();
    for group in Group::ALL {
        assert!(
            !group.pages().is_empty(),
            "a navigation group needs a landing page"
        );
        for page in group.pages().iter().copied() {
            assert!(
                hashes.insert(page.hash()),
                "duplicate navigation hash: {}",
                page.hash()
            );
            let route = Route::parse(page.hash());
            assert!(route == Route::Page(page));
            assert!(route.active_page() == Some(page));
        }
    }
    assert_eq!(
        hashes.len(),
        13,
        "every administration page remains reachable"
    );
}

#[test]
fn article_editor_routes_share_article_navigation_and_keep_the_requested_id() {
    assert!(Route::parse("new") == Route::NewArticle);
    let id = ArticleId::parse("42").expect("fixture article identifier is valid");
    assert!(Route::parse("edit/42") == Route::EditArticle(id));
    for hash in ["new", "edit/42", ""] {
        let route = Route::parse(hash);
        assert!(route.active_page() == Some(Page::Articles));
    }
}

#[test]
fn invalid_article_and_unknown_routes_have_no_selected_content_page() {
    for hash in ["edit/invalid", "edit/0", "edit/-1", "edit/1/2", "unknown"] {
        let route = Route::parse(hash);
        assert!(route == Route::NotFound, "{hash}");
        assert!(route.active_page().is_none(), "{hash}");
    }
}

#[test]
fn account_settings_have_no_selected_content_navigation_page() {
    let route = Route::parse("account");
    assert!(route == Route::Account);
    assert!(route.active_page().is_none());
}
