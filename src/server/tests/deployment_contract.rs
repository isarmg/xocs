use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
const BINARY: &str = env!("CARGO_BIN_EXE_blog");

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
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            let mut diagnostic = String::new();
            child
                .0
                .stderr
                .take()
                .unwrap()
                .take(65536)
                .read_to_string(&mut diagnostic)
                .unwrap();
            panic!("runtime exited before ready: {status}: {diagnostic}");
        }
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

#[test]
fn runtime_never_initializes_state_and_has_no_offline_commands() {
    let root = fixture(available_address());
    let config = root.path().join("config/config.json");
    let original = fs::read(&config).unwrap();
    let rejected = Command::new(BINARY)
        .env_clear()
        .arg("--root")
        .arg(root.path())
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    for command in [
        "init",
        "check",
        "config",
        "admin",
        "reset-password",
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
                .arg(command)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    assert!(!root.path().join("state").exists());
    assert!(!root.path().join("run").exists());
    assert!(!root.path().join("logs").exists());
    assert_eq!(fs::read(&config).unwrap(), original);
    fs::write(&config, br#"{"trusted_proxies":["127.0.0.1"]}"#).unwrap();
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
    assert!(!root.path().join("state").exists());
}

#[path = "support/database.rs"]
mod database;

#[tokio::test]
async fn current_installed_state_starts_and_foreign_state_rejects_without_writes() {
    let address = available_address();
    let root = fixture(address);
    let database = database::initialize(root.path(), "admin", "TemporaryPassphrase123")
        .await
        .unwrap();
    let running = start(root.path(), address);
    let duplicate = Command::new(BINARY)
        .env_clear()
        .arg("--root")
        .arg(root.path())
        .output()
        .unwrap();
    assert!(
        !duplicate.status.success(),
        "accepted concurrent runtime for the same physical state"
    );
    drop(running);
    let restarted = start(root.path(), address);
    drop(restarted);
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
        assert_eq!(fs::read(&database).unwrap(), bytes);
        assert_eq!(fs::read_dir(root.path().join("run")).unwrap().count(), 0);
    }
}
