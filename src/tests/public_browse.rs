use super::*;

async fn articles(app: &Router, query: &str) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/articles?{query}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "{query}");
    body_json(response).await
}

#[tokio::test]
async fn public_browse_filters_count_and_page_in_the_same_stable_order() {
    let (_dir, app, pool) = setup().await;
    for id in 1..=28 {
        sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,recommend_status,create_time) VALUES(?,1,1,?,?,'A shared story',?,'2026-10-12 00:00:00')")
            .bind(id).bind(if id <= 26 { 1 } else { 2 })
            .bind(format!("Story {id}")).bind(i64::from(id % 2 == 0))
            .execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,create_time) VALUES(29,1,2,3,'Different category','A shared story','2026-10-12 00:00:00')")
        .execute(&pool).await.unwrap();
    sqlx::query("UPDATE article SET deleted=1 WHERE id=28")
        .execute(&pool)
        .await
        .unwrap();
    let filter = "sort_id=1&label_id=1&recommended=true&search=story&size=12";
    let first = articles(&app, &format!("{filter}&page=1")).await;
    let second = articles(&app, &format!("{filter}&page=2")).await;
    assert_eq!(first["total"], 13);
    assert_eq!(second["total"], 13);
    assert_eq!(first["page"], 1);
    assert_eq!(second["page"], 2);
    assert_eq!(first["size"], 12);
    assert_eq!(second["size"], 12);
    assert_eq!(first["items"].as_array().unwrap().len(), 12);
    assert_eq!(second["items"].as_array().unwrap().len(), 1);
    let ids: Vec<_> = first["items"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second["items"].as_array().unwrap())
        .map(|item| item["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, (1..=13).rev().map(|n| n * 2).collect::<Vec<_>>());
    assert_eq!(articles(&app, "sort_id=1&label_id=2").await["total"], 1);
    assert_eq!(articles(&app, "sort_id=2&label_id=1").await["total"], 0);
    assert_eq!(articles(&app, "sort_id=2").await["items"][0]["id"], 29);
    assert!(
        articles(&app, &format!("{filter}&page=3")).await["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn public_search_treats_punctuation_as_literal_and_keeps_utf8_terms() {
    let (_dir, app, pool) = setup().await;
    sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content) VALUES(1,1,1,1,?,?),(2,1,1,1,'1000 open docs','ordinary content')")
        .bind("100% _draft_ C:\\notes \"quoted\" 春日记")
        .bind("a body with 100% and _draft_ and 春日记")
        .execute(&pool).await.unwrap();
    for term in [
        "100%",
        "_draft_",
        "C:\\notes",
        "\"quoted\"",
        "春日记",
        "春日",
    ] {
        let encoded =
            percent_encoding::utf8_percent_encode(term, percent_encoding::NON_ALPHANUMERIC);
        let page = articles(&app, &format!("search={encoded}")).await;
        assert_eq!(page["total"], 1, "{term}");
        assert_eq!(page["items"][0]["id"], 1, "{term}");
    }
}
