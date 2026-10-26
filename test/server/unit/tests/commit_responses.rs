use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

struct Case {
    name: &'static str,
    seed: &'static str,
    method: &'static str,
    path: &'static str,
    input: Value,
    next_write: &'static str,
    response_field: &'static str,
    expected: &'static str,
    replay: bool,
}

fn cases() -> Vec<Case> {
    let article = json!({
        "article_title":"mine", "article_content":"My submitted body", "article_cover":null,
        "video_url":null, "sort_id":1, "label_id":1, "view_status":true,
        "recommend_status":false, "comment_status":true, "password":null, "tips":null
    });
    let link = json!({
        "title":"mine", "classify":null, "cover":null, "url":"https://example.test/",
        "introduction":null, "link_type":"favorites", "status":true
    });
    let family = json!({
        "bg_cover":null, "man_cover":null, "woman_cover":null, "man_name":"mine",
        "woman_name":"Partner", "timing":"2026-10-01", "countdown_title":null,
        "countdown_time":null, "family_info":null, "status":true
    });
    let mut cases = Vec::new();
    for (name, seed, method, path) in [
        ("create category", "", "POST", "/api/v1/content/categories"),
        ("update category", "", "PUT", "/api/v1/content/categories/1"),
    ] {
        cases.push(Case {
            name,
            seed,
            method,
            path,
            input: json!({"name":"mine"}),
            next_write: "UPDATE sort SET sort_name='later' WHERE sort_name='mine'",
            response_field: "/sort_name",
            expected: "mine",
            replay: false,
        });
    }
    for (name, method, path, replay) in [
        ("create article", "POST", "/api/v1/content/articles", false),
        (
            "update article",
            "PUT",
            "/api/v1/content/articles/17",
            false,
        ),
        ("replay article", "POST", "/api/v1/content/articles", true),
    ] {
        cases.push(Case { name, seed:"", method, path, input:article.clone(),
            next_write:"UPDATE article SET article_title='later' WHERE article_title IN ('mine','edited before retry')",
            response_field:"/article_title", expected:if replay {"edited before retry"} else {"mine"}, replay });
    }
    for (name, seed, method, path, input) in [
        (
            "create link",
            "",
            "POST",
            "/api/v1/content/links",
            link.clone(),
        ),
        (
            "update link",
            "INSERT INTO resource_path(id,title,type,status) VALUES(17,'old','favorites',0)",
            "PUT",
            "/api/v1/content/links/17",
            link,
        ),
        (
            "change link status",
            "INSERT INTO resource_path(id,title,type,status) VALUES(17,'mine','favorites',0)",
            "PUT",
            "/api/v1/content/links/17/status",
            json!({"active":true}),
        ),
    ] {
        cases.push(Case {
            name,
            seed,
            method,
            path,
            input,
            next_write: "DELETE FROM resource_path WHERE title='mine' AND status=1",
            response_field: "/title",
            expected: "mine",
            replay: false,
        });
    }
    for (name, seed, method, path) in [
        ("create family", "", "POST", "/api/v1/content/family"),
        (
            "update family",
            "INSERT INTO family(id,user_id,man_name,woman_name,status) VALUES(17,1,'old','Partner',1)",
            "PUT",
            "/api/v1/content/family/17",
        ),
    ] {
        cases.push(Case {
            name,
            seed,
            method,
            path,
            input: family.clone(),
            next_write: "DELETE FROM family WHERE man_name='mine'",
            response_field: "/man_name",
            expected: "mine",
            replay: false,
        });
    }
    cases.push(Case {name:"update site", seed:"", method:"PUT", path:"/api/v1/content/site",
        input:json!({"author_name":"Owner", "web_name":"mine", "web_title":null,
            "notices":null, "background_image":null, "avatar":null, "random_cover":null, "media":{}}),
        next_write:"UPDATE web_info SET web_name='later' WHERE web_name='mine'",
        response_field:"/web_name", expected:"mine", replay:false });
    cases.push(Case {name:"replace home sections", seed:"", method:"PUT", path:"/api/v1/content/home-sections",
        input:json!([{"title":"mine", "kind":"latest", "sort_id":null, "priority":0, "enabled":true}]),
        next_write:"UPDATE home_sections SET title='later' WHERE title='mine'",
        response_field:"/0/title", expected:"mine", replay:false });
    cases.push(Case {
        name: "create guest message",
        seed: "",
        method: "POST",
        path: "/api/v1/tree-hole/guest",
        input: json!({"message":"mine", "image_path":null}),
        next_write: "DELETE FROM tree_hole WHERE message='mine'",
        response_field: "/message",
        expected: "mine",
        replay: false,
    });
    cases
}

#[tokio::test]
async fn committed_responses_survive_a_subsequent_writer_and_replays_read_current_content() {
    let mut failures = Vec::new();
    for case in cases() {
        let (directory, old_app, old_pool) = setup_with_pool_limit(1).await;
        sqlx::raw_sql("INSERT INTO sort(id,sort_name) VALUES(1,'Existing'); INSERT INTO label(id,sort_id,label_name) VALUES(1,1,'Existing'); INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(17,1,1,1,'old','old body')")
            .execute(&old_pool).await.unwrap();
        if !case.seed.is_empty() {
            sqlx::raw_sql(case.seed).execute(&old_pool).await.unwrap();
        }
        let session = session(&old_app).await;
        if case.replay {
            let response = old_app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(case.path)
                        .header(header::HOST, "127.0.0.1:18881")
                        .header(header::ORIGIN, "http://127.0.0.1:18881")
                        .header("sec-fetch-site", "same-origin")
                        .header(header::COOKIE, &session.0)
                        .header("x-csrf-token", &session.1)
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("Idempotency-Key", "response-race")
                        .body(Body::from(case.input.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            sqlx::query(
                "UPDATE article SET article_title='edited before retry' WHERE article_title='mine'",
            )
            .execute(&old_pool)
            .await
            .unwrap();
        }
        let service = AdministratorService::new(SqliteAdministratorStore::new(old_pool.clone()));
        let token = session.0.split_once('=').unwrap().1;
        let identity = service
            .authenticate_session(token, crate::app::now_micros().unwrap())
            .await
            .unwrap();
        let options = old_pool.connect_options().as_ref().clone();
        old_pool.close().await;

        // Schedule a real SQLite write exactly when the first transaction gives
        // back its connection. The one-connection pool makes its completion
        // precede any later acquisition, without timing or thread sleeps.
        let armed = Arc::new(AtomicBool::new(false));
        let ran = Arc::new(AtomicBool::new(false));
        let followup_armed = Arc::clone(&armed);
        let followup_ran = Arc::clone(&ran);
        let next_write = case.next_write;
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .after_release(move |connection, _| {
                let execute = followup_armed.swap(false, Ordering::SeqCst);
                let ran = Arc::clone(&followup_ran);
                Box::pin(async move {
                    if execute {
                        let changed = sqlx::query(next_write).execute(connection).await?;
                        assert!(
                            changed.rows_affected() > 0,
                            "the subsequent writer sees committed data"
                        );
                        ran.store(true, Ordering::SeqCst);
                    }
                    Ok(true)
                })
            })
            .connect_with(options)
            .await
            .unwrap();
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
        // Authentication itself is independent of response snapshot generation;
        // use a genuine service-created identity and retain transaction checks.
        let app = crate::content::admin_routes()
            .merge(crate::home::admin_routes())
            .merge(crate::content::public_routes())
            .layer(axum::Extension(identity))
            .with_state(state);
        armed.store(true, Ordering::SeqCst);
        let response = app
            .oneshot(
                Request::builder()
                    .method(case.method)
                    .uri(case.path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "response-race")
                    .body(Body::from(case.input.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = body_json(response).await;
        // Waiting for this connection also waits for the subsequent writer.
        let connection = pool.acquire().await.unwrap();
        assert!(ran.load(Ordering::SeqCst), "{}", case.name);
        if status != StatusCode::OK
            || body.pointer(case.response_field) != Some(&json!(case.expected))
        {
            failures.push(format!("{}: {status} {body}", case.name));
        }
        drop(connection);
        pool.close().await;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
