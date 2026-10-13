use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
const BINARY: &str = env!("CARGO_BIN_EXE_xocs");

fn fixture(bind: SocketAddr) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755)).unwrap();
    fs::create_dir(root.path().join("config")).unwrap();
    fs::set_permissions(
        root.path().join("config"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    let config = root.path().join("config/config.json");
    fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({"bind":bind.to_string(),"development":true}))
            .unwrap(),
    )
    .unwrap();
    fs::set_permissions(config, fs::Permissions::from_mode(0o600)).unwrap();
    root
}
fn available_address() -> SocketAddr {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}
struct Running(std::process::Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn start(root: &Path, address: SocketAddr) -> Running {
    let mut child = Running(
        Command::new(BINARY)
            .env_clear()
            .arg("--root")
            .arg(root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "runtime exited before ready"
        );
        if let Ok(mut socket) = TcpStream::connect_timeout(&address, Duration::from_millis(200)) {
            socket
                .set_read_timeout(Some(Duration::from_millis(500)))
                .unwrap();
            socket
                .write_all(
                    format!("GET /readyz HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n")
                        .as_bytes(),
                )
                .unwrap();
            let mut response = String::new();
            let _ = socket.read_to_string(&mut response);
            if response.starts_with("HTTP/1.1 200") {
                return child;
            }
        }
        assert!(Instant::now() < deadline, "runtime readiness timeout");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[tokio::test]
async fn offline_contract_is_pure_and_runtime_never_initializes_state() {
    let root = fixture(available_address());
    let config = root.path().join("config/config.json");
    let report = xocs::offline::validate_config(root.path(), &config)
        .await
        .unwrap();
    assert_eq!(report.data_dir, root.path().join("state"));
    assert!(!report.data_dir.exists());
    assert!(
        !Command::new(BINARY)
            .env_clear()
            .arg("--root")
            .arg(root.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!report.data_dir.exists());
    for old in [
        "init",
        "check",
        "config",
        "admin",
        "status",
        "web-assets",
        "release-identity",
        "state-contract",
    ] {
        assert!(
            !Command::new(BINARY)
                .env_clear()
                .arg("--root")
                .arg(root.path())
                .arg(old)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    assert!(
        xocs::offline::initialize(root.path(), &config, "admin", "short")
            .await
            .is_err()
    );
    assert!(!report.data_dir.exists());
    fs::write(&config, br#"{"trusted_proxies":["127.0.0.1"]}"#).unwrap();
    assert!(
        xocs::offline::validate_config(root.path(), &config)
            .await
            .is_err()
    );
    assert!(!report.data_dir.exists());
}

#[tokio::test]
async fn initialization_password_reset_and_runtime_share_one_database() {
    let address = available_address();
    let root = fixture(address);
    let config = root.path().join("config/config.json");
    xocs::offline::initialize(root.path(), &config, "admin", "TemporaryPassphrase123")
        .await
        .unwrap();
    let before = xocs::offline::check(root.path(), &config).await.unwrap();
    assert!(before.initialized);
    assert!(
        xocs::offline::initialize(root.path(), &config, "admin", "OverwritePassphrase123")
            .await
            .is_err()
    );
    let schema = before.config.schema_identity.clone();
    xocs::offline::migrate(
        root.path(),
        &config,
        &before.config.data_dir,
        &schema,
        &schema,
    )
    .await
    .unwrap();
    let mut wrong = schema.clone();
    wrong.schema_revision += 1;
    assert!(
        xocs::offline::migrate(
            root.path(),
            &config,
            &before.config.data_dir,
            &schema,
            &wrong
        )
        .await
        .is_err()
    );
    let running = start(root.path(), address);
    assert!(
        xocs::offline::reset_password(root.path(), &config, "admin", "ReplacementPassphrase123")
            .await
            .is_err()
    );
    drop(running);
    xocs::offline::reset_password(root.path(), &config, "admin", "ReplacementPassphrase123")
        .await
        .unwrap();
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(before.config.data_dir.join("site.sqlite"))
        .read_only(true);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    let (hash, version): (String, i64) = sqlx::query_as(
        "SELECT password_hash,session_version FROM _common_administrators WHERE username='admin'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(xcss::admin_auth::verify_password(
        "ReplacementPassphrase123",
        &hash
    ));
    assert_eq!(version, 2);
    pool.close().await;
    xocs::offline::check(root.path(), &config).await.unwrap();
    let restarted = start(root.path(), address);
    drop(restarted);

    let database = before.config.data_dir.join("site.sqlite");
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&database)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Delete);
    let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
    sqlx::query("UPDATE _common_administrators SET password_hash='invalid-hash'")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;
    let bytes = fs::read(&database).unwrap();
    for _ in 0..3 {
        assert!(xocs::offline::check(root.path(), &config).await.is_err());
        assert_eq!(fs::read(&database).unwrap(), bytes);
        assert_eq!(fs::read_dir(root.path().join("run")).unwrap().count(), 0);
    }
}
