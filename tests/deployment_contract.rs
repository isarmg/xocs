use std::{
    fs,
    io::Write,
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};
const BINARY: &str = env!("CARGO_BIN_EXE_xocs");
fn command(config: &std::path::Path, args: &[&str], password: Option<&str>) -> Output {
    let mut child = Command::new(BINARY)
        .env_clear()
        .args(["--config"])
        .arg(config)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(password) = password {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(password.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}
struct Running(std::process::Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn start(binary: &std::path::Path, config: &std::path::Path) -> Running {
    let mut running = Running(
        Command::new(binary)
            .env_clear()
            .arg("--config")
            .arg(config)
            .arg("run")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let until = Instant::now() + Duration::from_secs(15);
    loop {
        if command(config, &["status", "--json"], None)
            .status
            .success()
        {
            break;
        }
        assert!(running.0.try_wait().unwrap().is_none(), "server exited");
        assert!(Instant::now() < until, "server never became ready");
        std::thread::sleep(Duration::from_millis(100));
    }
    running
}

#[test]
fn explicit_initialization_and_relocated_binary_need_no_release_tree() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let config = root.path().join("config.json");
    let data = root.path().join("state %#?");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let bind = listener.local_addr().unwrap();
    drop(listener);
    fs::write(
        &config,
        serde_json::to_vec(
            &serde_json::json!({"data_dir":data,"bind":bind.to_string(),"development":true}),
        )
        .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(!command(&config, &["status"], None).status.success());
    assert!(!command(&config, &["run"], None).status.success());
    assert!(!data.exists());
    xcss::testkit::assert_deployment_cli_contract(
        std::path::Path::new(BINARY),
        &config,
        &data,
        &["site.sqlite"],
    );
    assert!(!command(&config, &["doctor"], None).status.success());
    assert!(
        !command(
            &config,
            &["init", "--database", "/tmp/forbidden.sqlite"],
            None
        )
        .status
        .success()
    );
    let binary = root.path().join("xocs");
    fs::copy(BINARY, &binary).unwrap();
    let mut running = start(&binary, &config);
    assert!(
        !command(
            &config,
            &["admin", "reset-password", "--username", "admin"],
            Some("ReplacementPassphrase123\n")
        )
        .status
        .success(),
        "offline password reset must reject a live owner"
    );
    running.0.kill().unwrap();
    running.0.wait().unwrap();
    let reset = command(
        &config,
        &["admin", "reset-password", "--username", "admin", "--json"],
        Some("ReplacementPassphrase123\n"),
    );
    assert!(
        reset.status.success(),
        "{}",
        String::from_utf8_lossy(&reset.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&reset.stdout).unwrap()["sessions_revoked"],
        true
    );
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let options = sqlx::sqlite::SqliteConnectOptions::new().filename(data.join("site.sqlite")).read_only(true);
        let pool = sqlx::SqlitePool::connect_with(options).await.unwrap();
        let (hash, version): (String, i64) = sqlx::query_as("SELECT password_hash, session_version FROM _common_administrators WHERE username='admin'").fetch_one(&pool).await.unwrap();
        assert!(xcss::admin_auth::verify_password("ReplacementPassphrase123", &hash));
        assert!(!xcss::admin_auth::verify_password("correct horse battery", &hash));
        assert_eq!(version, 2);
        pool.close().await;
    });
    let _restarted = start(&binary, &config);
    assert!(
        !command(
            &root.path().join("missing.json"),
            &["config", "validate"],
            None
        )
        .status
        .success()
    );
}

fn private_config(root: &std::path::Path, value: &serde_json::Value) -> std::path::PathBuf {
    fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
    let config = root.join("config.json");
    fs::write(&config, serde_json::to_vec(value).unwrap()).unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    config
}

#[test]
fn configuration_layers_and_internal_paths_have_one_contract() {
    let root = tempfile::tempdir().unwrap();
    let config = private_config(
        root.path(),
        &serde_json::json!({
            "data_dir": root.path().join("state"), "log_level": "warn"
        }),
    );
    let output = Command::new(BINARY)
        .env_clear()
        .env("XOCS_LOG_LEVEL", "error")
        .env("RUST_LOG", "not-a-valid-directive=")
        .env("XOCS_DATABASE", "/this/must/be/ignored.sqlite")
        .arg("--config")
        .arg(&config)
        .args(["config", "validate", "--log-level", "debug", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "valid");
    assert!(!root.path().join("state").exists());
    // Invalid lower-priority values cannot be hidden by valid overrides.
    let output = Command::new(BINARY)
        .env_clear()
        .env("XOCS_LOG_LEVEL", "invalid")
        .arg("--config")
        .arg(&config)
        .args(["config", "validate", "--log-level", "info"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    for bad in [
        serde_json::json!({"database":"/tmp/custom.sqlite"}),
        serde_json::json!({"data_dir":null}),
        serde_json::json!({"log_level":"info,xocs=trace"}),
        serde_json::json!({"data_dir":"/tmp/private-state","media":"/tmp/private-state/logs"}),
        serde_json::json!({"data_dir":"/tmp/private-state","media":"/tmp/private-state/site.sqlite-wal"}),
        serde_json::json!({"data_dir":"/tmp/private-state","media":"/tmp/private-state/.state-instance.lock"}),
    ] {
        fs::write(&config, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(
            !command(&config, &["config", "validate"], None)
                .status
                .success()
        );
    }
}

#[test]
fn foreground_uses_utc_structured_logs_and_ignores_rust_log() {
    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("state");
    let config = private_config(
        root.path(),
        &serde_json::json!({"data_dir":data,"log_level":"info"}),
    );
    let mut child = Command::new(BINARY)
        .env_clear()
        .env("RUST_LOG", "off")
        .arg("--config")
        .arg(&config)
        .arg("init")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"TemporaryPassphrase123\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fs::read_to_string(data.join("logs/server.jsonl")).unwrap();
    let record: serde_json::Value = serde_json::from_str(log.lines().last().unwrap()).unwrap();
    assert_eq!(record["event"], "common.initialization.completed");
    assert_eq!(record["level"], "INFO");
    assert!(record["timestamp"].as_str().unwrap().ends_with('Z'));
    assert!(
        output.stderr.is_empty(),
        "persistent logging must not duplicate its output"
    );
    assert!(!log.contains("TemporaryPassphrase123"));
}

#[test]
fn installer_setup_and_upgrade_preserve_configuration_database_media_and_logs() {
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let state = root.path().join("var/lib/isarmg/xocs");
    let media = root.path().join("srv/blog media");
    let config = root.path().join("etc/isarmg/xocs/config.json");
    let selected = private_config(
        root.path(),
        &serde_json::json!({
            "data_dir":"/var/lib/isarmg/xocs", "media":"/srv/blog media"
        }),
    );
    let original_config = fs::read(&selected).unwrap();
    let package = root.path().join("package");
    for path in [package.join("bin"), package.join("systemd")] {
        fs::create_dir_all(path).unwrap();
    }
    fs::copy(BINARY, package.join("bin/xocs")).unwrap();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("deploy/linux-x86_64");
    fs::copy(source.join("install.sh"), package.join("install.sh")).unwrap();
    fs::copy(
        source.join("xocs.service"),
        package.join("systemd/xocs.service"),
    )
    .unwrap();
    // The installer test mode roots filesystem paths and simulates only service
    // and account operations. Initialization and checks use the real binary.
    let invoke = |mode: &str| {
        let mut process = Command::new("/bin/bash");
        process
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("XOCS_INSTALL_ROOT", root.path())
            .env("XOCS_INSTALL_TEST", "1")
            // Caller environment must never override the selected JSON.
            .env("XOCS_DATA_DIR", "/this/must/not/be/created")
            .arg(package.join("install.sh"))
            .arg(mode)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if mode == "setup" {
            process.arg(&selected).arg("admin");
        }
        let mut child = process.spawn().unwrap();
        if mode == "setup" {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(b"TemporaryPassphrase123\n")
                .unwrap();
        }
        child.wait_with_output().unwrap()
    };
    let installed = invoke("setup");
    assert!(
        installed.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&installed.stdout),
        String::from_utf8_lossy(&installed.stderr)
    );
    assert_eq!(fs::read(&config).unwrap(), original_config);
    assert_eq!(
        fs::read_to_string(root.path().join("service-actions")).unwrap(),
        "daemon-reload\n"
    );
    assert!(state.join("site.sqlite").is_file());
    assert!(media.is_dir());
    assert!(!state.join("media").exists());
    let access = fs::read_to_string(
        root.path()
            .join("etc/systemd/system/xocs.service.d/state.conf"),
    )
    .unwrap();
    assert!(access.contains("ReadWritePaths=\"/var/lib/isarmg/xocs\""));
    assert!(access.contains("ReadWritePaths=\"/srv/blog media\""));
    fs::write(media.join("keep-original.png"), b"business media bytes").unwrap();
    let preserved = [
        config,
        state.join("site.sqlite"),
        media.join("keep-original.png"),
        state.join("logs/server.jsonl"),
    ];
    let before: Vec<_> = preserved
        .iter()
        .map(|path| fs::read(path).unwrap())
        .collect();
    // A repeated setup fails before touching initialized state.
    assert!(!invoke("setup").status.success());
    for field in ["data_dir", "media"] {
        for forbidden in [
            "/opt/isarmg/xocs/bin",
            "/etc/isarmg/xocs",
            "/etc/systemd/system",
            "/run/isarmg/xocs",
            "//opt/isarmg/xocs",
        ] {
            let mut value =
                serde_json::json!({"data_dir":"/var/lib/isarmg/xocs","media":"/srv/blog media"});
            value[field] = forbidden.into();
            fs::write(&selected, serde_json::to_vec(&value).unwrap()).unwrap();
            let rejected = invoke("setup");
            assert!(!rejected.status.success());
            assert!(String::from_utf8_lossy(&rejected.stderr).contains("must not overlap"));
        }
    }
    fs::write(&selected, original_config).unwrap();
    let stopped_upgrade = invoke("upgrade");
    assert!(
        stopped_upgrade.status.success(),
        "{}",
        String::from_utf8_lossy(&stopped_upgrade.stderr)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("service-actions")).unwrap(),
        "daemon-reload\nis-active --quiet xocs.service\ndaemon-reload\n"
    );
    fs::write(root.path().join("service-actions"), b"").unwrap();
    fs::write(root.path().join("service-active"), b"active").unwrap();
    let active_upgrade = invoke("upgrade");
    assert!(
        active_upgrade.status.success(),
        "{}",
        String::from_utf8_lossy(&active_upgrade.stderr)
    );
    for (path, bytes) in preserved.iter().zip(before) {
        assert_eq!(fs::read(path).unwrap(), bytes, "changed {}", path.display());
    }
    assert_eq!(
        fs::read(root.path().join("opt/isarmg/xocs/bin/xocs")).unwrap(),
        fs::read(BINARY).unwrap()
    );
    assert_eq!(
        fs::read_to_string(root.path().join("service-actions")).unwrap(),
        "is-active --quiet xocs.service\nstop xocs.service\ndaemon-reload\nstart xocs.service\n"
    );
}
