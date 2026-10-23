use super::*;
use serde_json::{Value, json};

async fn login(app: &Router) -> (String, String) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::HOST, "127.0.0.1:18881")
                .header(header::ORIGIN, "http://127.0.0.1:18881")
                .header("sec-fetch-site", "same-origin")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"username":"admin","password":"TemporaryPassphrase123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap().split(';').next().unwrap())
        .collect::<Vec<_>>()
        .join("; ");
    let value = body_json(response).await;
    (cookie, value["csrf_token"].as_str().unwrap().to_owned())
}

async fn admin(
    app: &Router,
    session: &(String, String),
    method: &str,
    path: &str,
    value: Value,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(header::HOST, "127.0.0.1:18881")
                .header(header::ORIGIN, "http://127.0.0.1:18881")
                .header("sec-fetch-site", "same-origin")
                .header(header::COOKIE, &session.0)
                .header("x-csrf-token", &session.1)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(value.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn get(app: &Router, path: &str) -> Response {
    app.clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn upload(app: &Router, session: &(String, String), marker: &str) -> Value {
    let bytes = [b"\x89PNG\r\n\x1a\n".as_slice(), marker.as_bytes()].concat();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/content/upload")
                .header(header::HOST, "127.0.0.1:18881")
                .header(header::ORIGIN, "http://127.0.0.1:18881")
                .header("sec-fetch-site", "same-origin")
                .header(header::COOKIE, &session.0)
                .header("x-csrf-token", &session.1)
                .header(header::CONTENT_TYPE, "image/png")
                .header("x-file-name", "fixture.png")
                .body(Body::from(bytes))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn remove(app: &Router, session: &(String, String), image: &Value) -> Response {
    admin(
        app,
        session,
        "DELETE",
        &format!("/api/v1/content/resources/{}", image["id"]),
        json!({"path":image["path"]}),
    )
    .await
}

fn image_file(directory: &std::path::Path, image: &Value) -> std::path::PathBuf {
    directory.join(
        image["path"]
            .as_str()
            .unwrap()
            .strip_prefix("/media/")
            .unwrap(),
    )
}

#[tokio::test]
async fn unused_media_requires_auth_csrf_and_current_object_identity_then_really_disappears() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let image = upload(&app, &session, "first").await;
    let uri = format!("/api/v1/content/resources/{}", image["id"]);
    assert_eq!(
        get(&app, image["path"].as_str().unwrap()).await.status(),
        StatusCode::OK
    );
    let anonymous = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"path":image["path"]}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    let no_csrf = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&uri)
                .header(header::HOST, "127.0.0.1:18881")
                .header(header::ORIGIN, "http://127.0.0.1:18881")
                .header("sec-fetch-site", "same-origin")
                .header(header::COOKIE, &session.0)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"path":image["path"]}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(no_csrf.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        admin(&app, &session, "DELETE", &uri, json!({}))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    let response = remove(&app, &session, &image).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, json!({"deleted":true}));
    assert!(!image_file(directory.path(), &image).exists());
    assert_eq!(
        get(&app, image["path"].as_str().unwrap()).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM resource")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::NOT_FOUND
    );
    // INTEGER PRIMARY KEY may reuse a removed maximum ID. The old path must
    // not authorize deletion of a new object that happens to receive that ID.
    let replacement = upload(&app, &session, "different image").await;
    assert_eq!(replacement["id"], image["id"]);
    assert_ne!(replacement["path"], image["path"]);
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::CONFLICT
    );
    assert!(image_file(directory.path(), &replacement).exists());
    assert_eq!(
        get(&app, replacement["path"].as_str().unwrap())
            .await
            .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn saved_draft_and_every_active_media_field_block_deletion_even_after_nul_separators() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let image = upload(&app, &session, "referenced").await;
    let path = image["path"].as_str().unwrap();
    let cases = [
        (
            "article cover",
            "INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,article_cover,view_status) VALUES(1,1,1,1,'Draft','Body',?,0)",
            "DELETE FROM article",
        ),
        (
            "article video after NUL",
            "INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,video_url) VALUES(1,1,1,1,'Video','Body',?)",
            "DELETE FROM article",
        ),
        (
            "article body after NUL",
            "INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,1,1,'Draft',?)",
            "DELETE FROM article",
        ),
        (
            "site avatar after NUL",
            "INSERT INTO web_info(id,avatar) VALUES(1,?)",
            "DELETE FROM web_info",
        ),
        (
            "family woman_cover after NUL",
            "INSERT INTO family(id,user_id,woman_cover,status) VALUES(1,1,?,0)",
            "DELETE FROM family",
        ),
        (
            "album cover",
            "INSERT INTO resource_path(id,cover,type,status) VALUES(1,?,'lovePhoto',0)",
            "DELETE FROM resource_path",
        ),
        (
            "link URL after NUL",
            "INSERT INTO resource_path(id,url,type,status) VALUES(1,?,'favorites',0)",
            "DELETE FROM resource_path",
        ),
        (
            "private note image",
            "INSERT INTO wei_yan(id,user_id,type,content,image_path,is_public) VALUES(1,1,'friend','Private',?,0)",
            "DELETE FROM wei_yan",
        ),
        (
            "guest message image",
            "INSERT INTO tree_hole(id,message,image_path) VALUES(1,'Image',?)",
            "DELETE FROM tree_hole",
        ),
        (
            "comment content",
            "INSERT INTO comment(id,source,type,comment_content) VALUES(1,1,'article',?)",
            "DELETE FROM comment",
        ),
    ];
    for (label, insert, cleanup) in cases {
        sqlx::query(insert).bind(path).execute(&pool).await.unwrap();
        assert_eq!(
            remove(&app, &session, &image).await.status(),
            StatusCode::CONFLICT,
            "{label}"
        );
        assert!(image_file(directory.path(), &image).exists(), "{label}");
        sqlx::query(cleanup).execute(&pool).await.unwrap();
    }
    sqlx::query("UPDATE user SET avatar=? WHERE id=1")
        .bind(path)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::CONFLICT
    );
    sqlx::query("UPDATE user SET avatar=NULL WHERE id=1")
        .execute(&pool)
        .await
        .unwrap();
    let letter = json!({"recipient":"Partner","sender":"Owner","content":format!("Saved picture {path}"),"is_public":false});
    assert_eq!(
        admin(&app, &session, "PUT", "/api/v1/content/letter", letter)
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::CONFLICT,
        "private letter content is retained"
    );
    sqlx::query("DELETE FROM wei_yan WHERE type='letter'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn encoded_markdown_and_json_image_references_cannot_be_deleted() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let image = upload(&app, &session, "encoded").await;
    let path = image["path"].as_str().unwrap();
    let percent = path.replace('/', "%2F");
    let entities = path.replace('/', "&#x2f;").replace('.', "&#46;");
    for content in [
        format!("![saved]({percent})"),
        format!("![saved]({entities})"),
        format!("![saved]({})", path.replace('/', "\\/").replace('.', "\\.")),
        format!(
            "![saved][upload]\n\n[upload]: {}",
            path.replace('/', "\\/").replace('.', "\\.")
        ),
    ] {
        sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,1,1,'Encoded',?)").bind(content).execute(&pool).await.unwrap();
        assert_eq!(
            remove(&app, &session, &image).await.status(),
            StatusCode::CONFLICT
        );
        sqlx::query("DELETE FROM article")
            .execute(&pool)
            .await
            .unwrap();
    }
    let json_path = json!([path]).to_string();
    let unicode_path = format!(
        "[\"{}\"]",
        path.chars()
            .map(|ch| format!("\\u{:04x}", u32::from(ch)))
            .collect::<String>()
    );
    for encoded in [
        json_path.replace('/', "\\/"),
        json_path.replace('/', "\\u002f"),
        unicode_path,
    ] {
        assert_eq!(
            serde_json::from_str::<Value>(&encoded).unwrap(),
            json!([path])
        );
        sqlx::query("INSERT INTO web_info(id,random_cover) VALUES(1,?)")
            .bind(&encoded)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            remove(&app, &session, &image).await.status(),
            StatusCode::CONFLICT,
            "random_cover JSON {encoded}"
        );
        sqlx::query("DELETE FROM web_info")
            .execute(&pool)
            .await
            .unwrap();
        let promotion = format!(
            "[\"推送封面：{}\"]",
            encoded.trim_start_matches("[\"").trim_end_matches("\"]")
        );
        sqlx::query("INSERT INTO web_info(id,notices) VALUES(1,?)")
            .bind(promotion)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            remove(&app, &session, &image).await.status(),
            StatusCode::CONFLICT,
            "notice JSON {encoded}"
        );
        sqlx::query("DELETE FROM web_info")
            .execute(&pool)
            .await
            .unwrap();
    }
    assert!(image_file(directory.path(), &image).exists());
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn deleted_article_without_restore_entry_does_not_pin_an_unused_image_forever() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let image = upload(&app, &session, "deleted article").await;
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,article_cover,deleted) VALUES(1,1,1,1,'Deleted','Body',?,1)")
        .bind(image["path"].as_str().unwrap()).execute(&pool).await.unwrap();
    assert_eq!(
        get(&app, "/api/v1/articles/1").await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::OK
    );
    assert!(!image_file(directory.path(), &image).exists());
}

#[tokio::test]
async fn media_deletion_rejects_foreign_paths_types_symlinks_and_hard_links() {
    use std::os::unix::fs::symlink;
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let image = upload(&app, &session, "unsafe paths").await;
    let original = image_file(directory.path(), &image);
    let uri = format!("/api/v1/content/resources/{}", image["id"]);
    sqlx::query("UPDATE resource SET store_type='remote' WHERE id=?")
        .bind(image["id"].as_i64().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::CONFLICT
    );
    sqlx::query("UPDATE resource SET store_type='local',type='video' WHERE id=?")
        .bind(image["id"].as_i64().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::CONFLICT
    );
    sqlx::query(
        "UPDATE resource SET type='image',path='https://example.test/image.png' WHERE id=?",
    )
    .bind(image["id"].as_i64().unwrap())
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        admin(
            &app,
            &session,
            "DELETE",
            &uri,
            json!({"path":"https://example.test/image.png"})
        )
        .await
        .status(),
        StatusCode::CONFLICT
    );
    sqlx::query("UPDATE resource SET path=? WHERE id=?")
        .bind(image["path"].as_str().unwrap())
        .bind(image["id"].as_i64().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(outside.path(), b"outside must survive").unwrap();
    let bytes = std::fs::read(&original).unwrap();
    std::fs::remove_file(&original).unwrap();
    symlink(outside.path(), &original).unwrap();
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(
        std::fs::read(outside.path()).unwrap(),
        b"outside must survive"
    );
    assert!(
        std::fs::symlink_metadata(&original)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    std::fs::remove_file(&original).unwrap();
    std::fs::write(&original, &bytes).unwrap();
    let alias = directory.path().join("outside-alias");
    std::fs::hard_link(&original, &alias).unwrap();
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
    assert_eq!(std::fs::read(&original).unwrap(), bytes);
    assert_eq!(std::fs::read(&alias).unwrap(), bytes);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM resource")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    std::fs::remove_file(alias).unwrap();
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn concurrent_same_hash_upload_and_delete_never_leave_a_registered_missing_file() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    for _ in 0..4 {
        let image = upload(&app, &session, "same concurrent bytes").await;
        let (deleted, uploaded) = tokio::join!(
            remove(&app, &session, &image),
            upload(&app, &session, "same concurrent bytes")
        );
        assert_eq!(deleted.status(), StatusCode::OK);
        assert_eq!(uploaded["path"], image["path"]);
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource WHERE path=?")
            .bind(image["path"].as_str().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
        let present = image_file(directory.path(), &image).exists();
        assert_eq!(present, count == 1);
        assert_eq!(
            get(&app, image["path"].as_str().unwrap()).await.status(),
            if present {
                StatusCode::OK
            } else {
                StatusCode::NOT_FOUND
            }
        );
        assert!(!std::fs::read_dir(directory.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".blog-delete-")
        }));
    }
}

#[tokio::test]
async fn deletion_quarantine_recovery_uses_committed_database_state_and_stays_private() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let retained = upload(&app, &session, "retained DB row").await;
    let original = image_file(directory.path(), &retained);
    let name = original.file_name().unwrap().to_str().unwrap();
    let pending = directory.path().join(format!(".blog-delete-{name}"));
    let bytes = std::fs::read(&original).unwrap();
    std::fs::rename(&original, &pending).unwrap();
    assert_eq!(
        get(&app, &format!("/media/.blog-delete-{name}"))
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    crate::media::recover_deletions(&pool, directory.path())
        .await
        .unwrap();
    assert_eq!(std::fs::read(&original).unwrap(), bytes);
    assert!(!pending.exists());
    assert_eq!(
        get(&app, retained["path"].as_str().unwrap()).await.status(),
        StatusCode::OK
    );
    let removed = upload(&app, &session, "committed DB deletion").await;
    let removed_file = image_file(directory.path(), &removed);
    let removed_name = removed_file.file_name().unwrap().to_str().unwrap();
    let removed_pending = directory
        .path()
        .join(format!(".blog-delete-{removed_name}"));
    std::fs::rename(&removed_file, &removed_pending).unwrap();
    sqlx::query("DELETE FROM resource WHERE id=?")
        .bind(removed["id"].as_i64().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    crate::media::recover_deletions(&pool, directory.path())
        .await
        .unwrap();
    assert!(!removed_pending.exists());
    assert!(!removed_file.exists());
    assert_eq!(
        get(&app, removed["path"].as_str().unwrap()).await.status(),
        StatusCode::NOT_FOUND
    );
    // Recovery may encounter both names after an interrupted prior restore.
    // Equal bytes permit cleanup; a different original must never be replaced.
    std::fs::copy(&original, &pending).unwrap();
    crate::media::recover_deletions(&pool, directory.path())
        .await
        .unwrap();
    assert!(!pending.exists());
    std::fs::write(&pending, b"conflicting quarantine").unwrap();
    assert!(
        crate::media::recover_deletions(&pool, directory.path())
            .await
            .is_err()
    );
    assert_eq!(std::fs::read(&original).unwrap(), bytes);
    assert!(pending.exists());
}

#[tokio::test]
async fn removed_public_pages_and_member_management_routes_are_absent() {
    let (_directory, app, _pool) = setup().await;
    let session = login(&app).await;
    for path in ["/about", "/weiYan"] {
        assert_eq!(
            get(&app, path).await.status(),
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    for path in ["/api/v1/content/users", "/api/v1/content/users/1/status"] {
        for method in ["GET", "PUT"] {
            assert_eq!(
                admin(&app, &session, method, path, json!({"active":false}))
                    .await
                    .status(),
                StatusCode::NOT_FOUND,
                "{method} {path}"
            );
        }
    }
}
