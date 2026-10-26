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

#[test]
fn invalid_utf8_arguments_return_a_cli_error() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let output = Command::new(env!("CARGO_BIN_EXE_blog-assets"))
        .arg(OsString::from_vec(vec![0xff]))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("command-line arguments must be UTF-8"));
    assert!(!diagnostic.contains("panicked"));
}

struct FixtureProcess {
    pid: Pid,
    started: String,
}

impl FixtureProcess {
    fn identify(path: &Path) -> Self {
        let pid: i32 = fs::read_to_string(path)
            .expect("read the fixture's published PID")
            .trim()
            .parse()
            .expect("invariant: the fixture publishes a decimal PID");
        let pid = Pid::from_raw(pid).expect("invariant: a running fixture has a positive PID");
        let (_, started) = Self::state(pid).expect("Fixture process is running");
        Self { pid, started }
    }

    fn state(pid: Pid) -> Option<(char, String)> {
        let stat = match fs::read_to_string(format!("/proc/{}/stat", pid.as_raw_nonzero())) {
            Ok(stat) => stat,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => panic!("read fixture process state: {error}"),
        };
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
        if self.running()
            && let Err(error) = kill_process(self.pid, Signal::KILL)
            && error != rustix::io::Errno::SRCH
        {
            eprintln!("stop remaining fixture process: {error}");
        }
    }
}

struct Cli(Child);

impl Cli {
    fn wait(&mut self) -> ExitStatus {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.0.try_wait().expect("observe the fixture CLI's exit") {
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
        // A completed fixture may already have been reaped by its assertion.
        if let Err(error) = self.0.kill()
            && error.kind() != std::io::ErrorKind::InvalidInput
        {
            eprintln!("stop fixture child: {error}");
        }
        if let Err(error) = self.0.wait() {
            eprintln!("reap fixture child: {error}");
        }
    }
}

fn wait_for_file(path: PathBuf) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match fs::read_to_string(&path) {
            Ok(value) if !value.trim().is_empty() => return,
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("read fixture readiness {}: {error}", path.display()),
        }
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
    fs::create_dir_all(web.join("dist")).expect("create the preview fixture's output");
    fs::write(web.join("rust-web.toml"), "[web]\nentry='src/main.js'\n")
        .expect("write the preview fixture's configuration");
    fs::write(web.join("dist/index.html"), "preview fixture")
        .expect("write the preview fixture's entry page");
    let mut cli = Cli(Command::new(env!("CARGO_BIN_EXE_blog-assets"))
        .args(["preview", "--product-root"])
        .arg(root)
        .args(["--bind", "127.0.0.1:0"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("start the preview fixture"));
    let stdout = cli
        .0
        .stdout
        .take()
        .expect("invariant: preview stdout is piped");
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let reader = thread::spawn(move || {
        let mut line = String::new();
        BufReader::new(stdout)
            .read_line(&mut line)
            .expect("read the preview's listening event");
        sender
            .send(line)
            .expect("invariant: the fixture waits for the preview's listening event");
    });
    let line = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("Preview did not report its listening address");
    reader.join().expect("the preview event reader must finish");
    let listening: serde_json::Value =
        serde_json::from_str(&line).expect("preview must emit a JSON listening event");
    assert_eq!(listening["event"], "listening");
    (
        cli,
        listening["address"]
            .as_str()
            .expect("the listening event must contain its address")
            .to_string(),
    )
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
