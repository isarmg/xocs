use blog_application::admin_sqlite::{ADMIN_PERSISTENT_DDL, SqliteAdministratorStore};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

async fn fixture() -> sqlx::Result<(SqliteAdministratorStore, SqlitePool)> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    sqlx::raw_sql(ADMIN_PERSISTENT_DDL).execute(&pool).await?;
    sqlx::query("INSERT INTO _common_administrators(administrator_id,username,password_hash,created_at_micros,updated_at_micros) VALUES('admin','admin','unused-fixture-hash',0,0)")
        .execute(&pool).await?;
    Ok((SqliteAdministratorStore::new(pool.clone()), pool))
}

async fn session(pool: &SqlitePool, id: i64, active: bool) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO _common_admin_sessions(rowid,session_id,administrator_id,token_hash,csrf_hash,administrator_session_version,created_at_micros,last_seen_at_micros,idle_expires_at_micros,absolute_expires_at_micros) VALUES(?,?,'admin',?,zeroblob(32),1,0,0,?,1000)")
        .bind(id).bind(id.to_string()).bind(format!("{id:032}").into_bytes())
        .bind(if active { 900_i64 } else { 10_i64 }).execute(pool).await?;
    Ok(())
}

async fn ids(pool: &SqlitePool) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar("SELECT rowid FROM _common_admin_sessions ORDER BY rowid")
        .fetch_all(pool)
        .await
}

#[tokio::test]
async fn bounded_passes_advance_beyond_live_rows_and_revisit_after_wrap()
-> Result<(), Box<dyn std::error::Error>> {
    let (store, pool) = fixture().await?;
    for id in 1..=5 {
        session(&pool, id, id <= 2).await?;
    }
    assert_eq!(store.prune_inactive_sessions(100, 2).await?, 0);
    assert_eq!(store.prune_inactive_sessions(100, 2).await?, 2);
    assert_eq!(store.prune_inactive_sessions(100, 2).await?, 1);
    assert_eq!(ids(&pool).await?, [1, 2]);
    assert_eq!(store.prune_inactive_sessions(100, 2).await?, 0);
    assert_eq!(store.prune_inactive_sessions(950, 2).await?, 2);
    assert!(ids(&pool).await?.is_empty());
    assert_eq!(store.prune_inactive_sessions(1000, 2).await?, 0);
    assert!(store.prune_inactive_sessions(1000, 0).await.is_err());
    assert!(store.prune_inactive_sessions(1000, 129).await.is_err());
    Ok(())
}

#[tokio::test]
async fn clones_serialize_progress_and_scan_survives_deleted_and_reused_rowids()
-> Result<(), Box<dyn std::error::Error>> {
    let (store, pool) = fixture().await?;
    for id in 1..=4 {
        session(&pool, id, id <= 2).await?;
    }
    let clone = store.clone();
    let (first, second) = tokio::join!(
        store.prune_inactive_sessions(100, 2),
        clone.prune_inactive_sessions(100, 2),
    );
    assert_eq!(first? + second?, 2);
    assert_eq!(ids(&pool).await?, [1, 2]);
    sqlx::query("DELETE FROM _common_admin_sessions WHERE rowid=2")
        .execute(&pool)
        .await?;
    session(&pool, 2, false).await?;
    session(&pool, i64::MAX, false).await?;
    assert_eq!(store.prune_inactive_sessions(100, 1).await?, 1);
    assert_eq!(clone.prune_inactive_sessions(100, 1).await?, 0);
    assert_eq!(store.prune_inactive_sessions(100, 1).await?, 1);
    assert_eq!(ids(&pool).await?, [1]);
    Ok(())
}

#[tokio::test]
async fn cleanup_keyset_uses_the_rowid_index_without_sorting() -> sqlx::Result<()> {
    let (_, pool) = fixture().await?;
    let plans: Vec<(i64, i64, i64, String)> = sqlx::query_as(
        "EXPLAIN QUERY PLAN SELECT rowid FROM _common_admin_sessions WHERE rowid>? ORDER BY rowid LIMIT ?",
    ).bind(5_i64).bind(128_i64).fetch_all(&pool).await?;
    assert!(
        plans
            .iter()
            .any(|(_, _, _, detail)| detail.contains("SEARCH") && detail.contains("rowid>")),
        "{plans:?}"
    );
    assert!(
        plans
            .iter()
            .all(|(_, _, _, detail)| !detail.contains("TEMP B-TREE")),
        "{plans:?}"
    );
    Ok(())
}
