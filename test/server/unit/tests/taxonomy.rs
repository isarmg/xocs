use super::*;

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

async fn write(
    app: &Router,
    session: &(String, String),
    method: &str,
    path: &str,
    value: serde_json::Value,
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

fn article(sort: i64, label: i64) -> serde_json::Value {
    serde_json::json!({
        "article_title":"Owned fixture", "article_content":"Test body", "article_cover":null,
        "video_url":null, "sort_id":sort, "label_id":label, "view_status":true,
        "recommend_status":false, "comment_status":true, "password":null, "tips":null
    })
}

async fn seed(pool: &SqlitePool) {
    sqlx::query("INSERT INTO sort(id,sort_name) VALUES(1,'A'),(2,'B')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO label(id,sort_id,label_name) VALUES(1,1,'Used'),(2,1,'Unused'),(3,2,'Other'),(4,1,'Concurrent')")
        .execute(pool).await.unwrap();
}

#[tokio::test]
async fn used_label_cannot_move_but_can_be_renamed_and_unused_label_can_move() {
    let (_directory, app, pool) = setup().await;
    let session = login(&app).await;
    seed(&pool).await;
    let created = write(
        &app,
        &session,
        "POST",
        "/api/v1/content/articles",
        article(1, 1),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let moved = write(
        &app,
        &session,
        "PUT",
        "/api/v1/content/labels/1",
        serde_json::json!({"sort_id":2,"name":"Used","description":null}),
    )
    .await;
    assert_eq!(moved.status(), StatusCode::CONFLICT);
    assert!(
        body_json(moved).await["message"]
            .as_str()
            .unwrap()
            .contains("不能更改分类")
    );
    let renamed = write(
        &app,
        &session,
        "PUT",
        "/api/v1/content/labels/1",
        serde_json::json!({"sort_id":1,"name":"Renamed","description":"Updated"}),
    )
    .await;
    assert_eq!(renamed.status(), StatusCode::OK);
    assert_eq!(body_json(renamed).await["label_name"], "Renamed");
    let unused = write(
        &app,
        &session,
        "PUT",
        "/api/v1/content/labels/2",
        serde_json::json!({"sort_id":2,"name":"Unused","description":null}),
    )
    .await;
    assert_eq!(unused.status(), StatusCode::OK);
    assert_eq!(body_json(unused).await["sort_id"], 2);
    let stored: (i64, i64) = sqlx::query_as("SELECT sort_id,label_id FROM article")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, (1, 1));
}

#[tokio::test]
async fn article_create_and_update_reject_mismatched_or_missing_taxonomy_without_changes() {
    let (_directory, app, pool) = setup().await;
    let session = login(&app).await;
    seed(&pool).await;
    for (sort, label) in [(1, 3), (2, 1), (999, 1), (1, 999)] {
        let response = write(
            &app,
            &session,
            "POST",
            "/api/v1/content/articles",
            article(sort, label),
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    let created = write(
        &app,
        &session,
        "POST",
        "/api/v1/content/articles",
        article(1, 1),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let id = body_json(created).await["id"].as_i64().unwrap();
    for (sort, label) in [(1, 3), (2, 1), (999, 1), (1, 999)] {
        let response = write(
            &app,
            &session,
            "PUT",
            &format!("/api/v1/content/articles/{id}"),
            article(sort, label),
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    let stored: (i64, i64) = sqlx::query_as("SELECT sort_id,label_id FROM article WHERE id=?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, (1, 1));
}

#[tokio::test]
async fn concurrent_label_move_and_article_writes_cannot_commit_an_inconsistent_pair() {
    let (_directory, app, pool) = setup().await;
    let session = login(&app).await;
    seed(&pool).await;
    let (created, moved) = tokio::join!(
        write(
            &app,
            &session,
            "POST",
            "/api/v1/content/articles",
            article(1, 4)
        ),
        write(
            &app,
            &session,
            "PUT",
            "/api/v1/content/labels/4",
            serde_json::json!({"sort_id":2,"name":"Concurrent","description":null})
        )
    );
    assert!(matches!(
        (created.status(), moved.status()),
        (StatusCode::OK, StatusCode::CONFLICT) | (StatusCode::BAD_REQUEST, StatusCode::OK)
    ));
    let created = write(
        &app,
        &session,
        "POST",
        "/api/v1/content/articles",
        article(1, 1),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let id = body_json(created).await["id"].as_i64().unwrap();
    let article_path = format!("/api/v1/content/articles/{id}");
    let (updated, moved) = tokio::join!(
        write(&app, &session, "PUT", &article_path, article(1, 2)),
        write(
            &app,
            &session,
            "PUT",
            "/api/v1/content/labels/2",
            serde_json::json!({"sort_id":2,"name":"Unused","description":null})
        )
    );
    assert!(matches!(
        (updated.status(), moved.status()),
        (StatusCode::OK, StatusCode::CONFLICT) | (StatusCode::BAD_REQUEST, StatusCode::OK)
    ));
    let inconsistent: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM article a LEFT JOIN label l ON l.id=a.label_id LEFT JOIN sort s ON s.id=a.sort_id WHERE a.deleted=0 AND (l.id IS NULL OR s.id IS NULL OR a.sort_id<>l.sort_id)")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(inconsistent, 0);
}

#[tokio::test]
async fn deleting_an_empty_category_removes_its_labels_before_id_reuse() {
    let (_directory, app, pool) = setup().await;
    let session = login(&app).await;
    seed(&pool).await;
    let deleted = write(
        &app,
        &session,
        "DELETE",
        "/api/v1/content/categories/2",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::OK);
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM label WHERE sort_id=2")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        remaining, 0,
        "Deleted categories must not leave visible orphan labels or attach them to a later category that reuses the ID"
    );
    let created = write(
        &app,
        &session,
        "POST",
        "/api/v1/content/categories",
        serde_json::json!({"name":"Replacement","description":null,"priority":0}),
    )
    .await;
    assert_eq!(created.status(), StatusCode::OK);
    let id = body_json(created).await["id"].as_i64().unwrap();
    let inherited: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM label WHERE sort_id=?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(inherited, 0);
}

#[tokio::test]
async fn category_deletion_and_label_creation_cannot_leave_orphan_labels() {
    let (_directory, app, pool) = setup().await;
    let session = login(&app).await;
    seed(&pool).await;
    let (deleted, created) = tokio::join!(
        write(
            &app,
            &session,
            "DELETE",
            "/api/v1/content/categories/2",
            serde_json::json!({})
        ),
        write(
            &app,
            &session,
            "POST",
            "/api/v1/content/labels",
            serde_json::json!({"sort_id":2,"name":"New tag","description":null})
        )
    );
    assert_eq!(deleted.status(), StatusCode::OK);
    assert!(matches!(
        created.status(),
        StatusCode::OK | StatusCode::BAD_REQUEST
    ));
    let orphans: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM label l LEFT JOIN sort s ON s.id=l.sort_id WHERE s.id IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(orphans, 0);
}

async fn create_with_key(
    app: &Router,
    session: &(String, String),
    key: &str,
    value: serde_json::Value,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/content/articles")
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
async fn article_create_receipt_tracks_initial_payload_and_current_article() {
    let (_dir, app, pool) = setup().await;
    seed(&pool).await;
    let session = login(&app).await;
    let original = article(1, 1);
    let mut bad = original.clone();
    bad["article_title"] = "".into();
    assert_eq!(
        create_with_key(&app, &session, "stable-key", bad)
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    let first = create_with_key(&app, &session, "stable-key", original.clone()).await;
    assert_eq!(first.status(), StatusCode::OK);
    let first = body_json(first).await;
    let id = first["id"].as_i64().unwrap();
    let mut edited = original.clone();
    edited["article_title"] = "Edited after initial save".into();
    assert_eq!(
        write(
            &app,
            &session,
            "PUT",
            &format!("/api/v1/content/articles/{id}"),
            edited.clone()
        )
        .await
        .status(),
        StatusCode::OK
    );
    let replay = create_with_key(&app, &session, "stable-key", original.clone()).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay = body_json(replay).await;
    assert_eq!(replay["id"], id);
    assert_eq!(replay["article_title"], edited["article_title"]);
    assert_eq!(
        create_with_key(&app, &session, "stable-key", edited)
            .await
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        write(
            &app,
            &session,
            "DELETE",
            &format!("/api/v1/content/articles/{id}"),
            serde_json::json!({})
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        create_with_key(&app, &session, "stable-key", original)
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM article")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn article_search_uses_html_utf16_limit() {
    let (_dir, app, pool) = setup().await;
    seed(&pool).await;
    let session = login(&app).await;
    let mut value = article(1, 1);
    value["article_title"] = "文".repeat(80).into();
    assert_eq!(
        write(&app, &session, "POST", "/api/v1/content/articles", value)
            .await
            .status(),
        StatusCode::OK
    );
    for (search, status, total) in [
        ("文".repeat(80), StatusCode::OK, Some(1)),
        ("😀".repeat(100), StatusCode::OK, Some(0)),
        ("😀".repeat(101), StatusCode::BAD_REQUEST, None),
    ] {
        let query = url_encoding::percent::utf8_percent_encode(
            &search,
            url_encoding::percent::NON_ALPHANUMERIC,
        )
        .to_string();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/articles?search={query}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        if let Some(total) = total {
            assert_eq!(body_json(response).await["total"], total);
        }
    }
}

#[tokio::test]
async fn hard_deleted_ids_are_never_reused() {
    let (_dir, _app, pool) = setup().await;
    for (table, columns, values) in [
        ("comment", "source,comment_content", "1,'test'"),
        ("resource_path", "title", "'test'"),
        ("wei_yan", "content", "'test'"),
        ("tree_hole", "message", "'test'"),
        ("family", "man_name", "'test'"),
        ("sort", "sort_name", "'test'"),
        ("label", "sort_id,label_name", "1,'test'"),
    ] {
        let insert = format!("INSERT INTO {table}({columns}) VALUES({values})");
        let first = sqlx::query(sqlx::AssertSqlSafe(insert.clone()))
            .execute(&pool)
            .await
            .unwrap()
            .last_insert_rowid();
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM {table} WHERE id=?"
        )))
        .bind(first)
        .execute(&pool)
        .await
        .unwrap();
        let second = sqlx::query(sqlx::AssertSqlSafe(insert))
            .execute(&pool)
            .await
            .unwrap()
            .last_insert_rowid();
        assert!(second > first, "{table} reused {first}");
        let changed = sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM {table} WHERE id=?"
        )))
        .bind(first)
        .execute(&pool)
        .await
        .unwrap();
        assert_eq!(changed.rows_affected(), 0);
    }
}

#[tokio::test]
async fn concurrent_article_replays_share_one_receipt_on_a_single_connection() {
    let (_dir, app, pool) = setup_with_pool_limit(1).await;
    seed(&pool).await;
    let session = login(&app).await;
    let (first, second) = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::join!(
            create_with_key(&app, &session, "concurrent", article(1, 1)),
            create_with_key(&app, &session, "concurrent", article(1, 1))
        )
    })
    .await
    .expect("a receipt must not perform nested pool checkout");
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(body_json(first).await["id"], body_json(second).await["id"]);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM article")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn failed_taxonomy_leaves_key_available_and_unkeyed_posts_remain_independent() {
    let (_dir, app, pool) = setup().await;
    seed(&pool).await;
    let session = login(&app).await;
    assert_eq!(
        create_with_key(&app, &session, "correctable", article(1, 3))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        create_with_key(&app, &session, "correctable", article(1, 1))
            .await
            .status(),
        StatusCode::OK
    );
    for _ in 0..2 {
        assert_eq!(
            write(
                &app,
                &session,
                "POST",
                "/api/v1/content/articles",
                article(1, 1)
            )
            .await
            .status(),
            StatusCode::OK
        );
    }
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM article")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn article_receipt_survives_database_reopen_with_initial_fingerprint() {
    let (dir, app, pool) = setup().await;
    seed(&pool).await;
    let session = login(&app).await;
    let original = article(1, 1);
    assert_eq!(
        create_with_key(&app, &session, "durable", original.clone())
            .await
            .status(),
        StatusCode::OK
    );
    let before: (Vec<u8>, i64) = sqlx::query_as(
        "SELECT request_hash,article_id FROM article_submission WHERE request_id='durable'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut updated = original;
    updated["article_title"] = "changed".into();
    assert_eq!(
        write(
            &app,
            &session,
            "PUT",
            &format!("/api/v1/content/articles/{}", before.1),
            updated
        )
        .await
        .status(),
        StatusCode::OK
    );
    close_fixture_pool(&pool).await;
    let reopened = open_database(&dir.path().join("site.sqlite"))
        .await
        .unwrap();
    let after: (Vec<u8>, i64) = sqlx::query_as(
        "SELECT request_hash,article_id FROM article_submission WHERE request_id='durable'",
    )
    .fetch_one(&reopened)
    .await
    .unwrap();
    assert_eq!(before, after);
}
