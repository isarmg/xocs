use super::*;
use serde_json::{Value, json};

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
