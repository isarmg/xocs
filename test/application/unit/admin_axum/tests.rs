use super::*;
use axum::body::to_bytes;
use axum::extract::ConnectInfo;
use axum::http::{Method, Request as HttpRequest};
use std::net::SocketAddr;
use tower::ServiceExt;

fn request(method: Method, path: &str, body: &str) -> HttpRequest<axum::body::Body> {
    let mut request = HttpRequest::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ORIGIN, "http://127.0.0.1")
        .header(header::HOST, "127.0.0.1")
        .header("sec-fetch-site", "same-origin")
        .body(axum::body::Body::from(body.to_owned()))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
    request
}

async fn router() -> Router {
    let store = crate::testkit::administrator_fixture("admin", "correct horse battery").await;
    administrator_router(
        "example-product",
        AdministratorOriginMode::LoopbackDevelopmentHttp,
        Arc::new(AdministratorService::with_isolated_test_budget(store)),
    )
    .unwrap()
}

#[tokio::test]
async fn login_attempts_return_429_and_recover_without_success_reset() {
    use crate::admin_core::{AdministratorStore, LOGIN_ATTEMPTS_MAX};
    let store = crate::testkit::administrator_fixture("admin", "correct horse battery").await;
    let state = store.authentication_state().await.unwrap();
    let service = Arc::new(AdministratorService::with_isolated_test_budget(store));
    let router = administrator_router(
        "example-product",
        AdministratorOriginMode::LoopbackDevelopmentHttp,
        service,
    )
    .unwrap();
    for _ in 0..LOGIN_ATTEMPTS_MAX - 2 {
        state.admit_login_attempt().unwrap();
    }
    let failed = router
        .clone()
        .oneshot(request(
            Method::POST,
            ADMIN_LOGIN_PATH,
            r#"{"username":"admin","password":"incorrect password"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(failed.status(), StatusCode::UNAUTHORIZED);
    let success = router
        .clone()
        .oneshot(request(
            Method::POST,
            ADMIN_LOGIN_PATH,
            r#"{"username":"admin","password":"correct horse battery"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(success.status(), StatusCode::OK);
    let mut login = request(
        Method::POST,
        ADMIN_LOGIN_PATH,
        r#"{"username":"other-user","password":"incorrect password"}"#,
    );
    login
        .headers_mut()
        .insert("x-forwarded-for", "192.0.2.4".parse().unwrap());
    let limited = router.clone().oneshot(login).await.unwrap();
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after = limited.headers()[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    assert!((1..=60).contains(&retry_after));
    assert!(
        limited.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("no-store")
    );
    let envelope: ErrorEnvelope =
        serde_json::from_slice(&to_bytes(limited.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(envelope.code.as_str(), "auth.login_attempts_exceeded");
    assert!(envelope.retryable);
    assert_eq!(envelope.details["retry_after"], retry_after);
    state.elapse_window_for_test();
    let restored = router
        .oneshot(request(
            Method::POST,
            ADMIN_LOGIN_PATH,
            r#"{"username":"admin","password":"correct horse battery"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(restored.status(), StatusCode::OK);
}

#[tokio::test]
async fn verified_access_log_metadata_survives_logout_without_credentials() {
    let router = router().await;
    let bad = router
        .clone()
        .oneshot(request(
            Method::POST,
            ADMIN_LOGIN_PATH,
            r#"{"username":"admin","password":"incorrect password"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);
    assert!(bad.extensions().get::<VerifiedAdministrator>().is_none());
    let login = router
        .clone()
        .oneshot(request(
            Method::POST,
            ADMIN_LOGIN_PATH,
            r#"{"username":"admin","password":"correct horse battery"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let metadata = login.extensions().get::<VerifiedAdministrator>().unwrap();
    assert_eq!(metadata.username(), "admin");
    assert_eq!(
        format!("{metadata:?}"),
        "VerifiedAdministrator { username: \"admin\" }"
    );
    let cookie = login.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let data: crate::contracts::AdministratorSession =
        serde_json::from_slice(&to_bytes(login.into_body(), 4096).await.unwrap()).unwrap();
    let mut logout = request(Method::POST, ADMIN_LOGOUT_PATH, "");
    logout
        .headers_mut()
        .insert(header::COOKIE, cookie.parse().unwrap());
    logout.headers_mut().insert(
        crate::admin_auth::CSRF_HEADER,
        data.csrf_token.parse().unwrap(),
    );
    let response = router.clone().oneshot(logout).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response
            .extensions()
            .get::<VerifiedAdministrator>()
            .unwrap()
            .username(),
        "admin"
    );
    let mut restore = request(Method::GET, ADMIN_SESSION_PATH, "");
    restore
        .headers_mut()
        .insert(header::COOKIE, cookie.parse().unwrap());
    let response = router.oneshot(restore).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(
        response
            .extensions()
            .get::<VerifiedAdministrator>()
            .is_none()
    );
}

#[tokio::test]
async fn shared_persistent_management_contract() {
    let directory = tempfile::tempdir().unwrap();
    let pool = crate::sqlite::create_if_missing(
        directory.path().join("admin.sqlite3"),
        crate::sqlite::PoolOptions::new(2),
    )
    .await
    .unwrap();
    sqlx::raw_sql(crate::admin_sqlite::ADMIN_PERSISTENT_DDL)
        .execute(&pool)
        .await
        .unwrap();
    let service = Arc::new(AdministratorService::with_isolated_test_budget(
        crate::admin_sqlite::SqliteAdministratorStore::new(pool),
    ));
    crate::testkit::initialize_administrator_fixture(
        service.store().pool(),
        "admin",
        crate::testkit::ADMINISTRATOR_PASSWORD,
        1,
    )
    .await;
    let router = administrator_router(
        "example-product",
        AdministratorOriginMode::LoopbackDevelopmentHttp,
        service,
    )
    .unwrap();
    crate::testkit::assert_administrator_management_http_contract(|mut request| {
        let router = router.clone();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
        async move { router.oneshot(request).await.unwrap() }
    })
    .await;
}

#[tokio::test]
async fn retired_administrator_routes_are_not_registered() {
    let router = router().await;
    for (method, path) in [
        (
            Method::GET,
            crate::contracts::ADMINISTRATORS_PATH.to_string(),
        ),
        (
            Method::POST,
            crate::contracts::ADMINISTRATORS_PATH.to_string(),
        ),
        (
            Method::POST,
            format!("{}/admin/password", crate::contracts::ADMINISTRATORS_PATH),
        ),
        (
            Method::POST,
            format!("{}/admin/disable", crate::contracts::ADMINISTRATORS_PATH),
        ),
    ] {
        assert_eq!(
            router
                .clone()
                .oneshot(request(method, &path, ""))
                .await
                .unwrap()
                .status(),
            StatusCode::NOT_FOUND
        );
    }
}

#[tokio::test]
async fn shared_administrator_wire_contract() {
    let application = router().await;
    crate::testkit::assert_administrator_http_contract(
        |mut request| {
            let application = application.clone();
            request
                .extensions_mut()
                .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 1234))));
            async move { application.oneshot(request).await.unwrap() }
        },
        "example-product",
    )
    .await;
}

#[tokio::test]
async fn rejects_duplicate_origin_and_unknown_json() {
    let application = router().await;
    let mut duplicate = request(
        Method::POST,
        ADMIN_LOGIN_PATH,
        r#"{"username":"admin","password":"correct horse battery"}"#,
    );
    duplicate
        .headers_mut()
        .append(header::ORIGIN, HeaderValue::from_static("http://127.0.0.1"));
    assert_eq!(
        application
            .clone()
            .oneshot(duplicate)
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );

    let unknown = request(
        Method::POST,
        ADMIN_LOGIN_PATH,
        r#"{"username":"admin","password":"correct horse battery","extra":true}"#,
    );
    assert_eq!(
        application.oneshot(unknown).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn successful_login_uses_admin_cookie_and_contract() {
    let response = router()
        .await
        .oneshot(request(
            Method::POST,
            ADMIN_LOGIN_PATH,
            r#"{"username":" ADMIN ","password":"correct horse battery"}"#,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cookie.starts_with("admin-example-product-session="));
    assert!(cookie.ends_with("; Path=/; HttpOnly; SameSite=Strict"));
    let body = to_bytes(response.into_body(), 8 * 1024).await.unwrap();
    let session: crate::contracts::AdministratorSession = serde_json::from_slice(&body).unwrap();
    assert_eq!(session.username, "admin");
}
