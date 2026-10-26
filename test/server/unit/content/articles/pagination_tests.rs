use super::*;
use sqlx::Row;

#[tokio::test]
async fn article_page_optimization_preserves_filters_rank_and_summaries() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../../../../../src/server/schema/product.sql"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql("INSERT INTO user(id,username) VALUES(1,'author'); INSERT INTO sort(id,sort_name) VALUES(1,'one'),(2,'two'); INSERT INTO label(id,sort_id,label_name) VALUES(1,1,'one'),(2,2,'two');").execute(&pool).await.unwrap();
    for id in 1..=24_i64 {
        sqlx::query("INSERT INTO article(id,user_id,sort_id,label_id,article_title,article_content,view_status,password,recommend_status,deleted,create_time) VALUES(?,1,?,?,?,?,?,?,?,?,?)")
                .bind(id).bind(1 + id % 2).bind(1 + id % 2)
                .bind(if id % 3 == 0 { "needle title" } else { "other title" })
                .bind(if id % 2 == 0 { "needle body %" } else { "plain body" })
                .bind(if id == 24 { 2_i64 } else if id == 20 { -1_i64 } else { i64::from(id % 4 >= 2) })
                .bind(if id % 4 == 0 { Some("protected") } else { None })
                .bind(id % 2).bind(i64::from(id % 7 == 0))
                .bind(if id == 3 { None } else { Some("2026-10-12 12:00:00") })
                .execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO comment(source,type,comment_content) VALUES(6,'article','reply'),(6,'article','reply two')").execute(&pool).await.unwrap();
    for audience in [ArticleAudience::Administrator, ArticleAudience::Public] {
        let public = matches!(audience, ArticleAudience::Public);
        for search in ["", "needle", "n", "%"] {
            for (sort_id, label_id, recommended) in [
                (None, None, None),
                (Some(1), None, None),
                (None, Some(2), None),
                (None, None, Some(true)),
                (Some(2), Some(2), Some(false)),
            ] {
                for page in [1, 2] {
                    let pattern = format!(
                        "%{}%",
                        search
                            .replace('\\', "\\\\")
                            .replace('%', "\\%")
                            .replace('_', "\\_")
                    );
                    let phrase = format!("\"{search}\"");
                    let predicate = if public {
                        "deleted=0 AND (view_status=1 OR password IS NOT NULL)"
                    } else {
                        "deleted=0"
                    };
                    let body_match = if public {
                        "(password IS NULL AND article_content LIKE ?3 ESCAPE '\\')"
                    } else {
                        "article_content LIKE ?3 ESCAPE '\\'"
                    };
                    let filter = format!(
                        "{predicate} AND (?1 IS NULL OR sort_id=?1) AND (?6 IS NULL OR label_id=?6) AND (?4 IS NULL OR recommend_status=?4) AND (?2='' OR ((article_title LIKE ?3 ESCAPE '\\' OR {body_match}) AND (length(?2)<3 OR id IN (SELECT rowid FROM article_search WHERE article_search MATCH ?5))))"
                    );
                    let expected_total: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                        "SELECT COUNT(*) FROM article WHERE {filter}"
                    )))
                    .bind(sort_id)
                    .bind(search)
                    .bind(&pattern)
                    .bind(recommended.map(i64::from))
                    .bind(&phrase)
                    .bind(label_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
                    let expected = sqlx::query_as::<_,ArticleSummary>(sqlx::AssertSqlSafe(format!("{ARTICLE_SUMMARY_SQL} WHERE {filter} ORDER BY CASE WHEN ?2<>'' AND article_title LIKE ?3 ESCAPE '\\' THEN 0 ELSE 1 END,create_time DESC,id DESC LIMIT ?7 OFFSET ?8")))
                            .bind(sort_id).bind(search).bind(&pattern).bind(recommended.map(i64::from)).bind(&phrase).bind(label_id).bind(5).bind((page-1)*5)
                            .fetch_all(&pool).await.unwrap();
                    let actual = list_articles(
                        &pool,
                        ArticleQuery {
                            page: Some(page),
                            size: Some(5),
                            sort_id,
                            label_id,
                            search: Some(search.into()),
                            recommended,
                        },
                        audience,
                    )
                    .await
                    .unwrap();
                    assert_eq!(actual.total, expected_total);
                    assert_eq!(
                        serde_json::to_value(actual.items).unwrap(),
                        serde_json::to_value(expected).unwrap(),
                        "public={public}, search={search}, page={page}"
                    );
                }
            }
        }
    }
    let filters = "?1 IS NULL AND ?6 IS NULL AND ?4 IS NULL";
    let (_, list) = article_page_sql("", filters, ArticleAudience::Public, SearchMode::Browse);
    let plans = sqlx::query(sqlx::AssertSqlSafe(format!("EXPLAIN QUERY PLAN {list}")))
        .bind(None::<i64>)
        .bind("")
        .bind("%%")
        .bind(None::<i64>)
        .bind("\"\"")
        .bind(None::<i64>)
        .bind(5)
        .bind(0)
        .fetch_all(&pool)
        .await
        .unwrap();
    let details: Vec<String> = plans.iter().map(|row| row.get(3)).collect();
    assert!(
        details
            .iter()
            .any(|detail| detail.contains("MATERIALIZE article_page")),
        "{details:?}"
    );
    assert!(
        details
            .iter()
            .any(|detail| detail.contains("article_public_latest (deleted=? AND view_status=?)")),
        "{details:?}"
    );
    assert!(
        details
            .iter()
            .any(|detail| detail.contains("article USING INTEGER PRIMARY KEY (rowid=?)")),
        "{details:?}"
    );
}
