use super::{Group, Page, Route};
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
    assert!(Route::parse("edit/42") == Route::EditArticle(42));
    assert!(Route::parse("edit/invalid") == Route::EditArticle(0));
    for hash in ["new", "edit/42", "edit/invalid", "", "unknown"] {
        let route = Route::parse(hash);
        assert!(route.active_page() == Some(Page::Articles));
    }
}

#[test]
fn account_settings_have_no_selected_content_navigation_page() {
    let route = Route::parse("account");
    assert!(route == Route::Account);
    assert!(route.active_page().is_none());
}
