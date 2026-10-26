use super::*;
use serde_json::{Value, json};

#[path = "commit_responses.rs"]
mod commit_responses;

async fn session(app: &Router) -> (String, String) {
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

async fn public(app: &Router, path: &str) -> Value {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "{path}");
    body_json(response).await
}

async fn uploaded_image(app: &Router, session: &(String, String)) -> String {
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
                .header("x-file-name", "note.png")
                .body(Body::from(b"\x89PNG\r\n\x1a\nfixture".to_vec()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await["path"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn note(content: &str, image: Option<&str>, visible: bool) -> Value {
    json!({"content":content,"image_path":image,"is_public":visible})
}

fn letter(visible: bool) -> Value {
    json!({"recipient":"Partner","sender":"Owner","content":"First line\nSecond line <literal>","is_public":visible})
}

struct PausedAdministratorBody {
    entered: Option<tokio::sync::oneshot::Sender<()>>,
    release: tokio::sync::oneshot::Receiver<axum::body::Bytes>,
    sent: bool,
}

impl http_body::Body for PausedAdministratorBody {
    type Data = axum::body::Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        use std::{future::Future, task::Poll};

        if let Some(entered) = self.entered.take() {
            entered.send(()).expect("body observer remains available");
        }
        if self.sent {
            return Poll::Ready(None);
        }
        match std::pin::Pin::new(&mut self.release).poll(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(bytes) => {
                self.sent = true;
                Poll::Ready(Some(Ok(http_body::Frame::data(
                    bytes.expect("test releases the paused request body"),
                ))))
            }
        }
    }
}

async fn pause_administrator_request(
    app: &Router,
    session: &(String, String),
    path: &str,
    content_type: &str,
) -> (
    tokio::task::JoinHandle<Response>,
    tokio::sync::oneshot::Sender<axum::body::Bytes>,
) {
    let (entered, observed) = tokio::sync::oneshot::channel();
    let (release, resume) = tokio::sync::oneshot::channel();
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header(header::HOST, "127.0.0.1:18881")
        .header(header::ORIGIN, "http://127.0.0.1:18881")
        .header("sec-fetch-site", "same-origin")
        .header(header::COOKIE, &session.0)
        .header("x-csrf-token", &session.1)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::new(PausedAdministratorBody {
            entered: Some(entered),
            release: resume,
            sent: false,
        }))
        .unwrap();
    let app = app.clone();
    let task = tokio::spawn(async move { app.oneshot(request).await.unwrap() });
    // A request body is consumed only after the route's authentication gate.
    tokio::time::timeout(std::time::Duration::from_secs(5), observed)
        .await
        .expect("administrator request reaches body extraction")
        .expect("administrator request announces body extraction");
    (task, release)
}

#[tokio::test]
async fn content_mutations_recheck_revocable_authority_after_request_body_waits() {
    for change in [
        "UPDATE _common_admin_sessions SET revoked_at_micros=1",
        "UPDATE _common_admin_sessions SET idle_expires_at_micros=created_at_micros+1",
        "UPDATE _common_administrators SET active=0",
        "UPDATE _common_administrators SET session_version=session_version+1",
    ] {
        let (_directory, app, pool) = setup().await;
        let session = session(&app).await;
        let (request, release) = pause_administrator_request(
            &app,
            &session,
            "/api/v1/content/categories",
            "application/json",
        )
        .await;
        sqlx::query(change).execute(&pool).await.unwrap();
        release
            .send(json!({"name":"Rejected category"}).to_string().into())
            .unwrap();
        let response = request.await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{change}");
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sort")
                .fetch_one(&pool)
                .await
                .unwrap(),
            0,
            "{change}"
        );
    }
}

#[tokio::test]
async fn media_upload_rechecks_authority_before_publishing_a_file_or_resource() {
    use sha2::{Digest, Sha256};

    let (directory, app, pool) = setup().await;
    let session = session(&app).await;
    let (request, release) =
        pause_administrator_request(&app, &session, "/api/v1/content/upload", "image/png").await;
    sqlx::query("UPDATE _common_admin_sessions SET revoked_at_micros=1")
        .execute(&pool)
        .await
        .unwrap();
    let bytes = b"\x89PNG\r\n\x1a\nrevoked upload";
    release.send(bytes.as_slice().into()).unwrap();
    assert_eq!(request.await.unwrap().status(), StatusCode::UNAUTHORIZED);
    assert!(
        !directory
            .path()
            .join(format!("{}.png", hex::encode(Sha256::digest(bytes))))
            .exists()
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
async fn article_mutation_rechecks_authority_after_waiting_for_a_database_writer() {
    let (_directory, app, pool) = setup_with_pool_limit(2).await;
    let session = session(&app).await;
    sqlx::raw_sql("INSERT INTO sort(id,sort_name) VALUES(1,'Category'); INSERT INTO label(id,sort_id,label_name) VALUES(1,1,'Label');")
        .execute(&pool).await.unwrap();
    let (request, release) = pause_administrator_request(
        &app,
        &session,
        "/api/v1/content/articles",
        "application/json",
    )
    .await;
    let mut writer = pool.begin_with("BEGIN IMMEDIATE").await.unwrap();
    sqlx::query("UPDATE _common_administrators SET session_version=session_version+1")
        .execute(&mut *writer)
        .await
        .unwrap();
    release
        .send(
            json!({"article_title":"Rejected article","article_content":"Body","sort_id":1,"label_id":1,"view_status":true,"recommend_status":false,"comment_status":true})
                .to_string().into(),
        )
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while pool.num_idle() != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("article request waits for the held database writer");
    assert!(!request.is_finished());
    writer.commit().await.unwrap();
    assert_eq!(request.await.unwrap().status(), StatusCode::UNAUTHORIZED);
    for table in ["article", "article_submission"] {
        assert_eq!(
            sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!(
                "SELECT COUNT(*) FROM {table}"
            )))
            .fetch_one(&pool)
            .await
            .unwrap(),
            0,
            "{table}"
        );
    }
}

#[tokio::test]
async fn malformed_reply_floor_is_rejected_without_creating_a_thread_or_receipt() {
    let (_directory, app, pool) = setup().await;
    sqlx::raw_sql(
        "INSERT INTO comment(id,source,type,parent_comment_id,floor_comment_id,comment_content) VALUES
         (1,0,'message',0,NULL,'Root'),(2,0,'message',1,NULL,'Malformed reply');",
    )
    .execute(&pool)
    .await
    .unwrap();
    let response = anonymous_post(
        &app,
        "/api/v1/message-comments",
        json!({"content":"Another reply","parent_comment_id":2}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let counts: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM comment),(SELECT COUNT(*) FROM comment_submission)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (2, 0));
}

#[tokio::test]
async fn validated_letter_limits_cover_worst_case_json_escaping() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    let content = json!({
        "recipient":"\0".repeat(100),
        "sender":"\0".repeat(100),
        "content":"\0".repeat(16_384),
        "is_public":false,
    });
    let response = admin(
        &app,
        &session,
        "PUT",
        "/api/v1/content/letter",
        content.clone(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, content);
    let stored: String = sqlx::query_scalar("SELECT content FROM wei_yan WHERE type='letter'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(stored.len() < 128 * 1024);
    let response = admin(&app, &session, "GET", "/api/v1/content/letter", Value::Null).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, content);
}

#[tokio::test]
async fn home_section_targets_are_parsed_before_replacing_the_existing_sections() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    sqlx::query("INSERT INTO sort(id,sort_name) VALUES(1,'Category')")
        .execute(&pool)
        .await
        .unwrap();
    let before = public(&app, "/api/v1/home-sections").await;
    for (kind, sort_id) in [
        ("latest", json!(1)),
        ("recommended", json!(1)),
        ("category", Value::Null),
        ("category", json!(999)),
        ("unknown", Value::Null),
    ] {
        let response = admin(
            &app,
            &session,
            "PUT",
            "/api/v1/content/home-sections",
            json!([{"title":"Section","kind":kind,"sort_id":sort_id,"priority":0,"enabled":true}]),
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{kind}");
        assert_eq!(public(&app, "/api/v1/home-sections").await, before);
    }
    let response = admin(
        &app,
        &session,
        "PUT",
        "/api/v1/content/home-sections",
        json!([
            {"title":" Latest ","kind":"latest","sort_id":null,"priority":0,"enabled":true},
            {"title":"Recommended","kind":"recommended","sort_id":null,"priority":1,"enabled":true},
            {"title":"Category","kind":"category","sort_id":1,"priority":2,"enabled":true},
        ]),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let sections = body_json(response).await;
    assert_eq!(sections[0]["title"], "Latest");
    assert_eq!(sections[0]["kind"], "latest");
    assert!(sections[0]["sort_id"].is_null());
    assert_eq!(sections[1]["kind"], "recommended");
    assert_eq!(sections[2]["kind"], "category");
    assert_eq!(sections[2]["sort_id"], 1);
}

#[tokio::test]
async fn link_kinds_are_parsed_at_request_boundaries_before_database_writes() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    let link = |kind| {
        json!({
            "title": kind,
            "classify": "group",
            "cover": null,
            "url": null,
            "introduction": null,
            "link_type": kind,
            "status": true,
        })
    };
    for kind in ["favorites", "friendUrl", "lovePhoto", "funny"] {
        let response = admin(&app, &session, "POST", "/api/v1/content/links", link(kind)).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_json(response).await["link_type"], kind);
        let page = public(&app, &format!("/api/v1/links/page?kind={kind}")).await;
        assert_eq!(page["total"], 1);
        assert_eq!(page["items"][0]["link_type"], kind);
    }
    let all = admin(&app, &session, "GET", "/api/v1/content/links", Value::Null).await;
    assert_eq!(all.status(), StatusCode::OK);
    assert_eq!(body_json(all).await["total"], 4);
    assert_eq!(public(&app, "/api/v1/links/classes").await[0]["count"], 1);
    for path in [
        "/api/v1/links/page?kind=unknown",
        "/api/v1/links/classes?kind=unknown",
        "/api/v1/content/links?kind=unknown",
    ] {
        let response = admin(&app, &session, "GET", path, Value::Null).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{path}");
    }
    for (method, path) in [
        ("POST", "/api/v1/content/links"),
        ("PUT", "/api/v1/content/links/1"),
    ] {
        let response = admin(&app, &session, method, path, link("unknown")).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT title,type FROM resource_path ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0], ("favorites".into(), "favorites".into()));
}

#[tokio::test]
async fn notes_images_visibility_and_edits_complete_real_request_flow() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    let image = uploaded_image(&app, &session).await;
    let response = admin(
        &app,
        &session,
        "POST",
        "/api/v1/content/notes",
        note(" Private draft ", Some(&image), false),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let created = body_json(response).await;
    let id = created["id"].as_i64().unwrap();
    assert_eq!(created["content"], "Private draft");
    assert_eq!(created["image_path"], image);
    assert_eq!(created["is_public"], 0);
    assert_eq!(public(&app, "/api/v1/notes/page").await["total"], 0);
    let response = admin(
        &app,
        &session,
        "PUT",
        &format!("/api/v1/content/notes/{id}"),
        note("Edited and published", Some(&image), true),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let edited = body_json(response).await;
    assert_eq!(edited["id"], id);
    assert_eq!(edited["create_time"], created["create_time"]);
    let visible = public(&app, "/api/v1/notes/page").await;
    assert_eq!(visible["items"][0]["content"], "Edited and published");
    assert_eq!(visible["items"][0]["image_path"], image);
    let response = admin(
        &app,
        &session,
        "PUT",
        &format!("/api/v1/content/notes/{id}"),
        note("Hidden again", None, false),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(body_json(response).await["image_path"].is_null());
    assert_eq!(public(&app, "/api/v1/notes/page").await["total"], 0);
    let response = admin(
        &app,
        &session,
        "DELETE",
        &format!("/api/v1/content/notes/{id}"),
        json!({}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wei_yan WHERE type='friend'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn notes_bad_image_and_shape_are_rejected_without_mutation() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    let image = uploaded_image(&app, &session).await;
    for invalid in [
        note("", None, true),
        note(&"界".repeat(1025), None, true),
        note("External", Some("https://example.test/image.png"), true),
        note(
            "Missing",
            Some(&format!("/media/{}.png", "0".repeat(64))),
            true,
        ),
        note("Traversal", Some("/media/../image.png"), true),
        json!({"content":"missing visibility","image_path":null}),
        json!({"content":"missing image field","is_public":true}),
        json!({"content":"forged owner","image_path":null,"is_public":true,"user_id":2}),
    ] {
        let response = admin(&app, &session, "POST", "/api/v1/content/notes", invalid).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    sqlx::query("UPDATE resource SET status=0 WHERE path=?")
        .bind(&image)
        .execute(&pool)
        .await
        .unwrap();
    let response = admin(
        &app,
        &session,
        "POST",
        "/api/v1/content/notes",
        note("Inactive image", Some(&image), true),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wei_yan")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    sqlx::query("INSERT INTO wei_yan(id,user_id,type,content,is_public) VALUES(10,1,'news','Different type',1)").execute(&pool).await.unwrap();
    let response = admin(
        &app,
        &session,
        "PUT",
        "/api/v1/content/notes/10",
        note("Cannot overwrite news", None, true),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT content FROM wei_yan WHERE id=10")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "Different type"
    );
}

#[tokio::test]
async fn site_author_name_updates_business_content_but_not_credentials() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    let response = admin(&app, &session, "GET", "/api/v1/content/site", json!({})).await;
    assert_eq!(response.status(), StatusCode::OK);
    let mut site = body_json(response).await;
    assert_eq!(site["author_name"], "site-author");
    assert!(site.get("footer").is_none());
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM web_info")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0,
        "GET must not create a site row"
    );
    site["web_name"] = "My site".into();
    site["author_name"] = "展示作者".into();
    let response = admin(&app, &session, "PUT", "/api/v1/content/site", site.clone()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["author_name"], "展示作者");
    let response = admin(
        &app,
        &session,
        "POST",
        "/api/v1/content/notes",
        note("Named note", None, true),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["username"], "展示作者");
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,1,1,'Named article','Body')").execute(&pool).await.unwrap();
    assert_eq!(
        public(&app, "/api/v1/articles/1").await["username"],
        "展示作者"
    );
    assert!(
        public(&app, "/api/v1/site")
            .await
            .get("author_name")
            .is_none(),
        "public site contract remains unchanged"
    );
    assert!(public(&app, "/api/v1/site").await.get("footer").is_none());
    let mut removed_field = site.clone();
    removed_field["footer"] = "Removed page footer".into();
    assert_eq!(
        admin(&app, &session, "PUT", "/api/v1/content/site", removed_field)
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    // Administrator credentials live in shared tables, not this display identity.
    let _fresh_session = self::session(&app).await;
    sqlx::query("INSERT INTO user(id,username) VALUES(2,'reserved-name')")
        .execute(&pool)
        .await
        .unwrap();
    site["author_name"] = "reserved-name".into();
    site["web_name"] = "Must not commit".into();
    let response = admin(&app, &session, "PUT", "/api/v1/content/site", site.clone()).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(public(&app, "/api/v1/site").await["web_name"], "My site");
    for name in [" ".to_owned(), "界".repeat(101)] {
        site["author_name"] = name.into();
        assert_eq!(
            admin(&app, &session, "PUT", "/api/v1/content/site", site.clone())
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
}

#[tokio::test]
async fn site_media_is_explicit_persistent_and_clearable_without_default_assets() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    let empty = public(&app, "/api/v1/site").await;
    let keys = [
        "logo",
        "journey_background",
        "journal_background",
        "guestbook_background",
        "love_letter_background",
        "love_moments_cover",
        "love_photos_cover",
        "love_wishes_cover",
        "toolbox_background",
        "toolbox_video",
    ];
    assert_eq!(empty["media"].as_object().unwrap().len(), keys.len());
    for key in keys {
        assert!(empty["media"][key].is_null(), "{key} must be unconfigured");
    }
    let response = admin(&app, &session, "GET", "/api/v1/content/site", json!({})).await;
    let mut site = body_json(response).await;
    site["web_name"] = "Configurable media".into();
    site["background_image"] = " https://images.example.test/home.jpg ".into();
    site["avatar"] = "/media/owner.png".into();
    site["random_cover"] = "https://images.example.test/one.jpg\n/media/two.jpg".into();
    for (index, key) in keys.into_iter().enumerate() {
        site["media"][key] = if key == "toolbox_video" {
            " /media/clip.mp4 ".into()
        } else {
            json!(format!(
                " https://images.example.test/{index}.jpg?q=a'b&size=(2) "
            ))
        };
    }
    let response = admin(&app, &session, "PUT", "/api/v1/content/site", site).await;
    assert_eq!(response.status(), StatusCode::OK);
    let saved = body_json(response).await;
    assert_eq!(
        saved["background_image"],
        "https://images.example.test/home.jpg"
    );
    assert_eq!(saved["media"]["toolbox_video"], "/media/clip.mp4");
    assert_eq!(
        saved["media"]["logo"],
        "https://images.example.test/0.jpg?q=a'b&size=(2)"
    );
    assert_eq!(public(&app, "/api/v1/site").await["media"], saved["media"]);
    let persisted: String = sqlx::query_scalar("SELECT media FROM web_info")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&persisted).unwrap(),
        saved["media"]
    );
    let response = admin(&app, &session, "GET", "/api/v1/content/site", json!({})).await;
    assert_eq!(body_json(response).await["media"], saved["media"]);

    let mut cleared = saved;
    cleared["background_image"] = " ".into();
    cleared["avatar"] = "".into();
    cleared["random_cover"] = Value::Null;
    for key in keys {
        cleared["media"][key] = "  ".into();
    }
    let response = admin(&app, &session, "PUT", "/api/v1/content/site", cleared).await;
    assert_eq!(response.status(), StatusCode::OK);
    let cleared = body_json(response).await;
    assert!(cleared["background_image"].is_null());
    assert!(cleared["avatar"].is_null());
    assert!(cleared["random_cover"].is_null());
    assert_eq!(cleared["media"], empty["media"]);
    assert_eq!(public(&app, "/api/v1/site").await["media"], empty["media"]);
}

#[tokio::test]
async fn site_media_rejects_invalid_shapes_and_urls_before_mutating_saved_state() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    let response = admin(&app, &session, "GET", "/api/v1/content/site", json!({})).await;
    let mut original = body_json(response).await;
    original["web_name"] = "Kept".into();
    let response = admin(
        &app,
        &session,
        "PUT",
        "/api/v1/content/site",
        original.clone(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let mut bad = original.clone();
    bad["author_name"] = "Must not commit".into();
    bad["web_name"] = "Must not commit".into();
    let mut invalid = Vec::new();
    let mut missing = bad.clone();
    missing.as_object_mut().unwrap().remove("media");
    invalid.push(missing);
    for media in [
        Value::Null,
        json!([]),
        json!({"unknown":null}),
        json!({"logo":12}),
    ] {
        let mut input = bad.clone();
        input["media"] = media;
        invalid.push(input);
    }
    for url in [
        "javascript:alert(1)".to_owned(),
        "data:image/png;base64,AAAA".to_owned(),
        "//images.example.test/logo.jpg".to_owned(),
        "/\\images.example.test/logo.jpg".to_owned(),
        "images/logo.jpg".to_owned(),
        "http:relative.jpg".to_owned(),
        "https://".to_owned(),
        "https://images.example.test/\nlogo.jpg".to_owned(),
        "https://images.example.test/\0logo.jpg".to_owned(),
        format!("https://images.example.test/{}", "x".repeat(2048)),
    ] {
        let mut input = bad.clone();
        input["media"]["toolbox_video"] = json!(url);
        invalid.push(input);
    }
    for key in original["media"].as_object().unwrap().keys() {
        let mut input = bad.clone();
        input["media"][key] = "javascript:alert(1)".into();
        invalid.push(input);
    }
    for key in ["avatar", "background_image"] {
        let mut input = bad.clone();
        input[key] = "javascript:alert(1)".into();
        invalid.push(input);
    }
    for covers in [
        "[\"/image.jpg\",\"javascript:alert(1)\"]",
        "[\"/image.jpg\",12]",
        "https://images.example.test/one.jpg\n//other.example.test/two.jpg",
    ] {
        let mut input = bad.clone();
        input["random_cover"] = covers.into();
        invalid.push(input);
    }
    for input in invalid {
        let response = admin(&app, &session, "PUT", "/api/v1/content/site", input.clone()).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{input}");
    }
    let response = admin(&app, &session, "GET", "/api/v1/content/site", json!({})).await;
    assert_eq!(body_json(response).await, original);
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT username FROM user WHERE user_type=0")
            .fetch_one(&pool)
            .await
            .unwrap(),
        "site-author"
    );
}

#[tokio::test]
async fn news_pagination_reaches_history_beyond_public_limit_and_scopes_updates() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,1,1,'First','Body'),(2,1,1,1,'Second','Body')").execute(&pool).await.unwrap();
    for id in 1..=105_i64 {
        sqlx::query("INSERT INTO wei_yan(id,user_id,content,type,source,is_public,create_time) VALUES(?,1,?,'news',1,1,'2026-10-12 00:00:00')")
            .bind(id).bind(format!("Update {id}")).execute(&pool).await.unwrap();
    }
    let response = admin(
        &app,
        &session,
        "GET",
        "/api/v1/content/articles/1/news?page=6&size=20",
        json!({}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let history = body_json(response).await;
    assert_eq!(history["total"], 105);
    assert_eq!(history["page"], 6);
    assert_eq!(history["size"], 20);
    assert_eq!(history["items"].as_array().unwrap().len(), 5);
    assert_eq!(history["items"][4]["id"], 1);
    assert_eq!(
        public(&app, "/api/v1/articles/1/news")
            .await
            .as_array()
            .unwrap()
            .len(),
        100
    );
    let update = json!({"content":" Edited oldest ","create_time":"2026-10-10T12:00:00+02:00"});
    assert_eq!(
        admin(
            &app,
            &session,
            "PUT",
            "/api/v1/content/articles/2/news/1",
            update.clone()
        )
        .await
        .status(),
        StatusCode::NOT_FOUND
    );
    let response = admin(
        &app,
        &session,
        "PUT",
        "/api/v1/content/articles/1/news/1",
        update,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let changed = body_json(response).await;
    assert_eq!(changed["content"], "Edited oldest");
    assert_eq!(changed["create_time"], "2026-10-10 10:00:00");
    let response = admin(
        &app,
        &session,
        "PUT",
        "/api/v1/content/articles/1/news/1",
        json!({"content":"Retain date","create_time":null}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_json(response).await["create_time"],
        changed["create_time"]
    );
    for invalid in [
        json!({"content":"","create_time":null}),
        json!({"content":"ok","create_time":"invalid"}),
        json!({"content":"ok","create_time":null,"source":2}),
    ] {
        assert_eq!(
            admin(
                &app,
                &session,
                "PUT",
                "/api/v1/content/articles/1/news/1",
                invalid
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        admin(
            &app,
            &session,
            "DELETE",
            "/api/v1/content/articles/1/news/1",
            json!({})
        )
        .await
        .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn letter_reads_do_not_create_and_private_public_flow_is_isolated() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    assert!(public(&app, "/api/v1/letter").await.is_null());
    let response = admin(&app, &session, "GET", "/api/v1/content/letter", json!({})).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_json(response).await,
        json!({"recipient":"","sender":"","content":"","is_public":false})
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wei_yan")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    let response = admin(
        &app,
        &session,
        "PUT",
        "/api/v1/content/letter",
        letter(false),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, letter(false));
    assert!(public(&app, "/api/v1/letter").await.is_null());
    assert_eq!(
        admin(
            &app,
            &session,
            "PUT",
            "/api/v1/content/letter",
            letter(true)
        )
        .await
        .status(),
        StatusCode::OK
    );
    let visible = public(&app, "/api/v1/letter").await;
    assert_eq!(visible["recipient"], "Partner");
    assert_eq!(visible["content"], "First line\nSecond line <literal>");
    assert!(visible.get("is_public").is_none());
    assert_eq!(
        public(&app, "/api/v1/notes/page").await["total"],
        0,
        "letter must not become a note"
    );
    let response = admin(&app, &session, "GET", "/api/v1/content/notes", json!({})).await;
    assert_eq!(body_json(response).await["total"], 0);
    assert_eq!(
        admin(
            &app,
            &session,
            "PUT",
            "/api/v1/content/letter",
            letter(false)
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert!(public(&app, "/api/v1/letter").await.is_null());
    sqlx::query("INSERT INTO user(id,username) VALUES(2,'other')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO wei_yan(user_id,type,content,is_public) VALUES(2,'letter','malformed other owner',1)").execute(&pool).await.unwrap();
    assert!(
        public(&app, "/api/v1/letter").await.is_null(),
        "other owner's letter cannot appear"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM wei_yan WHERE type='letter' AND user_id=1"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[tokio::test]
async fn concurrent_letter_puts_keep_single_owned_row() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    let mut second = letter(true);
    second["content"] = "Second complete draft".into();
    let (first, second_response) = tokio::join!(
        admin(
            &app,
            &session,
            "PUT",
            "/api/v1/content/letter",
            letter(false)
        ),
        admin(
            &app,
            &session,
            "PUT",
            "/api/v1/content/letter",
            second.clone()
        ),
    );
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second_response.status(), StatusCode::OK);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM wei_yan WHERE type='letter' AND user_id=1"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    let response = admin(&app, &session, "GET", "/api/v1/content/letter", json!({})).await;
    let saved = body_json(response).await;
    assert!(
        saved == letter(false) || saved == second,
        "one complete payload wins, without mixing fields"
    );
}

#[tokio::test]
async fn letter_invalid_input_and_corrupt_storage_never_publish() {
    let (_directory, app, pool) = setup().await;
    let session = session(&app).await;
    for invalid in [
        json!({"recipient":"","sender":"Owner","content":"Body","is_public":true}),
        json!({"recipient":"Partner","sender":"","content":"Body","is_public":true}),
        json!({"recipient":"Partner","sender":"Owner","content":" ","is_public":true}),
        json!({"recipient":"x".repeat(101),"sender":"Owner","content":"Body","is_public":true}),
        json!({"recipient":"Partner","sender":"Owner","content":"x".repeat(16_385),"is_public":true}),
        json!({"recipient":"Partner","sender":"Owner","content":"Body","is_public":true,"user_id":2}),
        json!({"recipient":"Partner","sender":"Owner","content":[],"is_public":true}),
    ] {
        assert_eq!(
            admin(&app, &session, "PUT", "/api/v1/content/letter", invalid)
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wei_yan")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
    let stored = json!({"recipient":"Partner","sender":"Owner","content":"Body"}).to_string();
    for invalid in [
        "not-json".to_owned(),
        json!({"recipient":"Partner","sender":"Owner","content":"Body","unknown":true}).to_string(),
        "x".repeat(128 * 1024 + 1),
    ] {
        sqlx::query("DELETE FROM wei_yan")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO wei_yan(user_id,type,content,is_public) VALUES(1,'letter',?,1)")
            .bind(invalid)
            .execute(&pool)
            .await
            .unwrap();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/letter")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
    sqlx::query("DELETE FROM wei_yan")
        .execute(&pool)
        .await
        .unwrap();
    for _ in 0..2 {
        sqlx::query("INSERT INTO wei_yan(user_id,type,content,is_public) VALUES(1,'letter',?,1)")
            .bind(&stored)
            .execute(&pool)
            .await
            .unwrap();
    }
    assert_eq!(
        admin(&app, &session, "GET", "/api/v1/content/letter", json!({}))
            .await
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        admin(
            &app,
            &session,
            "PUT",
            "/api/v1/content/letter",
            letter(true)
        )
        .await
        .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wei_yan")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2,
        "malformed singleton state is not silently rewritten"
    );
}

#[tokio::test]
async fn management_routes_require_administrator_and_csrf() {
    let (_directory, app, _pool) = setup().await;
    let session = session(&app).await;
    for (method, path, value) in [
        (
            "PUT",
            "/api/v1/content/notes/1",
            note("Content", None, true),
        ),
        ("PUT", "/api/v1/content/letter", letter(true)),
        ("GET", "/api/v1/content/letter", json!({})),
        (
            "PUT",
            "/api/v1/content/articles/1/news/1",
            json!({"content":"Update","create_time":null}),
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(value.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {path}"
        );
        if method == "PUT" {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .header(header::HOST, "127.0.0.1:18881")
                        .header(header::ORIGIN, "http://127.0.0.1:18881")
                        .header("sec-fetch-site", "same-origin")
                        .header(header::COOKIE, &session.0)
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(value.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::FORBIDDEN,
                "no CSRF: {method} {path}"
            );
        }
    }
}

async fn publish_news_with_key(
    app: &Router,
    session: &(String, String),
    article: i64,
    key: &str,
    value: Value,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/content/articles/{article}/news"))
                .header(header::HOST, "127.0.0.1:18881")
                .header(header::ORIGIN, "http://127.0.0.1:18881")
                .header("sec-fetch-site", "same-origin")
                .header(header::COOKIE, &session.0)
                .header("x-csrf-token", &session.1)
                .header(header::CONTENT_TYPE, "application/json")
                .header("Idempotency-Key", key)
                .body(Body::from(value.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn news_publish_retries_preserve_identity_and_deliberate_repeats() {
    let (directory, app, pool) = setup().await;
    let auth = session(&app).await;
    assert_eq!(
        admin(
            &app,
            &auth,
            "POST",
            "/api/v1/content/categories",
            json!({"name":"Review"})
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        admin(
            &app,
            &auth,
            "POST",
            "/api/v1/content/labels",
            json!({"sort_id":1,"name":"Review"})
        )
        .await
        .status(),
        StatusCode::OK
    );
    let input = json!({"article_title":"Review article","article_content":"Review body","sort_id":1,"label_id":1,"view_status":true,"recommend_status":false,"comment_status":true});
    let first = admin(
        &app,
        &auth,
        "POST",
        "/api/v1/content/articles",
        input.clone(),
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);
    let id = body_json(first).await["id"].as_i64().unwrap();
    let second = admin(&app, &auth, "POST", "/api/v1/content/articles", input).await;
    assert_eq!(second.status(), StatusCode::OK);
    let other_id = body_json(second).await["id"].as_i64().unwrap();
    let payload = json!({"content":"One intended update","create_time":null});
    let committed = publish_news_with_key(&app, &auth, id, "first-publish", payload.clone()).await;
    assert_eq!(committed.status(), StatusCode::OK);
    drop(committed); // Commit succeeded, but its response body never reached the editor.
    let retry = publish_news_with_key(&app, &auth, id, "first-publish", payload.clone()).await;
    assert_eq!(retry.status(), StatusCode::OK);
    let saved = body_json(retry).await;
    assert_eq!(
        admin(
            &app,
            &auth,
            "PUT",
            &format!("/api/v1/content/articles/{id}/news/{}", saved["id"]),
            json!({"content":"Edited after publication","create_time":null})
        )
        .await
        .status(),
        StatusCode::OK
    );
    let replay_after_edit =
        publish_news_with_key(&app, &auth, id, "first-publish", payload.clone()).await;
    assert_eq!(replay_after_edit.status(), StatusCode::OK);
    let replay_after_edit = body_json(replay_after_edit).await;
    assert_eq!(replay_after_edit["id"], saved["id"]);
    assert_eq!(
        replay_after_edit["content"], "Edited after publication",
        "Retry must not overwrite a later edit"
    );
    let public_path = format!("/api/v1/articles/{id}/news");
    assert_eq!(
        public(&app, &public_path).await.as_array().unwrap().len(),
        1
    );
    assert_eq!(
        publish_news_with_key(
            &app,
            &auth,
            id,
            "first-publish",
            json!({"content":"Different","create_time":null})
        )
        .await
        .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        publish_news_with_key(
            &app,
            &auth,
            id,
            "first-publish",
            json!({"content":"One intended update","create_time":"2026-10-01T00:00:00Z"})
        )
        .await
        .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        publish_news_with_key(&app, &auth, other_id, "first-publish", payload.clone())
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let (one, two) = tokio::join!(
        publish_news_with_key(&app, &auth, id, "second-publish", payload.clone()),
        publish_news_with_key(&app, &auth, id, "second-publish", payload.clone())
    );
    assert_eq!(one.status(), StatusCode::OK);
    assert_eq!(two.status(), StatusCode::OK);
    assert_eq!(body_json(one).await["id"], body_json(two).await["id"]);
    assert_eq!(
        public(&app, &public_path).await.as_array().unwrap().len(),
        2,
        "A fresh key permits a deliberate identical update"
    );
    assert_eq!(
        admin(
            &app,
            &auth,
            "DELETE",
            &format!("/api/v1/content/articles/{id}/news/{}", saved["id"]),
            Value::Null
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        publish_news_with_key(&app, &auth, id, "first-publish", payload.clone())
            .await
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        public(&app, &public_path).await.as_array().unwrap().len(),
        1
    );
    assert_eq!(
        publish_news_with_key(&app, &auth, id, "", payload)
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM article_news_submission")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2
    );
    let before: Vec<(String, Vec<u8>, Option<i64>)> = sqlx::query_as(
        "SELECT request_id,request_hash,news_id FROM article_news_submission ORDER BY request_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    close_fixture_pool(&pool).await;
    let reopened = open_database(&directory.path().join("site.sqlite"))
        .await
        .unwrap();
    let after: Vec<(String, Vec<u8>, Option<i64>)> = sqlx::query_as(
        "SELECT request_id,request_hash,news_id FROM article_news_submission ORDER BY request_id",
    )
    .fetch_all(&reopened)
    .await
    .unwrap();
    assert_eq!(
        before, after,
        "Live and deleted receipts survive database reopen"
    );
}

async fn create_note_with_key(
    app: &Router,
    session: &(String, String),
    key: &str,
    value: Value,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/content/notes")
                .header(header::HOST, "127.0.0.1:18881")
                .header(header::ORIGIN, "http://127.0.0.1:18881")
                .header("sec-fetch-site", "same-origin")
                .header(header::COOKIE, &session.0)
                .header("x-csrf-token", &session.1)
                .header(header::CONTENT_TYPE, "application/json")
                .header("Idempotency-Key", key)
                .body(Body::from(value.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn note_creation_receipts_recover_lost_responses_without_overwriting_later_edits() {
    let (_directory, app, pool) = setup_with_pool_limit(1).await;
    let auth = session(&app).await;
    let image = uploaded_image(&app, &auth).await;
    let original = note("  One intended note  ", Some(&image), true);
    let first = create_note_with_key(&app, &auth, "original-note", original.clone()).await;
    assert_eq!(first.status(), StatusCode::OK);
    drop(first); // The row committed, but the response never reached the editor.
    let recovered = create_note_with_key(&app, &auth, "original-note", original.clone()).await;
    assert_eq!(recovered.status(), StatusCode::OK);
    let recovered = body_json(recovered).await;
    let id = recovered["id"].as_i64().unwrap();
    assert_eq!(recovered["content"], "One intended note");
    let normalized = note("One intended note", Some(&image), true);
    let replay = create_note_with_key(&app, &auth, "original-note", normalized).await;
    assert_eq!(body_json(replay).await["id"], id);
    assert_eq!(
        admin(
            &app,
            &auth,
            "PUT",
            &format!("/api/v1/content/notes/{id}"),
            note("Edited and hidden", None, false),
        )
        .await
        .status(),
        StatusCode::OK
    );
    let resource: i64 = sqlx::query_scalar("SELECT id FROM resource WHERE path=?")
        .bind(&image)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        admin(
            &app,
            &auth,
            "DELETE",
            &format!("/api/v1/content/resources/{resource}"),
            json!({"path":image}),
        )
        .await
        .status(),
        StatusCode::OK
    );
    let replay = create_note_with_key(&app, &auth, "original-note", original.clone()).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay = body_json(replay).await;
    assert_eq!(replay["id"], id);
    assert_eq!(replay["content"], "Edited and hidden");
    assert_eq!(replay["is_public"], 0);
    assert!(replay["image_path"].is_null());
    for changed in [
        note("Different draft", Some(&image), true),
        note("One intended note", None, true),
        note("One intended note", Some(&image), false),
    ] {
        assert_eq!(
            create_note_with_key(&app, &auth, "original-note", changed)
                .await
                .status(),
            StatusCode::CONFLICT
        );
    }
    let parallel = note("Concurrent note", None, true);
    let (first, second) = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::join!(
            create_note_with_key(&app, &auth, "parallel-note", parallel.clone()),
            create_note_with_key(&app, &auth, "parallel-note", parallel.clone()),
        )
    })
    .await
    .expect("note receipts must not nest pool checkout");
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(body_json(first).await["id"], body_json(second).await["id"]);
    for _ in 0..2 {
        assert_eq!(
            admin(
                &app,
                &auth,
                "POST",
                "/api/v1/content/notes",
                parallel.clone()
            )
            .await
            .status(),
            StatusCode::OK,
            "unkeyed posts remain deliberate independent creations"
        );
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wei_yan WHERE type='friend'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        4
    );
    assert_eq!(
        admin(
            &app,
            &auth,
            "DELETE",
            &format!("/api/v1/content/notes/{id}"),
            Value::Null,
        )
        .await
        .status(),
        StatusCode::OK
    );
    let deleted_receipt: Option<i64> =
        sqlx::query_scalar("SELECT note_id FROM note_submission WHERE request_id='original-note'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(deleted_receipt.is_none());
    assert_eq!(
        create_note_with_key(&app, &auth, "original-note", original)
            .await
            .status(),
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn note_receipts_validate_keys_and_commit_atomically_then_survive_database_reopen() {
    let (directory, app, pool) = setup().await;
    let auth = session(&app).await;
    let draft = note("Durable note", None, true);
    for key in ["".to_owned(), "has space".to_owned(), "x".repeat(129)] {
        assert_eq!(
            create_note_with_key(&app, &auth, &key, draft.clone())
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    let invalid_image = format!("/media/{}.png", "0".repeat(64));
    assert_eq!(
        create_note_with_key(
            &app,
            &auth,
            "durable-note",
            note("Durable note", Some(&invalid_image), true),
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
    for table in ["note_submission", "wei_yan"] {
        assert_eq!(
            sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!(
                "SELECT COUNT(*) FROM {table}"
            )))
            .fetch_one(&pool)
            .await
            .unwrap(),
            0,
            "failed creation must not leave a row or consume its receipt key"
        );
    }
    let response = create_note_with_key(&app, &auth, "durable-note", draft.clone()).await;
    assert_eq!(response.status(), StatusCode::OK);
    let saved = body_json(response).await;
    close_fixture_pool(&pool).await;
    let pool = open_database(&directory.path().join("site.sqlite"))
        .await
        .unwrap();
    let app = router(
        AppState {
            scope: blog_application::server_runtime::WorkScope::new(),
            pool: pool.clone(),
            admin: Arc::new(AdministratorService::new(SqliteAdministratorStore::new(
                pool.clone(),
            ))),
            origin: AdministratorOriginMode::LoopbackDevelopmentHttp,
            media: directory.path().to_path_buf(),
            media_operations: Arc::new(tokio::sync::Mutex::new(())),
            public_mutations: Arc::new(crate::app::PublicMutationCapacity::default()),
            password_slots: Arc::new(tokio::sync::Semaphore::new(2)),
        },
        None,
    )
    .unwrap();
    let replay = create_note_with_key(&app, &auth, "durable-note", draft).await;
    assert_eq!(replay.status(), StatusCode::OK);
    assert_eq!(body_json(replay).await, saved);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wei_yan WHERE type='friend'")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
}

#[tokio::test]
async fn paginated_content_keeps_counts_items_and_source_reads_in_one_snapshot() {
    use std::sync::atomic::{AtomicBool, Ordering};
    for (path, expected_total) in [
        ("/api/v1/articles", 1),
        ("/api/v1/content/articles", 1),
        ("/api/v1/content/articles/1/news", 1),
        ("/api/v1/notes/page", 1),
        ("/api/v1/content/notes", 1),
        ("/api/v1/links/page?kind=lovePhoto", 1),
        ("/api/v1/content/links", 1),
        ("/api/v1/content/tree-hole", 1),
        ("/api/v1/content/resources", 1),
        ("/api/v1/message-comments", 1),
        ("/api/v1/message-comments/3/replies", 1),
        ("/api/v1/love-comments", 1),
        ("/api/v1/love-comments/5/replies", 1),
        ("/api/v1/content/comments", 6),
    ] {
        let (directory, app, fixture_pool) = setup().await;
        let auth = session(&app).await;
        sqlx::raw_sql(
            "INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,1,1,'Before edit','Body');
             INSERT INTO wei_yan(id,user_id,type,content,is_public,source) VALUES(1,1,'friend','Before edit',1,NULL),(2,1,'news','Before edit',1,1);
             INSERT INTO resource_path(id,title,type,status) VALUES(1,'Before edit','lovePhoto',1);
             INSERT INTO tree_hole(id,message) VALUES(1,'Before edit');
             INSERT INTO resource(id,type,path,status) VALUES(1,'image','/media/fixture.png',1);
             INSERT INTO comment(id,source,type,parent_comment_id,floor_comment_id,comment_content) VALUES
                (1,1,'article',0,NULL,'Article root'),(2,1,'article',1,1,'Article reply'),
                (3,0,'message',0,NULL,'Message root'),(4,0,'message',3,3,'Message reply'),
                (5,1,'love',0,NULL,'Love root'),(6,1,'love',5,5,'Love reply');",
        )
        .execute(&fixture_pool)
        .await
        .unwrap();
        let changed = Arc::new(AtomicBool::new(false));
        let observed = changed.clone();
        // Authentication uses the fixture pool. On the content pool, release of
        // the first source/count read commits a concurrent edit deterministically.
        // A transaction retains that connection until its whole page is assembled.
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .after_release(move |connection, _| {
                let observed = observed.clone();
                Box::pin(async move {
                    if !observed.swap(true, Ordering::SeqCst) {
                        sqlx::raw_sql("BEGIN IMMEDIATE; UPDATE article SET deleted=1; DELETE FROM comment; DELETE FROM wei_yan; DELETE FROM resource_path; DELETE FROM tree_hole; DELETE FROM resource; COMMIT;")
                            .execute(connection).await?;
                    }
                    Ok(true)
                })
            })
            .connect_lazy_with((*fixture_pool.connect_options()).clone());
        let app = router(
            AppState {
                scope: blog_application::server_runtime::WorkScope::new(),
                pool: pool.clone(),
                admin: Arc::new(AdministratorService::new(SqliteAdministratorStore::new(
                    fixture_pool.clone(),
                ))),
                origin: AdministratorOriginMode::LoopbackDevelopmentHttp,
                media: directory.path().to_path_buf(),
                media_operations: Arc::new(tokio::sync::Mutex::new(())),
                public_mutations: Arc::new(crate::app::PublicMutationCapacity::default()),
                password_slots: Arc::new(tokio::sync::Semaphore::new(2)),
            },
            None,
        )
        .unwrap();
        let response = admin(&app, &auth, "GET", path, Value::Null).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let body = body_json(response).await;
        assert_eq!(body["total"], expected_total, "{path}");
        assert_eq!(
            body["items"].as_array().unwrap().len(),
            expected_total as usize,
            "{path}"
        );
        // Pool return hooks run asynchronously. A subsequent checkout waits for
        // the edit to commit; closing immediately can skip that return hook.
        let mut connection = pool.acquire().await.unwrap();
        assert!(
            changed.load(Ordering::SeqCst),
            "concurrent edit did not run: {path}"
        );
        let remaining: i64 = sqlx::query_scalar("SELECT (SELECT COUNT(*) FROM article WHERE deleted=0)+(SELECT COUNT(*) FROM comment)+(SELECT COUNT(*) FROM wei_yan)+(SELECT COUNT(*) FROM resource_path)+(SELECT COUNT(*) FROM tree_hole)+(SELECT COUNT(*) FROM resource)")
            .fetch_one(&mut *connection).await.unwrap();
        assert_eq!(remaining, 0, "concurrent edit did not commit: {path}");
        connection.close().await.unwrap();
        pool.close().await;
    }
}
