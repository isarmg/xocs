use serde_json::Value;
fn fields(v: &Value, keys: &[&str], test: fn(&Value) -> bool) -> bool {
    v.is_object() && keys.iter().all(|k| v.get(k).is_some_and(test))
}
fn nullable(v: &Value) -> bool {
    v.is_null() || v.is_string()
}
fn integer(v: &Value) -> bool {
    v.as_i64()
        .is_some_and(|n| (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&n))
}
fn count(v: &Value) -> bool {
    integer(v) && v.as_i64().is_some_and(|n| n >= 0)
}
fn id(v: &Value) -> bool {
    integer(v) && v.as_i64().is_some_and(|n| n > 0)
}
fn nullable_id(v: &Value) -> bool {
    v.is_null() || id(v)
}
fn flag(v: &Value) -> bool {
    v == 0 || v == 1
}
fn article_fields(v: &Value) -> bool {
    id(&v["id"])
        && v["article_title"].is_string()
        && fields(
            v,
            &[
                "sort_id",
                "label_id",
                "view_count",
                "like_count",
                "comment_count",
            ],
            count,
        )
        && fields(
            v,
            &["article_cover", "sort_name", "label_name", "create_time"],
            nullable,
        )
        && fields(v, &["recommend_status", "view_status"], flag)
}
pub fn article(v: &Value) -> bool {
    article_fields(v)
        && id(&v["user_id"])
        && v["article_content"].is_string()
        && fields(v, &["comment_status", "password_required"], flag)
        && fields(
            v,
            &["video_url", "username", "tips", "update_time"],
            nullable,
        )
}
pub fn summary(v: &Value) -> bool {
    article_fields(v) && fields(v, &["excerpt", "search_snippet"], nullable)
}
pub fn category(v: &Value) -> bool {
    id(&v["id"])
        && v["sort_name"].is_string()
        && fields(v, &["sort_description"], nullable)
        && v.get("priority").is_some_and(|n| n.is_null() || integer(n))
        && count(&v["article_count"])
}
pub fn label(v: &Value) -> bool {
    fields(v, &["id", "sort_id"], id)
        && v["label_name"].is_string()
        && fields(v, &["label_description"], nullable)
}
pub fn comment(v: &Value) -> bool {
    id(&v["id"])
        && v.get("user_id").is_some_and(nullable_id)
        && v["comment_content"].is_string()
        && fields(
            v,
            &["username", "avatar", "create_time", "parent_username"],
            nullable,
        )
        && count(&v["reply_count"])
}
fn admin_comment(v: &Value) -> bool {
    id(&v["id"])
        && count(&v["source"])
        && v.get("user_id").is_some_and(nullable_id)
        && v["comment_content"].is_string()
        && fields(v, &["comment_type", "username", "create_time"], nullable)
}
pub fn site(v: &Value) -> bool {
    fields(
        v,
        &[
            "web_name",
            "web_title",
            "notices",
            "background_image",
            "avatar",
            "random_cover",
        ],
        nullable,
    )
}
pub fn note(v: &Value) -> bool {
    id(&v["id"])
        && v.get("user_id").is_some_and(nullable_id)
        && v["content"].is_string()
        && flag(&v["is_public"])
        && fields(v, &["username", "image_path", "create_time"], nullable)
}
pub fn wall(v: &Value) -> bool {
    id(&v["id"])
        && v.get("user_id").is_some_and(nullable_id)
        && v["message"].is_string()
        && fields(
            v,
            &["username", "avatar", "image_path", "create_time"],
            nullable,
        )
}
pub fn link(v: &Value) -> bool {
    id(&v["id"])
        && fields(
            v,
            &[
                "title",
                "classify",
                "cover",
                "url",
                "introduction",
                "link_type",
                "create_time",
            ],
            nullable,
        )
}
pub fn family(v: &Value) -> bool {
    id(&v["id"])
        && fields(
            v,
            &[
                "bg_cover",
                "man_cover",
                "woman_cover",
                "man_name",
                "woman_name",
                "timing",
                "countdown_title",
                "countdown_time",
                "family_info",
            ],
            nullable,
        )
}
pub fn news(v: &Value) -> bool {
    id(&v["id"]) && v["content"].is_string() && fields(v, &["create_time"], nullable)
}
pub fn home(v: &Value) -> bool {
    id(&v["id"])
        && v["title"].is_string()
        && ["latest", "recommended", "category"].contains(&v["kind"].as_str().unwrap_or_default())
        && integer(&v["priority"])
        && v["enabled"].is_boolean()
        && if v["kind"] == "category" {
            id(&v["sort_id"])
        } else {
            v["sort_id"].is_null()
        }
}
pub fn resource(v: &Value) -> bool {
    id(&v["id"])
        && v.get("size").is_some_and(|n| n.is_null() || count(n))
        && v.get("status").is_some_and(|n| n.is_null() || integer(n))
        && fields(
            v,
            &["original_name", "path", "resource_type", "create_time"],
            nullable,
        )
}
pub fn upload(v: &Value) -> bool {
    let path = v["path"].as_str().unwrap_or_default();
    let valid_path = path
        .strip_prefix("/media/")
        .and_then(|s| s.split_once('.'))
        .is_some_and(|(hash, ext)| {
            hash.len() == 64
                && hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                && ["png", "jpg", "gif", "webp"].contains(&ext)
        });
    id(&v["id"])
        && valid_path
        && ["image/png", "image/jpeg", "image/gif", "image/webp"]
            .contains(&v["mime_type"].as_str().unwrap_or_default())
        && v["size"]
            .as_u64()
            .is_some_and(|n| n > 0 && n <= 10 * 1024 * 1024)
}
fn page(v: &Value, test: impl Fn(&Value) -> bool) -> bool {
    v["items"]
        .as_array()
        .is_some_and(|a| a.iter().all(&test) && a.len() <= v["size"].as_u64().unwrap_or(0) as usize)
        && count(&v["total"])
        && id(&v["page"])
        && v["page"].as_i64().unwrap_or(0) <= 100_000
        && id(&v["size"])
        && v["size"].as_i64().unwrap_or(0) <= 100
}
fn array(v: &Value, test: impl Fn(&Value) -> bool) -> bool {
    v.as_array().is_some_and(|a| a.iter().all(test))
}
pub fn validate(path: &str, method: &str, v: &Value) -> bool {
    let base = path.split('?').next().unwrap_or(path);
    if method == "DELETE" {
        return v["deleted"] == true;
    }
    if base.ends_with("/upload") {
        return upload(v);
    }
    if base.ends_with("/site/stats") {
        return fields(v, &["article_count", "view_count"], count);
    }
    if base.ends_with("/statistics") {
        return fields(v, &["articles", "comments", "members", "categories"], count);
    }
    if base.ends_with("/site") {
        return site(v) && (!base.contains("/content/") || v["author_name"].is_string());
    }
    if base.ends_with("/home-sections") {
        return array(v, home);
    }
    if base.ends_with("/categories") {
        return if method == "GET" {
            array(v, category)
        } else {
            category(v)
        };
    }
    if base.contains("/categories/") {
        return category(v);
    }
    if base.ends_with("/labels") {
        return if method == "GET" {
            array(v, |v| {
                label(v) && (base.contains("/content/") || count(&v["article_count"]))
            })
        } else {
            label(v)
        };
    }
    if base.contains("/labels/") {
        return label(v);
    }
    if base.ends_with("/letter") {
        return if base.contains("/content/") {
            fields(v, &["recipient", "sender", "content"], Value::is_string)
                && v["is_public"].is_boolean()
        } else {
            v.is_null() || fields(v, &["recipient", "sender", "content"], Value::is_string)
        };
    }
    if base.contains("/news") {
        return if base.contains("/content/") && base.ends_with("/news") && method == "GET" {
            page(v, news)
        } else if method == "GET" {
            array(v, news)
        } else {
            news(v)
        };
    }
    if base.ends_with("/access") {
        return flag(&v["password_required"]) && fields(v, &["tips"], nullable);
    }
    if base.ends_with("/articles") {
        return if method == "GET" {
            page(v, summary)
        } else {
            article(v)
        };
    }
    if base.contains("/articles/") {
        return article(v);
    }
    if base.ends_with("/links/classes") {
        return array(v, |v| v["classify"].is_string() && count(&v["count"]));
    }
    if base.ends_with("/links/page") || base.ends_with("/content/links") && method == "GET" {
        return page(v, |v| {
            link(v)
                && (!base.contains("/content/")
                    || v.get("status").is_some_and(|n| n.is_null() || flag(n)))
        });
    }
    if base.contains("/links/") || base.ends_with("/links") {
        return link(v)
            && (!base.contains("/content/")
                || v.get("status").is_some_and(|n| n.is_null() || flag(n)));
    }
    if base.ends_with("/family") {
        return if method == "GET" {
            array(v, |v| {
                family(v)
                    && (!base.contains("/content/")
                        || v.get("status").is_some_and(|n| n.is_null() || flag(n)))
            })
        } else {
            family(v) && v.get("status").is_some_and(|n| n.is_null() || flag(n))
        };
    }
    if base.contains("/family/") {
        return family(v) && v.get("status").is_some_and(|n| n.is_null() || flag(n));
    }
    if base.ends_with("/notes/page") || base.ends_with("/content/notes") && method == "GET" {
        return page(v, |v| {
            note(v) && (base.contains("/content/") || count(&v["like_count"]))
        });
    }
    if base.contains("/notes/") || base.ends_with("/notes") {
        return note(v);
    }
    if base.ends_with("/content/resources") {
        return page(v, resource);
    }
    if base.ends_with("/content/comments") {
        return page(v, admin_comment);
    }
    if base.ends_with("/content/tree-hole") {
        return page(v, wall);
    }
    if base.ends_with("/tree-hole") {
        return array(v, wall);
    }
    if base.ends_with("/tree-hole/guest") {
        return wall(v);
    }
    if base.contains("comments") {
        return if method == "GET" {
            page(v, comment)
        } else {
            comment(v)
        };
    }
    false
}
/// A valid page from a different selection must never replace the requested page.
pub fn matches_page(path: &str, v: &Value) -> bool {
    if !v.get("items").is_some_and(Value::is_array) {
        return true;
    }
    let query = path.split_once('?').map(|(_, q)| q).unwrap_or_default();
    for key in ["page", "size"] {
        if let Some(value) = query
            .split('&')
            .filter_map(|p| p.split_once('='))
            .find_map(|(k, v)| (k == key).then_some(v))
            && value.parse::<u64>().ok() != v[key].as_u64()
        {
            return false;
        }
    }
    let base = path.split('?').next().unwrap_or(path);
    (!base.contains("comments") && base != "/api/v1/notes/page")
        || v["size"].as_u64().is_some_and(|n| n <= 50)
}
#[cfg(test)]
mod tests {
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
        let label = json!({"id":1,"sort_id":1,"label_name":"Tag","label_description":null,"article_count":0});
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
}
