//! Interrupt the actual CLI, including its separately owned Git process group.
use rustix::process::{Pid, Signal, kill_process};
use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

struct FixtureProcess {
    pid: Pid,
    started: String,
}

impl FixtureProcess {
    fn identify(path: &Path) -> Self {
        let pid: i32 = fs::read_to_string(path).unwrap().trim().parse().unwrap();
        let pid = Pid::from_raw(pid).unwrap();
        let (_, started) = Self::state(pid).expect("Fixture process is running");
        Self { pid, started }
    }

    fn state(pid: Pid) -> Option<(char, String)> {
        let stat = fs::read_to_string(format!("/proc/{}/stat", pid.as_raw_nonzero())).ok()?;
        let fields = stat
            .rsplit_once(") ")?
            .1
            .split_whitespace()
            .collect::<Vec<_>>();
        Some((fields[0].chars().next()?, fields.get(19)?.to_string()))
    }

    fn running(&self) -> bool {
        Self::state(self.pid)
            .is_some_and(|(state, started)| state != 'Z' && started == self.started)
    }
}

impl Drop for FixtureProcess {
    fn drop(&mut self) {
        if self.running() {
            let _ = kill_process(self.pid, Signal::KILL);
        }
    }
}

struct Cli(Child);

impl Cli {
    fn wait(&mut self) -> ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "CLI did not finish interruption cleanup"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Cli {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for_file(path: PathBuf) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while fs::read_to_string(&path)
        .ok()
        .is_none_or(|value| value.trim().is_empty())
    {
        assert!(
            Instant::now() < deadline,
            "Fixture did not become ready: {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn preview(root: &Path) -> (Cli, String) {
    let web = root.join("src/web");
    fs::create_dir_all(web.join("dist")).unwrap();
    fs::write(web.join("rust-web.toml"), "[web]\nentry='src/main.js'\n").unwrap();
    fs::write(web.join("dist/index.html"), "preview fixture").unwrap();
    let mut cli = Cli(Command::new(env!("CARGO_BIN_EXE_blog-assets"))
        .args(["preview", "--product-root"])
        .arg(root)
        .args(["--bind", "127.0.0.1:0"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap());
    let stdout = cli.0.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let reader = thread::spawn(move || {
        let mut line = String::new();
        BufReader::new(stdout).read_line(&mut line).unwrap();
        let _ = sender.send(line);
    });
    let line = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("Preview did not report its listening address");
    reader.join().unwrap();
    let listening: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(listening["event"], "listening");
    (cli, listening["address"].as_str().unwrap().to_string())
}

#[test]
fn idle_preview_terminates_without_waiting_for_connections() {
    let temporary = tempfile::tempdir().unwrap();
    let (mut cli, _) = preview(temporary.path());
    kill_process(Pid::from_child(&cli.0), Signal::TERM).unwrap();
    assert_eq!(cli.wait().code(), Some(143));
}

#[test]
fn preview_terminates_with_a_stalled_response_reader_still_connected() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    let (mut cli, address) = preview(root);
    fs::File::create(root.join("src/web/dist/fixture.wasm"))
        .unwrap()
        .set_len(16 * 1024 * 1024)
        .unwrap();
    let marker = root.join("headers-received");
    let (host, port) = address
        .strip_prefix("http://")
        .unwrap()
        .split_once(':')
        .unwrap();
    let script = concat!(
        "import pathlib,socket,sys,time\n",
        "s=socket.socket(socket.AF_INET,socket.SOCK_STREAM)\n",
        "s.setsockopt(socket.SOL_SOCKET,socket.SO_RCVBUF,1024)\n",
        "s.settimeout(5)\n",
        "s.connect((sys.argv[1],int(sys.argv[2])))\n",
        "s.sendall(b'GET /fixture.wasm HTTP/1.1\\r\\nHost: localhost\\r\\n\\r\\n')\n",
        "headers=b''\n",
        "while not headers.endswith(b'\\r\\n\\r\\n'):\n",
        " chunk=s.recv(1)\n",
        " if not chunk: raise RuntimeError('Connection closed before response headers')\n",
        " headers+=chunk\n",
        "assert headers.startswith(b'HTTP/1.1 200 '),headers\n",
        "pathlib.Path(sys.argv[3]).write_text('ready')\n",
        "time.sleep(60)\n",
    );
    let mut client = Cli(Command::new("python3")
        .args(["-c", script, host, port])
        .arg(&marker)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap());
    wait_for_file(marker);
    kill_process(Pid::from_child(&cli.0), Signal::TERM).unwrap();
    assert_eq!(cli.wait().code(), Some(143));
    assert!(
        client.0.try_wait().unwrap().is_none(),
        "Client must keep the unread connection open until preview exits"
    );
}

#[test]
fn interrupt_and_terminate_clean_owned_groups_before_conventional_exit() {
    for (signal, expected_status) in [(Signal::INT, 130), (Signal::TERM, 143)] {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        let tools = root.join("bin");
        fs::create_dir(&tools).unwrap();
        let git = tools.join("git");
        fs::write(
            &git,
            concat!(
                "#!/bin/sh\n",
                "printf '%s\\n' \"$$\" > \"$BLOG_PROCESS_FIXTURE/git.pid\"\n",
                "sleep 60 &\n",
                "descendant=$!\n",
                "printf '%s\\n' \"$descendant\" > \"$BLOG_PROCESS_FIXTURE/child.pid\"\n",
                "wait \"$descendant\"\n",
            ),
        )
        .unwrap();
        fs::set_permissions(&git, fs::Permissions::from_mode(0o700)).unwrap();
        let mut paths = vec![tools];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let mut cli = Cli(Command::new(env!("CARGO_BIN_EXE_blog-assets"))
            .args(["source-stamp", "--product-root"])
            .arg(root)
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("BLOG_PROCESS_FIXTURE", root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap());
        wait_for_file(root.join("git.pid"));
        let git = FixtureProcess::identify(&root.join("git.pid"));
        wait_for_file(root.join("child.pid"));
        let descendant = FixtureProcess::identify(&root.join("child.pid"));
        assert!(git.running() && descendant.running());

        // Send only to the CLI. Its isolated child group receives no signal
        // unless the executable explicitly asks the runner to cancel it.
        kill_process(Pid::from_child(&cli.0), signal).unwrap();
        let status = cli.wait();
        assert_eq!(status.code(), Some(expected_status), "{signal:?}: {status}");
        let deadline = Instant::now() + Duration::from_secs(1);
        while (git.running() || descendant.running()) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(!git.running(), "{signal:?}: fake Git survived cleanup");
        assert!(
            !descendant.running(),
            "{signal:?}: Git descendant survived cleanup"
        );
    }
}
