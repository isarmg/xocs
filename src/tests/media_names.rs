use super::*;

#[tokio::test]
async fn uploaded_unicode_names_roundtrip_through_resources_and_media() {
    let (_directory, app, pool) = setup().await;
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
    let session = body_json(response).await;
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("../../web/tests/fixtures/upload-names.json")).unwrap();
    for (index, fixture) in fixtures.as_array().unwrap().iter().enumerate() {
        let name = fixture["name"].as_str().unwrap();
        // Distinct valid GIFs avoid content-addressed deduplication between names.
        let mut image = hex::decode("47494638396101000100800000000000ffffff21f90401000000002c00000000010001000002024401003b").unwrap();
        image[13] = index as u8;
        let response = app
            .clone()
            .oneshot(with_peer(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/content/upload")
                    .header(header::HOST, "127.0.0.1:18881")
                    .header(header::ORIGIN, "http://127.0.0.1:18881")
                    .header("sec-fetch-site", "same-origin")
                    .header(header::COOKIE, &cookie)
                    .header("x-csrf-token", session["csrf_token"].as_str().unwrap())
                    .header(header::CONTENT_TYPE, "image/gif")
                    .header("x-file-name", fixture["header"].as_str().unwrap())
                    .body(Body::from(image.clone()))
                    .unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let uploaded = body_json(response).await;
        let id = uploaded["id"].as_i64().unwrap();
        let stored: String = sqlx::query_scalar("SELECT original_name FROM resource WHERE id=?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(stored, name);
        let listed = app
            .clone()
            .oneshot(with_peer(
                Request::builder()
                    .uri("/api/v1/content/resources")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        let list = body_json(listed).await;
        assert!(
            list["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["id"] == id && item["original_name"] == name)
        );
        let downloaded = app
            .clone()
            .oneshot(with_peer(
                Request::builder()
                    .uri(uploaded["path"].as_str().unwrap())
                    .body(Body::empty())
                    .unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(downloaded.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(downloaded.into_body(), 1024)
                .await
                .unwrap()
                .as_ref(),
            image
        );
    }
}
