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
    let response = body_json(response).await;
    (cookie, response["csrf_token"].as_str().unwrap().to_owned())
}

async fn write(
    app: &Router,
    session: &(String, String),
    method: &str,
    path: &str,
    value: Value,
    key: Option<&str>,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, "127.0.0.1:18881")
        .header(header::ORIGIN, "http://127.0.0.1:18881")
        .header("sec-fetch-site", "same-origin")
        .header(header::COOKIE, &session.0)
        .header("x-csrf-token", &session.1)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(key) = key {
        request = request.header("Idempotency-Key", key);
    }
    app.clone()
        .oneshot(request.body(Body::from(value.to_string())).unwrap())
        .await
        .unwrap()
}

async fn seed(pool: &SqlitePool) {
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,1,1,'Timeline','Body')")
        .execute(pool).await.unwrap();
}

#[tokio::test]
async fn news_rejects_relative_times_and_invalid_calendar_dates_without_mutation() {
    let (_directory, app, pool) = setup().await;
    let auth = session(&app).await;
    seed(&pool).await;
    let invalid = [
        "now",
        "12:00",
        "2451545",
        "2023-02-29T12:00:00Z",
        "2026-02-30T12:00:00",
        "2026-04-31",
        "2026-01-01T24:00:00Z",
        "2026-01-01T12:60:00Z",
        "2026-01-01T12:00:60Z",
        "+10000-01-01T00:00:00",
        "0000-01-01T00:00:00+01:00",
    ];
    for time in invalid {
        let response = write(
            &app,
            &auth,
            "POST",
            "/api/v1/content/articles/1/news",
            json!({"content":"Must not commit","create_time":time}),
            Some("correctable-time"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{time}");
    }
    for table in ["wei_yan", "article_news_submission"] {
        let count: i64 =
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 0, "{table}");
    }
    let response = write(
        &app,
        &auth,
        "POST",
        "/api/v1/content/articles/1/news",
        json!({"content":"Kept","create_time":"2024-02-29T12:00:00Z"}),
        Some("correctable-time"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let saved = body_json(response).await;
    let path = format!("/api/v1/content/articles/1/news/{}", saved["id"]);
    for time in invalid {
        let response = write(
            &app,
            &auth,
            "PUT",
            &path,
            json!({"content":"Must not replace","create_time":time}),
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{time}");
    }
    let stored: (String, String) =
        sqlx::query_as("SELECT content,create_time FROM wei_yan WHERE id=?")
            .bind(saved["id"].as_i64().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored, ("Kept".into(), "2024-02-29 12:00:00".into()));
}

#[tokio::test]
async fn news_normalized_absolute_time_keeps_lost_response_retries_on_one_record() {
    let (_directory, app, pool) = setup().await;
    let auth = session(&app).await;
    seed(&pool).await;
    let response = write(
        &app,
        &auth,
        "POST",
        "/api/v1/content/articles/1/news",
        json!({"content":" Kept update ","create_time":"2024-02-29T08:30:45.125+02:00"}),
        Some("absolute-time"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    drop(response); // The write committed, but the editor did not receive its body.
    for time in ["2024-02-29T06:30:45.125Z", "2024-02-29 06:30:45"] {
        let response = write(
            &app,
            &auth,
            "POST",
            "/api/v1/content/articles/1/news",
            json!({"content":"Kept update","create_time":time}),
            Some("absolute-time"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let saved = body_json(response).await;
        assert_eq!(saved["content"], "Kept update");
        assert_eq!(saved["create_time"], "2024-02-29 06:30:45");
    }
    let counts: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM wei_yan),(SELECT COUNT(*) FROM article_news_submission)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (1, 1));
}

#[tokio::test]
async fn news_accepts_absolute_utc_dates_and_blank_edit_preserves_the_saved_time() {
    let (_directory, app, pool) = setup().await;
    let auth = session(&app).await;
    seed(&pool).await;
    for (time, expected) in [
        ("2024-02-29", "2024-02-29 00:00:00"),
        ("2024-02-29T12:30", "2024-02-29 12:30:00"),
        ("2024-02-29 12:30:45.999", "2024-02-29 12:30:45"),
        ("2024-03-01T00:30:00+01:00", "2024-02-29 23:30:00"),
    ] {
        let response = write(
            &app,
            &auth,
            "POST",
            "/api/v1/content/articles/1/news",
            json!({"content":"Absolute update","create_time":time}),
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK, "{time}");
        let saved = body_json(response).await;
        assert_eq!(saved["create_time"], expected);
        let path = format!("/api/v1/content/articles/1/news/{}", saved["id"]);
        for blank in [json!(null), json!(" ")] {
            let response = write(
                &app,
                &auth,
                "PUT",
                &path,
                json!({"content":"Edited update","create_time":blank}),
                None,
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(body_json(response).await["create_time"], expected);
        }
    }
}
