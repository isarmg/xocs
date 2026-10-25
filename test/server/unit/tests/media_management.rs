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

async fn upload_video_bytes(
    app: &Router,
    session: &(String, String),
    bytes: Vec<u8>,
    content_type: &str,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/content/upload-video")
                .header(header::HOST, "127.0.0.1:18881")
                .header(header::ORIGIN, "http://127.0.0.1:18881")
                .header("sec-fetch-site", "same-origin")
                .header(header::COOKIE, &session.0)
                .header("x-csrf-token", &session.1)
                .header(header::CONTENT_TYPE, content_type)
                .header("x-file-name", "background.webm")
                .body(Body::from(bytes))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn real_video_upload_range_and_site_references_share_managed_media_lifecycle() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let bytes = include_bytes!("../../../browser/fixtures/background.webm");
    // The request's media declaration does not determine the stored MIME type.
    let response =
        upload_video_bytes(&app, &session, bytes.to_vec(), "application/octet-stream").await;
    assert_eq!(response.status(), StatusCode::OK);
    let video = body_json(response).await;
    assert_eq!(video["mime_type"], "video/webm");
    assert_eq!(video["size"], bytes.len());
    assert!(video["path"].as_str().unwrap().ends_with(".webm"));
    let kind: String = sqlx::query_scalar("SELECT type FROM resource WHERE id=?")
        .bind(video["id"].as_i64().unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(kind, "video");
    let duplicate = upload_video_bytes(&app, &session, bytes.to_vec(), "video/webm").await;
    assert_eq!(body_json(duplicate).await, video);

    // A video above the image limit remains accepted. Streaming WebM allows
    // an unknown-length Segment and a trailing Void element for padding.
    let mut larger = bytes.to_vec();
    let segment = larger
        .windows(4)
        .position(|part| part == [0x18, 0x53, 0x80, 0x67])
        .unwrap();
    assert_eq!(larger[segment + 4], 1); // The fixture uses an eight-byte size.
    larger[segment + 4..segment + 12].copy_from_slice(&[1, 255, 255, 255, 255, 255, 255, 255]);
    let padding = 11 * 1024 * 1024;
    larger.push(0xec);
    larger.extend_from_slice(&(0x1000_0000_u32 | padding).to_be_bytes());
    larger.resize(larger.len() + padding as usize, 0);
    let uploaded = upload_video_bytes(&app, &session, larger, "video/webm").await;
    assert_eq!(uploaded.status(), StatusCode::OK);
    let larger = body_json(uploaded).await;
    assert!(larger["size"].as_u64().unwrap() > 10 * 1024 * 1024);
    assert_eq!(
        remove(&app, &session, &larger).await.status(),
        StatusCode::OK
    );

    let path = video["path"].as_str().unwrap();
    let range = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .header(header::RANGE, "bytes=1-8")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(range.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(range.headers()[header::CONTENT_TYPE], "video/webm");
    assert_eq!(
        range.headers()[header::CONTENT_RANGE],
        format!("bytes 1-8/{}", bytes.len())
    );
    assert_eq!(
        to_bytes(range.into_body(), 1024).await.unwrap().as_ref(),
        &bytes[1..9]
    );
    let head = app
        .clone()
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(head.status(), StatusCode::OK);
    assert_eq!(
        head.headers()[header::CONTENT_LENGTH],
        bytes.len().to_string()
    );
    assert!(to_bytes(head.into_body(), 1024).await.unwrap().is_empty());

    sqlx::query("INSERT INTO web_info(id,media) VALUES(1,?) ON CONFLICT(id) DO UPDATE SET media=excluded.media")
        .bind(json!({"toolbox_video":path}).to_string()).execute(&pool).await.unwrap();
    assert_eq!(
        remove(&app, &session, &video).await.status(),
        StatusCode::CONFLICT
    );
    assert!(image_file(directory.path(), &video).exists());
    sqlx::query("UPDATE web_info SET media='{}' WHERE id=1")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        remove(&app, &session, &video).await.status(),
        StatusCode::OK
    );
    assert!(!image_file(directory.path(), &video).exists());
    assert_eq!(get(&app, path).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn saved_article_video_fences_retain_uploaded_media_until_the_reference_is_removed() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let response = upload_video_bytes(
        &app,
        &session,
        include_bytes!("../../../browser/fixtures/background.webm").to_vec(),
        "video/webm",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let video = body_json(response).await;
    let path = video["path"].as_str().unwrap();
    sqlx::raw_sql("INSERT INTO sort(id,sort_name) VALUES(1,'Video'); INSERT INTO label(id,sort_id,label_name) VALUES(1,1,'Video');")
        .execute(&pool).await.unwrap();
    let mut draft = json!({
        "article_title":"Private video draft",
        "article_content":format!("Before\n\n```video\n{path}\n```\n\nAfter"),
        "sort_id":1,
        "label_id":1,
        "view_status":false,
        "recommend_status":false,
        "comment_status":true,
        "password":"TemporaryVideoPassphrase123"
    });
    let response = admin(
        &app,
        &session,
        "POST",
        "/api/v1/content/articles",
        draft.clone(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let article = body_json(response).await;
    let article_uri = format!("/api/v1/content/articles/{}", article["id"]);
    draft["password"] = Value::Null;

    for alias in [
        path.to_owned(),
        path.replace("/media/", "/media/./"),
        path.replace("/media/", "/media/nested/../"),
        path.replace("/media/", "/media/%2e/"),
        path.replace("/media/", "/media/nested/%2E%2e/"),
        format!(
            "http://127.0.0.1:18881{}?download=1#t=2",
            path.replace("/media/", "/media/./")
        ),
    ] {
        draft["article_content"] = json!(format!("Before\n\n```video\n{alias}\n```\n\nAfter"));
        let response = admin(&app, &session, "PUT", &article_uri, draft.clone()).await;
        assert_eq!(response.status(), StatusCode::OK, "{alias}");
        let saved = body_json(response).await;
        assert_eq!(saved["article_content"], draft["article_content"]);
        assert_eq!(saved["view_status"], 0);
        assert_eq!(saved["password_required"], 1);
        assert_eq!(
            remove(&app, &session, &video).await.status(),
            StatusCode::CONFLICT,
            "saved video fence: {alias}"
        );
        assert!(image_file(directory.path(), &video).exists());
        assert_eq!(get(&app, path).await.status(), StatusCode::OK);
    }

    draft["article_content"] = json!("The video was removed from this draft.");
    assert_eq!(
        admin(&app, &session, "PUT", &article_uri, draft)
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        remove(&app, &session, &video).await.status(),
        StatusCode::OK
    );
    assert!(!image_file(directory.path(), &video).exists());
    assert_eq!(get(&app, path).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn video_upload_rejects_documents_images_and_oversized_bodies() {
    let (_directory, app, pool) = setup().await;
    let anonymous = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/content/upload-video")
                .header(header::CONTENT_TYPE, "video/webm")
                .body(Body::from(
                    include_bytes!("../../../browser/fixtures/background.webm").as_slice(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    let session = login(&app).await;
    for (bytes, mime) in [
        (b"<html>not video</html>".to_vec(), "video/mp4"),
        (b"\x89PNG\r\n\x1a\nimage".to_vec(), "video/webm"),
        (Vec::new(), "video/webm"),
    ] {
        assert_eq!(
            upload_video_bytes(&app, &session, bytes, mime)
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    let oversized = vec![0; 50 * 1024 * 1024 + 1];
    assert_eq!(
        upload_video_bytes(&app, &session, oversized, "video/mp4")
            .await
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM resource")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn browser_normalized_media_aliases_in_saved_settings_and_markdown_block_deletion() {
    let (directory, app, pool) = setup().await;
    let session = login(&app).await;
    let image = upload(&app, &session, "normalized aliases").await;
    let path = image["path"].as_str().unwrap();
    let response = admin(&app, &session, "GET", "/api/v1/content/site", json!({})).await;
    assert_eq!(response.status(), StatusCode::OK);
    let mut site = body_json(response).await;
    site["web_name"] = "Alias fixture".into();
    for alias in [
        path.replace("/media/", "/media/./"),
        path.replace("/media/", "/media/nested/../"),
        path.replace("/media/", "/media/%2e/"),
        path.replace("/media/", "/media/nested/%2E%2e/"),
    ] {
        for field in ["avatar", "media", "random_cover"] {
            match field {
                "media" => site["media"]["logo"] = json!(alias),
                "random_cover" => site[field] = json!(json!([alias]).to_string()),
                _ => site[field] = json!(alias),
            }
            let response = admin(&app, &session, "PUT", "/api/v1/content/site", site.clone()).await;
            assert_eq!(response.status(), StatusCode::OK, "{field}: {alias}");
            assert_eq!(
                remove(&app, &session, &image).await.status(),
                StatusCode::CONFLICT,
                "saved {field}: {alias}"
            );
            assert!(image_file(directory.path(), &image).exists());
            match field {
                "media" => site["media"]["logo"] = Value::Null,
                _ => site[field] = Value::Null,
            }
            assert_eq!(
                admin(&app, &session, "PUT", "/api/v1/content/site", site.clone())
                    .await
                    .status(),
                StatusCode::OK
            );
        }
    }
    sqlx::raw_sql("INSERT INTO sort(id,sort_name) VALUES(1,'Aliases'); INSERT INTO label(id,sort_id,label_name) VALUES(1,1,'Aliases');")
        .execute(&pool).await.unwrap();
    for alias in [
        path.replace("/media/", "/media/./"),
        path.replace("/media/", "/media/nested/../"),
        path.replace("/media/", "/media/%2e/"),
        path.replace("/media/", "/media/nested/%2E%2e/"),
    ] {
        let response = admin(&app, &session, "POST", "/api/v1/content/articles", json!({
            "article_title":"Alias", "article_content":format!("![saved][upload]\n\n[upload]: {alias}"),
            "sort_id":1,"label_id":1,"view_status":true,"recommend_status":false,"comment_status":true
        })).await;
        assert_eq!(response.status(), StatusCode::OK);
        let article = body_json(response).await;
        assert_eq!(
            remove(&app, &session, &image).await.status(),
            StatusCode::CONFLICT,
            "saved Markdown: {alias}"
        );
        assert_eq!(
            admin(
                &app,
                &session,
                "DELETE",
                &format!("/api/v1/content/articles/{}", article["id"]),
                Value::Null
            )
            .await
            .status(),
            StatusCode::OK
        );
    }
    assert_eq!(
        remove(&app, &session, &image).await.status(),
        StatusCode::OK
    );
    assert!(!image_file(directory.path(), &image).exists());
    assert_eq!(get(&app, path).await.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn uploads_reject_malformed_names_without_publishing_media() {
    let (_directory, app, pool) = setup().await;
    let session = login(&app).await;
    for name in ["bad%FF.png", "..%2Fprivate.png", &"x".repeat(201)] {
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
                    .header("x-file-name", name)
                    .body(Body::from(b"\x89PNG\r\n\x1a\nfixture".as_slice()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(body_json(response).await["code"], "bad_request");
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
