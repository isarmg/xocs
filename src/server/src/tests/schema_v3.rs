use super::*;

#[tokio::test]
async fn genuine_v3_schema_is_rejected_without_changing_any_database_bytes() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let database = directory.path().join("site.sqlite");
    let pool = open_database(&database, true).await.unwrap();
    // Frozen product DDL from b10064e8b15fc44698feb10bab7d810b3d0179a4.
    let ddl = format!(
        "{};\n{}\n{}",
        blog_application::schema_identity::PRODUCT_METADATA_DDL,
        blog_application::admin_sqlite::ADMIN_PERSISTENT_DDL,
        include_str!("../../tests/fixtures/schema-v3.sql")
    );
    let fingerprint = blog_application::sqlite::fingerprint_trusted_ddl(&[&ddl]).unwrap();
    sqlx::raw_sql(sqlx::AssertSqlSafe(ddl))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO product_metadata(singleton,application,application_version,schema_revision,schema_sha256) VALUES(1,'blog','blog-db-v3',3,?)")
        .bind(fingerprint).execute(&pool).await.unwrap();
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
