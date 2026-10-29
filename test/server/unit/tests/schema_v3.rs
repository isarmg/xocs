use super::*;

#[tokio::test]
async fn genuine_v3_schema_is_rejected_without_changing_any_database_bytes() {
    // Frozen product DDL from b10064e8b15fc44698feb10bab7d810b3d0179a4.
    old_schema_is_rejected(
        include_str!("../../integration/fixtures/schema-v3.sql"),
        "blog-db-v3",
        3,
    )
    .await;
}

#[tokio::test]
async fn genuine_v4_schema_is_rejected_without_changing_any_database_bytes() {
    old_schema_is_rejected(
        include_str!("../../integration/fixtures/schema-v4.sql"),
        "blog-db-v4",
        4,
    )
    .await;
}

async fn old_schema_is_rejected(product_ddl: &str, version: &str, revision: i64) {
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let database = directory.path().join("site.sqlite");
    create_empty_database(&database);
    let pool = open_database(&database).await.unwrap();
    let ddl = format!(
        "{};\n{}\n{}",
        blog_application::schema_identity::PRODUCT_METADATA_DDL,
        blog_application::admin_sqlite::ADMIN_PERSISTENT_DDL,
        product_ddl
    );
    let fingerprint = blog_application::sqlite::fingerprint_trusted_ddl(&[&ddl]).unwrap();
    sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO product_metadata(singleton,application,application_version,schema_revision,schema_sha256) VALUES(1,'blog',?,?,?)")
        .bind(version).bind(revision).bind(fingerprint).execute(&pool).await.unwrap();
    close_fixture_pool(&pool).await;
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
async fn future_identity_is_rejected_even_when_its_ddl_matches_current() {
    let (directory, _app, pool) = setup().await;
    sqlx::query("UPDATE product_metadata SET application_version='blog-db-v6',schema_revision=6")
        .execute(&pool)
        .await
        .unwrap();
    close_fixture_pool(&pool).await;
    let database = directory.path().join("site.sqlite");
    let before = std::fs::read(&database).unwrap();
    assert!(
        validate_existing_database(&database, database.parent().unwrap())
            .await
            .is_err()
    );
    assert_eq!(std::fs::read(database).unwrap(), before);
}
