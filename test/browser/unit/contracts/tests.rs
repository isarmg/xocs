use super::*;
use serde_json::json;
#[test]
fn invalid_pages_and_missing_nulls_rejected() {
    assert!(!page(
        &json!({"items":[],"page":1,"size":0,"total":0}),
        comment
    ));
    assert!(!site(&json!({"web_name":"blog"})));
}
#[test]
fn read_page_identity_and_nullable_fields() {
    let v = json!({"items":[],"page":2,"size":20,"total":21});
    assert!(matches_page("/api/v1/content/notes?page=2&size=20", &v));
    assert!(!matches_page("/api/v1/content/notes?page=1&size=20", &v));
    assert!(!category(
        &json!({"id":1,"sort_name":"A","article_count":0})
    ));
    assert!(!validate(
        "/api/v1/content/family",
        "GET",
        &json!([{"id":1,"bg_cover":null,"man_cover":null,"woman_cover":null,"man_name":null,"woman_name":null,"timing":null,"countdown_title":null,"countdown_time":null,"family_info":null}])
    ));
}
#[test]
fn consumed_read_models_reject_malformed_fields() {
    let summary = json!({"id":1,"article_title":"Story","article_cover":null,"sort_id":0,"label_id":0,"sort_name":null,"label_name":null,"create_time":null,"recommend_status":0,"view_status":1,"view_count":0,"like_count":0,"comment_count":0,"excerpt":"Text","search_snippet":null});
    let page = |item: Value| json!({"items":[item],"total":1,"page":1,"size":10});
    let mut article = summary.clone();
    for (key, value) in [
        ("user_id", json!(1)),
        ("article_content", json!("Body")),
        ("comment_status", json!(1)),
        ("password_required", json!(0)),
        ("video_url", Value::Null),
        ("username", Value::Null),
        ("tips", Value::Null),
        ("update_time", Value::Null),
    ] {
        article[key] = value;
    }
    let category = json!({"id":1,"sort_name":"Stories","sort_description":null,"priority":-1,"article_count":0});
    let label =
        json!({"id":1,"sort_id":1,"label_name":"Tag","label_description":null,"article_count":0});
    let comment = json!({"id":1,"user_id":null,"username":null,"avatar":null,"comment_content":"Hello","create_time":null,"parent_username":null,"reply_count":0});
    let note = json!({"id":1,"user_id":1,"username":null,"content":"Moment","image_path":null,"create_time":null,"like_count":0,"is_public":1});
    let link = json!({"id":1,"title":"Photo","classify":null,"cover":null,"url":null,"introduction":null,"link_type":"lovePhoto","create_time":null});
    let mut bad_summary = summary.clone();
    bad_summary["view_count"] = json!("0");
    let mut bad_article = article.clone();
    bad_article["article_content"] = json!([]);
    let mut bad_category = category.clone();
    bad_category["sort_description"] = json!(false);
    let mut bad_label = label.clone();
    bad_label["article_count"] = json!(-1);
    let mut bad_comment = comment.clone();
    bad_comment["reply_count"] = json!(-1);
    let mut bad_note = note.clone();
    bad_note["like_count"] = Value::Null;
    let mut bad_link = link.clone();
    bad_link["cover"] = json!([]);
    for (path, valid, invalid) in [
        (
            "/api/v1/articles?page=1&size=10",
            page(summary),
            page(bad_summary),
        ),
        ("/api/v1/articles/1", article, bad_article),
        (
            "/api/v1/categories",
            json!([category]),
            json!([bad_category]),
        ),
        ("/api/v1/labels", json!([label]), json!([bad_label])),
        (
            "/api/v1/message-comments?page=1&size=10",
            page(comment),
            page(bad_comment),
        ),
        (
            "/api/v1/notes/page?page=1&size=10",
            page(note),
            page(bad_note),
        ),
        (
            "/api/v1/links/page?page=1&size=10",
            page(link),
            page(bad_link),
        ),
    ] {
        assert!(
            validate(path, "GET", &valid) && matches_page(path, &valid),
            "{path}"
        );
        assert!(!validate(path, "GET", &invalid), "{path}");
    }
    assert!(validate("/api/v1/letter", "GET", &Value::Null));
    assert!(!validate("/api/v1/content/letter", "GET", &Value::Null));
    assert!(!validate(
        "/api/v1/content/notes/1",
        "DELETE",
        &json!({"deleted":false})
    ));
}
#[test]
fn upload_path_and_size() {
    let mut v = json!({"id":1,"path":format!("/media/{}.png","a".repeat(64)),"mime_type":"image/png","size":1});
    assert!(upload(&v));
    v["size"] = json!(10 * 1024 * 1024 + 1);
    assert!(!upload(&v));
}
