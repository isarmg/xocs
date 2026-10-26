use std::{
    fs,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
const BINARY: &str = env!("CARGO_BIN_EXE_blog");

fn fixture(bind: SocketAddr) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("create deployment fixture root");
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755))
        .expect("set deployment fixture root permissions");
    fs::create_dir(root.path().join("config")).expect("create fixture configuration directory");
    fs::set_permissions(
        root.path().join("config"),
        fs::Permissions::from_mode(0o700),
    )
    .expect("set private fixture configuration directory permissions");
    let config = root.path().join("config/config.json");
    fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({"bind":bind.to_string(),"development":true}))
            .expect("invariant: fixture configuration serializes to JSON"),
    )
    .expect("write fixture configuration");
    fs::set_permissions(config, fs::Permissions::from_mode(0o600))
        .expect("set private fixture configuration file permissions");
    root
}
fn available_address() -> SocketAddr {
    TcpListener::bind("127.0.0.1:0")
        .expect("reserve a loopback test address")
        .local_addr()
        .expect("read the reserved loopback address")
}
struct Running(std::process::Child);
impl Running {
    fn stop(mut self) -> io::Result<()> {
        if self.0.try_wait()?.is_none() {
            self.0.kill()?;
        }
        self.0.wait()?;
        Ok(())
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        match self.0.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(error) => eprintln!("inspect fixture process during cleanup: {error}"),
        }
        // Assertion failures also need cleanup; Drop reports failures without a second panic.
        if let Err(kill) = self.0.kill() {
            match self.0.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => eprintln!("kill fixture process during cleanup: {kill}"),
                Err(wait) => {
                    eprintln!("kill fixture process during cleanup: {kill}; inspect exit: {wait}");
                }
            }
            return;
        }
        if let Err(error) = self.0.wait() {
            eprintln!("reap fixture process during cleanup: {error}");
        }
    }
}

fn probe_readiness(address: SocketAddr) -> io::Result<bool> {
    let mut socket = TcpStream::connect_timeout(&address, Duration::from_millis(200))?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;
    socket.set_write_timeout(Some(Duration::from_millis(500)))?;
    socket.write_all(
        format!("GET /readyz HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n").as_bytes(),
    )?;
    let mut response = String::new();
    socket.take(65_536).read_to_string(&mut response)?;
    Ok(response.starts_with("HTTP/1.1 200"))
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
            .expect("spawn the compiled server for the deployment fixture"),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.0.try_wait().expect("inspect fixture server exit") {
            let mut diagnostic = String::new();
            child
                .0
                .stderr
                .take()
                .expect("invariant: fixture server stderr was piped")
                .take(65536)
                .read_to_string(&mut diagnostic)
                .expect("read bounded fixture server startup diagnostics");
            panic!("runtime exited before ready: {status}: {diagnostic}");
        }
        match probe_readiness(address) {
            Ok(true) => return child,
            Ok(false) => {
                assert!(Instant::now() < deadline, "runtime readiness timeout");
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::ConnectionAborted
                        | io::ErrorKind::BrokenPipe
                        | io::ErrorKind::UnexpectedEof
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                ) =>
            {
                assert!(
                    Instant::now() < deadline,
                    "runtime readiness timeout: {error}"
                );
            }
            Err(error) => panic!("fixture readiness request failed: {error}"),
        }
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
    running.stop().expect("stop the first fixture server");
    let restarted = start(root.path(), address);
    restarted.stop().expect("stop the restarted fixture server");
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
