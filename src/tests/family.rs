use super::*;

async fn login(app: &Router) -> (String, String) {
    let response = app
        .clone()
        .oneshot(with_peer(
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
        ))
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

fn details(visible: bool) -> serde_json::Value {
    serde_json::json!({
        "bg_cover":null, "man_cover":null, "woman_cover":null,
        "man_name":"Owner", "woman_name":"Partner", "timing":"2026-10-01",
        "countdown_title":null, "countdown_time":null, "family_info":"Our home", "status":visible
    })
}

async fn admin_request(
    app: &Router,
    session: &(String, String),
    method: &str,
    path: &str,
    value: serde_json::Value,
) -> Response {
    app.clone()
        .oneshot(with_peer(
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
        ))
        .await
        .unwrap()
}

async fn public_home(app: &Router) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(with_peer(
            Request::builder()
                .uri("/api/v1/family")
                .body(Body::empty())
                .unwrap(),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

#[tokio::test]
async fn family_is_admin_owned_and_legacy_stories_do_not_reappear() {
    let (_directory, app, pool) = setup().await;
    sqlx::query("INSERT INTO user(id,username,user_type,user_status) VALUES(2,'other-author',2,1)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO family(id,user_id,man_name,woman_name,status) VALUES(1,1,'Earlier','Partner',1),(2,1,'Current','Partner',1),(3,2,'Other','Partner',1)")
        .execute(&pool).await.unwrap();
    let home = public_home(&app).await;
    assert_eq!(home.as_array().unwrap().len(), 1);
    assert_eq!(home[0]["id"], 2);
    assert!(home[0].get("like_count").is_none());

    for (method, path) in [
        ("GET", "/api/v1/content/family"),
        ("POST", "/api/v1/content/family"),
        ("PUT", "/api/v1/content/family/2"),
        ("DELETE", "/api/v1/content/family/2"),
    ] {
        let response = app
            .clone()
            .oneshot(with_peer(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(details(true).to_string()))
                    .unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {path}"
        );
    }
    let session = login(&app).await;
    let response = admin_request(
        &app,
        &session,
        "GET",
        "/api/v1/content/family",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let homes = body_json(response).await;
    assert_eq!(homes.as_array().unwrap().len(), 1);
    assert_eq!(homes[0]["id"], 2);
    for id in [1, 3] {
        for method in ["PUT", "DELETE"] {
            let response = admin_request(
                &app,
                &session,
                method,
                &format!("/api/v1/content/family/{id}"),
                details(true),
            )
            .await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{method} {id}");
        }
    }
    let response = admin_request(
        &app,
        &session,
        "PUT",
        "/api/v1/content/family/2",
        details(false),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(public_home(&app).await, serde_json::json!([]));
    let response = admin_request(
        &app,
        &session,
        "POST",
        "/api/v1/content/family",
        details(true),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let response = admin_request(
        &app,
        &session,
        "DELETE",
        "/api/v1/content/family/2",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(public_home(&app).await, serde_json::json!([]));
    let response = admin_request(
        &app,
        &session,
        "GET",
        "/api/v1/content/family",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(body_json(response).await, serde_json::json!([]));
    let retained: Vec<(i64, Option<i64>, String)> =
        sqlx::query_as("SELECT id,user_id,man_name FROM family ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        retained,
        vec![(1, None, "Earlier".into()), (3, Some(2), "Other".into())]
    );
    let mut forged = details(true);
    forged["user_id"] = 2.into();
    let response = admin_request(&app, &session, "POST", "/api/v1/content/family", forged).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = admin_request(
        &app,
        &session,
        "POST",
        "/api/v1/content/family",
        details(true),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(public_home(&app).await.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn concurrent_family_creation_only_creates_one_home() {
    let (_directory, app, pool) = setup().await;
    let session = login(&app).await;
    let (first, second) = tokio::join!(
        admin_request(
            &app,
            &session,
            "POST",
            "/api/v1/content/family",
            details(true)
        ),
        admin_request(
            &app,
            &session,
            "POST",
            "/api/v1/content/family",
            details(true)
        ),
    );
    assert!(matches!(
        (first.status(), second.status()),
        (StatusCode::OK, StatusCode::CONFLICT) | (StatusCode::CONFLICT, StatusCode::OK)
    ));
    let homes: Vec<i64> = sqlx::query_scalar("SELECT user_id FROM family")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(homes, vec![1]);
    let home = public_home(&app).await;
    assert_eq!(home.as_array().unwrap().len(), 1);
    assert_eq!(home[0]["man_name"], "Owner");
}
