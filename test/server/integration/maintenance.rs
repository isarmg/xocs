#![cfg(feature = "maintenance")]
use blog_application::{
    admin_core::{AdministratorService, AdministratorStore, LoginContext},
    admin_sqlite::SqliteAdministratorStore,
};
use std::{fs, os::unix::fs::PermissionsExt};

fn configured_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("create offline maintenance fixture root");
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700))
        .expect("set private offline fixture root permissions");
    fs::create_dir(root.path().join("config"))
        .expect("create offline fixture configuration directory");
    fs::set_permissions(
        root.path().join("config"),
        fs::Permissions::from_mode(0o700),
    )
    .expect("set private offline configuration directory permissions");
    fs::write(
        root.path().join("config/config.json"),
        b"{\"bind\":\"127.0.0.1:18091\",\"development\":true}",
    )
    .expect("write offline fixture configuration");
    fs::set_permissions(
        root.path().join("config/config.json"),
        fs::Permissions::from_mode(0o600),
    )
    .expect("set private offline configuration file permissions");
    root
}

fn context() -> LoginContext {
    LoginContext {
        request_id: None,
        now_micros: 1_800_000_000_000_000,
    }
}

#[tokio::test]
async fn offline_setup_preserves_current_schema_and_revokes_sessions_on_reset() {
    let root = configured_root();
    let original = "InitialPassphrase123";
    blog::maintenance::initialize(root.path(), "admin", original)
        .await
        .unwrap();
    blog::maintenance::check(root.path()).await.unwrap();
    assert!(
        blog::maintenance::initialize(root.path(), "admin", "MustNotReplace123")
            .await
            .is_err()
    );
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(root.path().join("state/site.sqlite"))
                .create_if_missing(false),
        )
        .await
        .unwrap();
    let service = AdministratorService::new(SqliteAdministratorStore::new(pool.clone()));
    let login = service.login("admin", original, &context()).await.unwrap();
    service
        .authenticate_session(&login.session_token, context().now_micros)
        .await
        .unwrap();
    pool.close().await;
    blog::maintenance::reset_password(root.path(), "admin", "ReplacementPassphrase123")
        .await
        .unwrap();
    blog::maintenance::check(root.path()).await.unwrap();
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        root.path().join("state/site.sqlite").display()
    ))
    .await
    .unwrap();
    let service = AdministratorService::new(SqliteAdministratorStore::new(pool.clone()));
    assert!(
        service
            .authenticate_session(&login.session_token, context().now_micros + 1)
            .await
            .is_err()
    );
    assert!(service.login("admin", original, &context()).await.is_err());
    service
        .login("admin", "ReplacementPassphrase123", &context())
        .await
        .unwrap();
    assert_eq!(service.store().administrator_count().await.unwrap(), 1);
    let identity: String = sqlx::query_scalar("SELECT application FROM product_metadata")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(identity, "blog");
    pool.close().await;
}

#[tokio::test]
async fn offline_operations_refuse_running_instance_and_unsafe_configuration() {
    let root = configured_root();
    blog::maintenance::initialize(root.path(), "admin", "InitialPassphrase123")
        .await
        .unwrap();
    let state =
        blog_application::state_file::PrivateStateDirectory::open(root.path().join("state"))
            .unwrap();
    let lock = state.try_instance_lock().unwrap();
    assert!(blog::maintenance::check(root.path()).await.is_err());
    assert!(
        blog::maintenance::reset_password(root.path(), "admin", "ReplacementPassphrase123")
            .await
            .is_err()
    );
    drop(lock);
    blog::maintenance::check(root.path()).await.unwrap();
    fs::set_permissions(
        root.path().join("config/config.json"),
        fs::Permissions::from_mode(0o666),
    )
    .unwrap();
    assert!(blog::maintenance::check(root.path()).await.is_err());
}

#[tokio::test]
async fn offline_root_ignores_runtime_environment_without_touching_other_instance() {
    let selected = configured_root();
    let other = configured_root();
    let fresh = configured_root();
    blog::maintenance::initialize(selected.path(), "admin", "SelectedPassphrase123")
        .await
        .unwrap();
    blog::maintenance::initialize(other.path(), "admin", "OtherPassphrase123")
        .await
        .unwrap();
    let other_state =
        blog_application::state_file::PrivateStateDirectory::open(other.path().join("state"))
            .unwrap();
    let other_lock = other_state.try_instance_lock().unwrap();
    let other_database = other.path().join("state/site.sqlite");
    let before = fs::read(&other_database).unwrap();
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "offline_environment_helper",
            "--ignored",
            "--nocapture",
        ])
        .env("BLOG_OFFLINE_TEST_SELECTED", selected.path())
        .env("BLOG_OFFLINE_TEST_FRESH", fresh.path())
        .env("BLOG_DATA_DIR", other.path().join("state"))
        .env("BLOG_MEDIA", other.path().join("media"))
        .env("BLOG_BIND", "not-a-socket")
        .env("BLOG_LOG_LEVEL", "invalid")
        .env("BLOG_DEVELOPMENT", "not-a-boolean")
        .env("BLOG_DEV_WEB_DIR", "/missing/foreign-web")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fs::read(&other_database).unwrap(),
        before,
        "the other instance changed"
    );
    drop(other_lock);
    for (root, password) in [
        (selected.path(), "SelectedReplacement123"),
        (other.path(), "OtherPassphrase123"),
        (fresh.path(), "FreshPassphrase123"),
    ] {
        blog::maintenance::check(root).await.unwrap();
        let pool = sqlx::SqlitePool::connect(&format!(
            "sqlite://{}",
            root.join("state/site.sqlite").display()
        ))
        .await
        .unwrap();
        let service = AdministratorService::new(SqliteAdministratorStore::new(pool.clone()));
        service.login("admin", password, &context()).await.unwrap();
        pool.close().await;
    }
}

#[test]
#[ignore = "isolated helper invoked by offline_root_ignores_runtime_environment_without_touching_other_instance"]
fn offline_environment_helper() {
    let selected =
        std::path::PathBuf::from(std::env::var_os("BLOG_OFFLINE_TEST_SELECTED").unwrap());
    let fresh = std::path::PathBuf::from(std::env::var_os("BLOG_OFFLINE_TEST_FRESH").unwrap());
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        blog::maintenance::check(&selected).await.unwrap();
        blog::maintenance::reset_password(&selected, "admin", "SelectedReplacement123")
            .await
            .unwrap();
        blog::maintenance::initialize(&fresh, "admin", "FreshPassphrase123")
            .await
            .unwrap();
        blog::maintenance::check(&fresh).await.unwrap();
    });
}

#[tokio::test]
async fn repeated_initialization_preserves_database_without_staging_leaks() {
    let root = configured_root();
    blog::maintenance::initialize(root.path(), "admin", "InitialPassphrase123")
        .await
        .unwrap();
    let database = root.path().join("state/site.sqlite");
    let original = fs::read(&database).unwrap();
    let entries = || {
        let mut names = fs::read_dir(root.path().join("state"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        names.sort();
        names
    };
    let initial_entries = entries();
    for _ in 0..2 {
        let error = blog::maintenance::initialize(root.path(), "admin", "MustNotReplace123")
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            blog::Error::PublishDatabase {
                source: blog_application::fs_safety::Error::DestinationExists(_),
                ..
            }
        ));
        assert_eq!(fs::read(&database).unwrap(), original);
        assert_eq!(entries(), initial_entries);
    }
    blog::maintenance::check(root.path()).await.unwrap();
}

#[tokio::test]
async fn offline_failures_release_the_instance_lock_without_repairing_storage() {
    let root = configured_root();
    blog::maintenance::initialize(root.path(), "admin", "InitialPassphrase123")
        .await
        .unwrap();
    let database = root.path().join("state/site.sqlite");
    let before = fs::read(&database).unwrap();
    fs::rename(root.path().join("media"), root.path().join("media.saved")).unwrap();
    fs::write(root.path().join("media"), b"refuse unsafe media storage").unwrap();
    let directory =
        blog_application::state_file::PrivateStateDirectory::open(root.path().join("state"))
            .unwrap();

    let failures = [
        blog::maintenance::initialize(root.path(), "admin", "MustNotReplace123").await,
        blog::maintenance::check(root.path()).await,
        blog::maintenance::reset_password(root.path(), "admin", "ReplacementPassphrase123").await,
    ];
    for failure in failures {
        assert!(matches!(
            failure,
            Err(blog::Error::Filesystem(
                blog_application::fs_safety::Error::Io(_)
            ))
        ));
        directory.try_instance_lock().unwrap().release().unwrap();
        assert_eq!(fs::read(&database).unwrap(), before);
        assert_eq!(
            fs::read(root.path().join("media")).unwrap(),
            b"refuse unsafe media storage"
        );
    }
}
