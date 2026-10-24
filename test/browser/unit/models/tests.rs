use super::*;
use serde_json::json;

#[test]
fn typed_site_keeps_required_nullable_fields_and_ignores_extensions() {
    let mut site = json!({"web_name":null,"web_title":"Title","notices":null,
            "background_image":null,"avatar":null,"random_cover":null,"future":true,
            "media":{"logo":null,"journey_background":null,"journal_background":null,
                "guestbook_background":null,"love_letter_background":null,
                "love_moments_cover":null,"love_photos_cover":null,"love_wishes_cover":null,
                "toolbox_background":null,"toolbox_video":"/media/background.webm"}});
    let decoded: Site = decode("/api/v1/site", "GET", site.clone()).unwrap();
    assert_eq!(decoded.web_title.as_deref(), Some("Title"));
    assert!(decoded.web_name.is_none());
    assert_eq!(
        decoded.media.toolbox_video.as_deref(),
        Some("/media/background.webm")
    );
    assert!(decoded.media.logo.is_none());
    for invalid in [json!(null), json!({}), json!({"toolbox_video":false})] {
        let mut invalid_site = site.clone();
        invalid_site["media"] = invalid;
        assert!(decode::<Site>("/api/v1/site", "GET", invalid_site).is_err());
    }
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
