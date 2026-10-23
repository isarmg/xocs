use crate::{
    AppState,
    app::router,
    database::{initialize, initialize_new_database, open_database, validate_existing_database},
};
use axum::{Router, http::StatusCode, response::Response};
use axum::{
    body::{Body, to_bytes},
    http::{Request, header},
};
use blog_application::admin_auth::AdministratorOriginMode;
use blog_application::admin_core::AdministratorService;
use blog_application::admin_sqlite::SqliteAdministratorStore;
use sqlx::SqlitePool;
use std::{os::unix::fs::PermissionsExt, sync::Arc};
use tower::ServiceExt;

async fn setup() -> (tempfile::TempDir, Router, SqlitePool) {
    setup_with_pool_limit(8).await
}

async fn setup_with_pool_limit(limit: u32) -> (tempfile::TempDir, Router, SqlitePool) {
    setup_with_password_slots(limit, Arc::new(tokio::sync::Semaphore::new(2))).await
}

async fn setup_with_password_slots(
    limit: u32,
    password_slots: Arc<tokio::sync::Semaphore>,
) -> (tempfile::TempDir, Router, SqlitePool) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let db = dir.path().join("site.sqlite");
    let pool = crate::database::open_database_with_limit(&db, true, limit)
        .await
        .unwrap();
    initialize(&pool, "admin", "TemporaryPassphrase123")
        .await
        .unwrap();
    std::fs::write(dir.path().join("index.html"), "<h1>BLOG</h1>").unwrap();
    let admin = Arc::new(AdministratorService::new(SqliteAdministratorStore::new(
        pool.clone(),
    )));
    let state = AppState {
        scope: blog_application::server_runtime::WorkScope::new(),
        pool: pool.clone(),
        admin,
        origin: AdministratorOriginMode::LoopbackDevelopmentHttp,
        media: dir.path().to_path_buf(),
        media_operations: Arc::new(tokio::sync::Mutex::new(())),
        public_mutations: Arc::new(crate::app::PublicMutationCapacity::default()),
        password_slots,
    };
    let app = router(state, Some(dir.path().to_path_buf())).unwrap();
    (dir, app, pool)
}

async fn body_json(response: Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 2_000_000).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn close_fixture_pool(pool: &SqlitePool) {
    // Own each connection before closing the pool. Pool-return tasks may
    // otherwise release their permits before SQLite finishes its last
    // checkpoint; the read-only assertion requires a stopped writer.
    let mut connections = Vec::new();
    while connections.len() < pool.size() as usize {
        connections.push(pool.acquire().await.unwrap());
    }
    let closed = pool.close();
    for connection in connections {
        connection.close().await.unwrap();
    }
    closed.await;
}

#[tokio::test]
async fn native_value_limits_preserve_valid_articles_and_survive_connection_replacement() {
    let (_directory, app, pool) = setup().await;
    let content = "x".repeat(2_000_000);
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(77,1,0,0,'large current article',?)")
        .bind(&content).execute(&pool).await.unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/articles/77")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 3_000_000).await.unwrap();
    let article: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(article["article_content"], content);

    for _ in 0..2 {
        let error = sqlx::query_scalar::<_, Vec<u8>>("SELECT zeroblob(5242880)")
            .fetch_one(&pool)
            .await
            .unwrap_err();
        assert_eq!(
            error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("18")
        );
        let bytes: i64 = sqlx::query_scalar(
            "SELECT length(CAST(article_content AS BLOB)) FROM article WHERE id=77",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(bytes, 2_000_000);
        pool.acquire().await.unwrap().close().await.unwrap();
    }
    pool.close().await;
}

#[tokio::test]
async fn schema_drift_is_rejected_without_changing_the_source_generation() {
    let (directory, _app, pool) = setup().await;
    sqlx::query("CREATE TABLE unexpected_runtime_table(id INTEGER)")
        .execute(&pool)
        .await
        .unwrap();
    close_fixture_pool(&pool).await;
    let database = directory.path().join("site.sqlite");
    let before = std::fs::read(&database).unwrap();
    let sidecars = ["-wal", "-shm", "-journal"]
        .map(|suffix| std::fs::read(format!("{}{suffix}", database.display())).ok());
    assert!(
        validate_existing_database(&database, database.parent().unwrap())
            .await
            .is_err()
    );
    assert_eq!(std::fs::read(&database).unwrap(), before);
    assert_eq!(
        sidecars,
        ["-wal", "-shm", "-journal"].map(|suffix| std::fs::read(format!(
            "{}{suffix}",
            database.display()
        ))
        .ok())
    );
}

#[tokio::test]
async fn initialization_publishes_only_complete_data_and_never_overwrites() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let database = directory.path().join("site.sqlite");
    initialize_new_database(&database, "admin", "TemporaryPassphrase123")
        .await
        .unwrap();
    validate_existing_database(&database, database.parent().unwrap())
        .await
        .unwrap();
    let before = std::fs::read(&database).unwrap();
    assert!(
        initialize_new_database(&database, "other-admin", "AnotherPassphrase123")
            .await
            .is_err()
    );
    assert_eq!(std::fs::read(&database).unwrap(), before);
}

#[tokio::test]
async fn anonymous_contract_rejects_forged_identity_without_echoing_sensitive_values() {
    let (_directory, app, _pool) = setup().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tree-hole/guest")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"message":"SensitivePassphrase123","user_id":42}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert_eq!(body["code"], "contract_violation");
    assert_eq!(body["details"]["reason"], "UNKNOWN_FIELD");
    assert!(body["request_id"].as_str().is_some_and(|id| !id.is_empty()));
    assert!(!body.to_string().contains("SensitivePassphrase123"));
}

#[tokio::test]
async fn public_rejections_keep_one_request_identity_and_safe_error_contract() {
    let (_directory, app, _pool) = setup().await;
    for (uri, method, content_type, payload, expected) in [
        (
            "/api/v1/tree-hole/guest",
            "POST",
            "application/json",
            "{\"password\":\"PrivatePassphrase\"}",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/v1/tree-hole/guest",
            "POST",
            "application/json",
            "{invalid-secret-json",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/v1/tree-hole/guest",
            "POST",
            "text/plain",
            "private-body",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "/api/v1/articles/not-an-integer",
            "GET",
            "application/json",
            "",
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/v1/unknown-route",
            "GET",
            "application/json",
            "",
            StatusCode::NOT_FOUND,
        ),
        (
            "/api/v1/articles",
            "POST",
            "application/json",
            "",
            StatusCode::METHOD_NOT_ALLOWED,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("content-type", content_type)
                    .header("x-request-id", "product-check-123")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected, "{uri}");
        assert_eq!(response.headers()["x-request-id"], "product-check-123");
        assert_eq!(response.headers()["cache-control"], "no-store");
        let value = body_json(response).await;
        assert_eq!(value["request_id"], "product-check-123");
        assert!(value["code"].is_string());
        assert!(value["retryable"].is_boolean());
        assert!(!value.to_string().contains("private-body"));
        assert!(!value.to_string().contains("PrivatePassphrase"));
        assert!(!value.to_string().contains("invalid-secret-json"));
    }
}
#[tokio::test]
async fn public_visibility_and_password_are_enforced() {
    let (_dir, app, pool) = setup().await;
    let hash = blog_application::admin_auth::hash_password("ProtectedPassphrase123").unwrap();
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,view_status,password) VALUES(2,1,1,1,'Protected','private body',0,?)")
        .bind(hash).execute(&pool).await.unwrap();
    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/articles")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let list = body_json(list).await;
    assert_eq!(list["total"], 1);
    assert!(list.to_string().contains("Protected"));
    assert!(!list.to_string().contains("private body"));
    assert!(list["items"][0]["excerpt"].is_null());
    let stats = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/site/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(stats.status(), StatusCode::OK);
    assert_eq!(body_json(stats).await["article_count"], 1);
    let public = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/articles/2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public.status(), StatusCode::NOT_FOUND);
    let access = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/articles/2/access")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(access.status(), StatusCode::OK);
    assert_eq!(body_json(access).await["password_required"], 1);
    let bad = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/articles/2/unlock")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"password":"wrong"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::FORBIDDEN);
    let good = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/articles/2/unlock")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"password":"ProtectedPassphrase123"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(good.status(), StatusCode::OK);
    let body = body_json(good).await.to_string();
    assert!(body.contains("private body"));
    assert!(!body.contains("ProtectedPassphrase123"));
    for _attempt in 1..=11 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/articles/2/unlock")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"password":"wrong"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn article_unlock_rechecks_password_after_waiting_for_verification() {
    let slots = Arc::new(tokio::sync::Semaphore::new(0));
    let (_dir, app, pool) = setup_with_password_slots(8, slots.clone()).await;
    let old_hash = blog_application::admin_auth::hash_password("ProtectedPassphrase123").unwrap();
    let new_hash = blog_application::admin_auth::hash_password("ReplacementPassphrase456").unwrap();
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,view_status,password) VALUES(2,1,1,1,'Protected','old body',0,?)")
        .bind(old_hash).execute(&pool).await.unwrap();
    let request = app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/api/v1/articles/2/unlock")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"password":"ProtectedPassphrase123"}"#))
            .unwrap(),
    );
    tokio::pin!(request);
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(100), &mut request)
            .await
            .is_err()
    );
    // A queued verifier reserves this permit. The request is not spawned, so it
    // cannot resume until explicitly polled again below, after the admin write.
    slots.add_permits(1);
    assert!(
        slots.try_acquire().is_err(),
        "request must already be queued for verification"
    );
    sqlx::query("UPDATE article SET password=?,article_content='new restricted body' WHERE id=2")
        .bind(new_hash)
        .execute(&pool)
        .await
        .unwrap();
    let response = request.await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(
        !body_json(response)
            .await
            .to_string()
            .contains("new restricted body")
    );
    let views: i64 = sqlx::query_scalar("SELECT view_count FROM article WHERE id=2")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(views, 0);
}

#[tokio::test]
async fn administrator_route_rejects_anonymous_mutation() {
    let (_dir, app, _pool) = setup().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/content/articles")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

async fn anonymous_post(app: &Router, path: &str, mut body: serde_json::Value) -> Response {
    if path.ends_with("comments") && body.get("request_id").is_none() {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        body["request_id"] = format!("00000000-0000-4000-8000-{id:012x}").into();
    }
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .header("cookie", "blog_member=obsolete-cookie")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn removed_account_application_and_chat_routes_are_not_available() {
    let (_directory, app, pool) = setup().await;
    for (method, path) in [
        ("POST", "/api/v1/members/register"),
        ("POST", "/api/v1/members/login"),
        ("POST", "/api/v1/members/logout"),
        ("GET", "/api/v1/members/session"),
        ("GET", "/api/v1/resources"),
        ("GET", "/api/v1/links"),
        ("GET", "/api/v1/friends"),
        ("GET", "/api/v1/notes"),
        ("GET", "/api/v1/members/profile"),
        ("POST", "/api/v1/members/password"),
        ("POST", "/api/v1/members/upload"),
        ("GET", "/api/v1/members/notes"),
        ("POST", "/api/v1/family/submissions"),
        ("POST", "/api/v1/links/friend-submissions"),
        ("GET", "/api/v1/im/friends"),
        ("GET", "/api/v1/im/groups"),
        ("GET", "/socket"),
        ("GET", "/im"),
        ("GET", "/user"),
        ("GET", "/friend"),
        ("GET", "/music"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 1);
}

#[tokio::test]
async fn public_journal_pagination_keeps_author_entries_and_hides_private_entries() {
    let (_directory, app, pool) = setup().await;
    sqlx::query("INSERT INTO user(id,username) VALUES(2,'historical-author')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO wei_yan(id,user_id,content,type,is_public) VALUES(1,1,'First public entry','friend',1),(2,1,'Second public entry','friend',1),(3,1,'Private entry','friend',0),(4,2,'Another author','friend',1)")
        .execute(&pool).await.unwrap();
    for (page, content) in [(1, "Second public entry"), (2, "First public entry")] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/notes/page?page={page}&size=1"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert_eq!(body["total"], 2);
        assert_eq!(body["items"].as_array().unwrap().len(), 1);
        assert_eq!(body["items"][0]["content"], content);
        assert!(!body.to_string().contains("Private entry"));
        assert!(!body.to_string().contains("Another author"));
    }
}

#[tokio::test]
async fn comments_and_replies_are_anonymous_and_stay_in_their_source() {
    let (_directory, app, pool) = setup().await;
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,0,0,'Public','Body')")
        .execute(&pool).await.unwrap();
    let root = anonymous_post(
        &app,
        "/api/v1/comments",
        serde_json::json!({"article_id":1,"content":"  Hello anonymously  "}),
    )
    .await;
    assert_eq!(root.status(), StatusCode::OK);
    assert!(!root.headers().contains_key(header::SET_COOKIE));
    let root = body_json(root).await;
    assert!(root["user_id"].is_null());
    assert!(root["username"].is_null());
    assert_eq!(root["comment_content"], "Hello anonymously");
    let reply = anonymous_post(&app, "/api/v1/comments", serde_json::json!({"article_id":1,"content":"Anonymous reply","parent_comment_id":root["id"]})).await;
    assert_eq!(reply.status(), StatusCode::OK);
    let reply = body_json(reply).await;
    assert!(reply["user_id"].is_null());
    assert_eq!(reply["floor_comment_id"], root["id"]);
    let nested = anonymous_post(&app, "/api/v1/comments", serde_json::json!({"article_id":1,"content":"Nested reply","parent_comment_id":reply["id"]})).await;
    assert_eq!(nested.status(), StatusCode::OK);
    assert_eq!(body_json(nested).await["floor_comment_id"], root["id"]);
    for path in ["/api/v1/message-comments", "/api/v1/love-comments"] {
        let response =
            anonymous_post(&app, path, serde_json::json!({"content":"Anonymous wish"})).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_json(response).await["user_id"].is_null());
        let cross_source = anonymous_post(
            &app,
            path,
            serde_json::json!({"content":"Wrong source","parent_comment_id":root["id"]}),
        )
        .await;
        assert_eq!(cross_source.status(), StatusCode::BAD_REQUEST);
    }
    let message = anonymous_post(
        &app,
        "/api/v1/tree-hole/guest",
        serde_json::json!({"message":"Anonymous message"}),
    )
    .await;
    assert_eq!(message.status(), StatusCode::OK);
    assert!(body_json(message).await["user_id"].is_null());
}

#[tokio::test]
async fn anonymous_guest_writes_do_not_create_address_frequency_buckets() {
    let (_dir, app, pool) = setup().await;
    for index in 0..12 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tree-hole/guest")
                    .header("content-type", "application/json")
                    .header("x-forwarded-for", "192.0.2.1")
                    .body(Body::from(format!(r#"{{"message":"Message {index}"}}"#)))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(body_json(response).await["user_id"].is_null());
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tree_hole")
            .fetch_one(&pool)
            .await
            .unwrap(),
        12
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_schema WHERE name='member_login_failures'"
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
}

#[tokio::test]
async fn arbitrary_forwarding_headers_do_not_change_guest_business_behavior() {
    let (_dir, app, pool) = setup().await;
    for value in ["unknown", "192.0.2.1, invalid", "", "192.0.2.1"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tree-hole/guest")
                    .header("content-type", "application/json")
                    .header("x-forwarded-for", value)
                    .body(Body::from(r#"{"message":"Header independent message"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tree_hole")
            .fetch_one(&pool)
            .await
            .unwrap(),
        4
    );
}

#[tokio::test]
async fn anonymous_comments_enforce_content_and_article_visibility() {
    let (_directory, app, pool) = setup().await;
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,view_status,comment_status,password) VALUES(1,1,0,0,'Open','Body',1,1,NULL),(2,1,0,0,'Closed','Body',1,0,NULL),(3,1,0,0,'Hidden','Body',0,1,NULL),(4,1,0,0,'Protected','Body',1,1,'hash')")
        .execute(&pool).await.unwrap();
    for id in [2, 3, 4, 999] {
        assert_eq!(
            anonymous_post(
                &app,
                "/api/v1/comments",
                serde_json::json!({"article_id":id,"content":"Unavailable"})
            )
            .await
            .status(),
            StatusCode::FORBIDDEN
        );
    }
    for content in [
        "".to_string(),
        " ".to_string(),
        "<script>alert(1)</script>".to_string(),
        "x".repeat(1025),
    ] {
        assert_eq!(
            anonymous_post(
                &app,
                "/api/v1/message-comments",
                serde_json::json!({"content":content})
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }
    let forged = anonymous_post(
        &app,
        "/api/v1/comments",
        serde_json::json!({"article_id":1,"content":"Forged author","user_id":1}),
    )
    .await;
    assert_eq!(forged.status(), StatusCode::BAD_REQUEST);
    for attempt in 1..=11 {
        let response = anonymous_post(
            &app,
            "/api/v1/message-comments",
            serde_json::json!({"content":format!("Message {attempt}")}),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comment")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 11);
    let restricted_delete = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/content/comments/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(restricted_delete.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn search_ranks_titles_and_hides_protected_body_matches() {
    let (_dir, app, pool) = setup().await;
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,1,1,'春日记','普通正文'),(2,1,1,1,'另一篇','正文提到春日记')")
        .execute(&pool).await.unwrap();
    let hash = blog_application::admin_auth::hash_password("ProtectedPassphrase123").unwrap();
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,view_status,password) VALUES(3,1,1,1,'加密文章','春日记藏在正文',0,?)")
        .bind(hash).execute(&pool).await.unwrap();
    let search = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/articles?search=%E6%98%A5%E6%97%A5%E8%AE%B0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(search.status(), StatusCode::OK);
    let result = body_json(search).await;
    assert_eq!(result["total"], 2);
    assert_eq!(result["items"][0]["id"], 1);
    assert_eq!(result["items"][1]["id"], 2);
    assert!(
        result["items"][0]["excerpt"]
            .as_str()
            .unwrap()
            .contains("普通正文")
    );
    assert!(
        result["items"][1]["search_snippet"]
            .as_str()
            .unwrap()
            .contains("春日记")
    );
    let short = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/articles?search=%E6%98%A5%E6%97%A5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(body_json(short).await["total"], 2);
    sqlx::query("UPDATE article SET article_content='正文已修改' WHERE id=2")
        .execute(&pool)
        .await
        .unwrap();
    let after = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/articles?search=%E6%98%A5%E6%97%A5%E8%AE%B0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(body_json(after).await["total"], 1);
}

async fn submit_message_comment(
    app: &Router,
    forwarded: Option<&str>,
    request_id: &str,
    content: &str,
) -> Response {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/v1/message-comments")
        .header("content-type", "application/json");
    if let Some(ip) = forwarded {
        request = request.header("x-forwarded-for", ip);
    }
    let request = request
        .body(Body::from(
            serde_json::json!({
                "request_id": request_id, "content": content,
            })
            .to_string(),
        ))
        .unwrap();
    app.clone().oneshot(request).await.unwrap()
}

fn comment_request_id(id: u64) -> String {
    format!("12345678-1234-4123-8123-{id:012x}")
}

#[tokio::test]
async fn comments_ignore_network_identity_and_keep_submission_idempotency() {
    let (_dir, app, pool) = setup().await;
    for (index, forwarded) in [Some(""), Some("unknown"), Some("192.0.2.1, invalid"), None]
        .into_iter()
        .enumerate()
    {
        let id = comment_request_id(index as u64 + 100);
        for _ in 0..2 {
            let response = submit_message_comment(&app, forwarded, &id, "Same submission").await;
            assert_eq!(response.status(), StatusCode::OK);
        }
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comment")
            .fetch_one(&pool)
            .await
            .unwrap(),
        4
    );
}

#[tokio::test]
async fn direct_comments_ignore_forwarding_headers_by_default() {
    let (_dir, app, _pool) = setup().await;
    for id in 1..=11 {
        assert_eq!(
            submit_message_comment(
                &app,
                Some(&format!("192.0.2.{id}")),
                &comment_request_id(id),
                "Direct visitor"
            )
            .await
            .status(),
            StatusCode::OK
        );
    }
}

#[tokio::test]
async fn comment_submission_retries_survive_concurrency_restart_and_deletion() {
    let (dir, app, pool) = setup().await;
    let id = comment_request_id(1);
    let (first, second) = tokio::join!(
        submit_message_comment(&app, None, &id, "One submission"),
        submit_message_comment(&app, None, &id, "One submission"),
    );
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let first = body_json(first).await;
    assert_eq!(body_json(second).await["id"], first["id"]);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comment")
            .fetch_one(&pool)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        submit_message_comment(&app, None, &id, "Changed payload")
            .await
            .status(),
        StatusCode::CONFLICT
    );
    // Dropping a committed response models a connection lost after commit.
    drop(submit_message_comment(&app, None, &comment_request_id(2), "Lost response").await);
    drop(app);
    close_fixture_pool(&pool).await;
    let pool = open_database(&dir.path().join("site.sqlite"), false)
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
            media: dir.path().to_path_buf(),
            media_operations: Arc::new(tokio::sync::Mutex::new(())),
            public_mutations: Arc::new(crate::app::PublicMutationCapacity::default()),
            password_slots: Arc::new(tokio::sync::Semaphore::new(2)),
        },
        Some(dir.path().to_path_buf()),
    )
    .unwrap();
    let recovered =
        submit_message_comment(&app, None, &comment_request_id(2), "Lost response").await;
    assert_eq!(recovered.status(), StatusCode::OK);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comment")
            .fetch_one(&pool)
            .await
            .unwrap(),
        2
    );
    for next in 3..=10 {
        assert_eq!(
            submit_message_comment(&app, None, &comment_request_id(next), "Fresh operation")
                .await
                .status(),
            StatusCode::OK
        );
    }
    assert_eq!(
        submit_message_comment(&app, None, &comment_request_id(11), "Another submission")
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        submit_message_comment(&app, None, &id, "One submission")
            .await
            .status(),
        StatusCode::OK
    );
    sqlx::query("DELETE FROM comment WHERE id=?")
        .bind(first["id"].as_i64().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        submit_message_comment(&app, None, &id, "One submission")
            .await
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comment")
            .fetch_one(&pool)
            .await
            .unwrap(),
        10
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM comment_submission")
            .fetch_one(&pool)
            .await
            .unwrap(),
        11
    );
}

#[tokio::test]
async fn comment_submission_requires_current_identity_and_current_ddl() {
    let (_dir, app, pool) = setup().await;
    for input in [
        serde_json::json!({"content":"Missing identity"}),
        serde_json::json!({"content":"Invalid identity","request_id":"not-a-uuid"}),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/message-comments")
                    .header("content-type", "application/json")
                    .body(Body::from(input.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    let identity = crate::schema::current_identity().unwrap();
    assert_eq!(identity.application_version, "blog-db-v1");
    assert_eq!(identity.schema_revision, 1);
    crate::schema::validate(&pool).await.unwrap();
    sqlx::query("DROP TABLE comment_submission")
        .execute(&pool)
        .await
        .unwrap();
    assert!(crate::schema::validate(&pool).await.is_err());
}

#[tokio::test]
async fn comment_normalization_matches_shared_unicode_fixtures() {
    let fixtures: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../integration/fixtures/comment-whitespace.json"
    ))
    .unwrap();
    let (_dir, app, _pool) = setup().await;
    for (index, fixture) in fixtures.iter().enumerate() {
        let input = fixture["input"].as_str().unwrap();
        let expected = fixture["expected"].as_str().unwrap();
        assert_eq!(input.trim(), expected, "{}", fixture["name"]);
        let response =
            submit_message_comment(&app, None, &comment_request_id(index as u64 + 1), input).await;
        if expected.is_empty() {
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        } else {
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                body_json(response).await["comment_content"],
                expected,
                "{}",
                fixture["name"]
            );
        }
    }
}

#[path = "tests/taxonomy.rs"]
mod taxonomy;

#[path = "tests/media_names.rs"]
mod media_names;

#[path = "tests/family.rs"]
mod family;

#[path = "tests/schema_v3.rs"]
mod schema_v3;

#[path = "tests/public_browse.rs"]
mod public_browse;

#[path = "tests/management.rs"]
mod management;

#[path = "tests/media_management.rs"]
mod media_management;
#[path = "tests/slow_body.rs"]
mod slow_body;

#[tokio::test]
async fn article_password_checks_ignore_forwarding_headers_and_failure_history() {
    let (_dir, app, pool) = setup().await;
    let hash = blog_application::admin_auth::hash_password("ProtectedPassphrase123").unwrap();
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,view_status,password) VALUES(1,1,0,0,'Protected','Secret',0,?)")
        .bind(hash).execute(&pool).await.unwrap();
    let request = |visitor: &str, password: &str| {
        Request::builder()
            .method("POST")
            .uri("/api/v1/articles/1/unlock")
            .header("content-type", "application/json")
            .header("x-forwarded-for", format!("{visitor}, 127.0.0.2"))
            .header("x-real-ip", "203.0.113.250")
            .body(Body::from(
                serde_json::json!({"password":password}).to_string(),
            ))
            .unwrap()
    };
    for _attempt in 1..=11 {
        let response = app
            .clone()
            .oneshot(request("192.0.2.1", "wrong password"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    let response = app
        .clone()
        .oneshot(request("192.0.2.2", "wrong password"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = app
        .oneshot(request("192.0.2.2", "ProtectedPassphrase123"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn article_comment_gate_is_rechecked_after_waiting_for_a_writer() {
    for change in [
        "view_status=0",
        "deleted=1",
        "comment_status=0",
        "password='protected'",
    ] {
        let (_dir, app, pool) = setup_with_pool_limit(2).await;
        sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,view_status,comment_status) VALUES(1,1,0,0,'Open','Body',1,1)")
            .execute(&pool).await.unwrap();
        // WAL permits the old pre-transaction permission read while this
        // administrator write is uncommitted, but insertion has to wait.
        let mut writer = pool.begin_with("BEGIN IMMEDIATE").await.unwrap();
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE article SET {change} WHERE id=1"
        )))
        .execute(&mut *writer)
        .await
        .unwrap();
        let request = tokio::spawn(async move {
            anonymous_post(
                &app,
                "/api/v1/comments",
                serde_json::json!({
                    "article_id": 1, "content": "Racing comment"
                }),
            )
            .await
        });
        // Give the request time to finish its read and reach the blocked
        // writer boundary. It must not complete until the writer releases it.
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut held_since = None;
            loop {
                if pool.num_idle() == 0 {
                    let since = held_since.get_or_insert_with(tokio::time::Instant::now);
                    if since.elapsed() >= std::time::Duration::from_millis(50) {
                        break;
                    }
                } else {
                    held_since = None;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("comment request did not reach the blocked writer");
        assert!(!request.is_finished());
        writer.commit().await.unwrap();
        let response = request.await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{change}");
        for table in ["comment", "comment_submission"] {
            let count: i64 =
                sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(count, 0, "{change}: {table}");
        }
    }
}

#[tokio::test]
async fn article_related_reads_keep_authorization_and_content_in_one_snapshot() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    for (path, release_after_gate) in [
        ("/api/v1/comments?article_id=1", 1),
        ("/api/v1/comments/1/replies", 2),
        ("/api/v1/articles/1/news", 1),
    ] {
        let (directory, _app, fixture_pool) = setup().await;
        sqlx::raw_sql("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,0,0,'Public','Body'); INSERT INTO comment(id,source,type,parent_comment_id,floor_comment_id,comment_content) VALUES(1,1,'article',0,NULL,'public root'),(2,1,'article',1,1,'public reply'); INSERT INTO wei_yan(id,user_id,content,type,source,is_public) VALUES(1,1,'public news','news',1,1);")
            .execute(&fixture_pool).await.unwrap();
        let releases = Arc::new(AtomicUsize::new(0));
        let observed = releases.clone();
        // A one-connection pool deterministically commits an administrator edit
        // when the handler releases its authorization read. A read transaction
        // keeps that connection until its complete response has been assembled.
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .after_release(move |connection, _| {
                let observed = observed.clone();
                Box::pin(async move {
                    if observed.fetch_add(1, Ordering::SeqCst) + 1 == release_after_gate {
                        sqlx::raw_sql("BEGIN IMMEDIATE; UPDATE article SET view_status=0,password='new protection' WHERE id=1; UPDATE comment SET comment_content='new restricted comment' WHERE source=1; UPDATE wei_yan SET content='new restricted news' WHERE source=1; COMMIT;")
                            .execute(connection).await?;
                    }
                    Ok(true)
                })
            })
            .connect_lazy_with((*fixture_pool.connect_options()).clone());
        let state = AppState {
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
        };
        let app = router(state, None).unwrap();
        let response = app
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let body = body_json(response).await;
        if path.ends_with("/news") {
            assert_eq!(body.as_array().unwrap().len(), 1);
            assert_eq!(body[0]["content"], "public news");
            pool.close().await;
            continue;
        }
        assert_eq!(body["total"], 1, "{path}");
        assert_eq!(
            body["items"][0]["comment_content"],
            if release_after_gate == 1 {
                "public root"
            } else {
                "public reply"
            },
            "{path}: permission and content must describe the same snapshot"
        );
        pool.close().await;
    }
}
