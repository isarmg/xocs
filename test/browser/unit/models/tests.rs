use super::*;
use serde_json::json;

#[test]
fn article_route_keys_reject_invalid_or_unsafe_numbers() {
    for (raw, expected) in [("1", 1), ("9007199254740991", 9_007_199_254_740_991)] {
        assert_eq!(ArticleId::parse(raw).unwrap().get(), expected);
    }
    for raw in [
        "",
        "word",
        "0",
        "-1",
        "1/2",
        "9007199254740992",
        "9223372036854775808",
    ] {
        assert!(ArticleId::parse(raw).is_none(), "{raw}");
    }
}

fn site_payload() -> serde_json::Value {
    serde_json::from_str(include_str!("site.json")).unwrap()
}

#[test]
fn typed_site_keeps_required_nullable_fields_and_ignores_extensions() {
    let mut site = site_payload();
    site["future"] = json!({"new":true});
    site["media"]["future"] = json!([false]);
    let directly_decoded: Site = serde_json::from_value(site.clone()).unwrap();
    let decoded: Site = decode("/api/v1/site", "GET", site.clone()).unwrap();
    assert_eq!(decoded.web_title.as_deref(), Some("Title"));
    assert!(decoded.web_name.is_none());
    assert_eq!(
        directly_decoded.media.toolbox_video.as_deref(),
        Some("/media/background.webm")
    );
    assert_eq!(
        decoded.media.toolbox_video,
        directly_decoded.media.toolbox_video
    );
    assert!(decoded.media.logo.is_none());
    site["web_title"] = serde_json::Value::Null;
    site["media"]["toolbox_video"] = serde_json::Value::Null;
    let all_null: Site = serde_json::from_value(site.clone()).unwrap();
    assert!(all_null.web_title.is_none());
    assert!(all_null.media.toolbox_video.is_none());
    assert!(decode::<Site>("/api/v1/site", "GET", site).is_ok());
}

#[test]
fn typed_site_deserialization_requires_every_nullable_field_without_boundary_help() {
    let baseline = site_payload();
    for key in baseline.as_object().unwrap().keys() {
        let mut missing = baseline.clone();
        missing.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<Site>(missing.clone()).is_err(),
            "missing {key}"
        );
        assert!(
            decode::<Site>("/api/v1/site", "GET", missing).is_err(),
            "missing {key}"
        );
    }
    for key in baseline["media"].as_object().unwrap().keys() {
        let mut missing = baseline.clone();
        missing["media"].as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<Site>(missing.clone()).is_err(),
            "missing media.{key}"
        );
        assert!(
            decode::<Site>("/api/v1/site", "GET", missing).is_err(),
            "missing media.{key}"
        );
    }
}

#[test]
fn typed_site_nullable_fields_reject_nonstring_values() {
    let baseline = site_payload();
    for wrong in [json!(true), json!(12), json!([]), json!({})] {
        for key in baseline
            .as_object()
            .unwrap()
            .keys()
            .filter(|key| key.as_str() != "media")
        {
            let mut invalid = baseline.clone();
            invalid[key] = wrong.clone();
            assert!(
                serde_json::from_value::<Site>(invalid.clone()).is_err(),
                "{key}: {wrong}"
            );
            assert!(
                decode::<Site>("/api/v1/site", "GET", invalid).is_err(),
                "{key}: {wrong}"
            );
        }
        for key in baseline["media"].as_object().unwrap().keys() {
            let mut invalid = baseline.clone();
            invalid["media"][key] = wrong.clone();
            assert!(
                serde_json::from_value::<Site>(invalid.clone()).is_err(),
                "media.{key}: {wrong}"
            );
            assert!(
                decode::<Site>("/api/v1/site", "GET", invalid).is_err(),
                "media.{key}: {wrong}"
            );
        }
    }
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

#[test]
fn home_source_carries_its_required_category_id() {
    for (kind, expected) in [
        ("latest", HomeSource::Latest),
        ("recommended", HomeSource::Recommended),
        ("category", HomeSource::Category { sort_id: 42 }),
    ] {
        let sections: Vec<HomeSection> = decode(
            "/api/v1/home-sections",
            "POST",
            json!([{"id": 1, "title": "Section", "kind": kind,
                "sort_id": if kind == "category" { json!(42) } else { json!(null) },
                "priority": 0, "enabled": true}]),
        )
        .unwrap();
        assert_eq!(sections[0].source, expected);
    }
    for sort_id in [
        json!(null),
        json!(0),
        json!(-1),
        json!("42"),
        json!(9007199254740992u64),
    ] {
        assert!(
            decode::<Vec<HomeSection>>(
                "/api/v1/home-sections",
                "GET",
                json!([{"id": 1, "title": "Section", "kind": "category",
                "sort_id": sort_id, "priority": 0, "enabled": true}]),
            )
            .is_err()
        );
    }
    assert!(
        serde_json::from_value::<HomeSection>(json!({
            "id": 1, "title": "Section", "kind": "category", "enabled": true,
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<HomeSection>(json!({
            "id": 1, "title": "Section", "kind": "unknown", "sort_id": null, "enabled": true,
        }))
        .is_err()
    );
}
