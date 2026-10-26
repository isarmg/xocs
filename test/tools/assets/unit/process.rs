use super::*;
use std::{fs, path::Path};

fn policy() -> Policy {
    Policy {
        timeout: Duration::from_secs(3),
        stdout: 1024,
        stderr: 1024,
        live_stdout: false,
        live_stderr: false,
    }
}

fn python(script: &str) -> Command {
    let mut command = Command::new("python3");
    command.args(["-c", script]);
    command
}

fn gone_or_zombie(pid: &str) -> bool {
    fs::read_to_string(format!("/proc/{}/stat", pid.trim()))
        .map(|stat| stat.rsplit_once(") ").unwrap().1.starts_with('Z'))
        .unwrap_or(true)
}

fn assert_stopped(marker: &Path) {
    let pid = fs::read_to_string(marker).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    while !gone_or_zombie(&pid) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    assert!(gone_or_zombie(&pid), "fixture child remains alive: {pid}");
}

#[test]
fn captures_both_streams_and_keeps_nonzero_diagnostics() {
    let output = run(
        &mut python("import os; os.write(1,b'result'); os.write(2,b'compiler warning')"),
        policy(),
        "fixture query",
    )
    .unwrap();
    assert_eq!(output, b"result");
    let error = run(
        &mut python("import os; os.write(1,b'partial result'); os.write(2,b'compiler error'); raise SystemExit(7)"),
        policy(),
        "fixture query",
    )
    .unwrap_err();
    let diagnostic = crate::error_chain(&error);
    assert!(diagnostic.contains("compiler error"), "{diagnostic}");
    assert!(diagnostic.contains("partial result"), "{diagnostic}");
    let ProcessError::Command { source, .. } = &error else {
        panic!("expected a command failure: {diagnostic}");
    };
    let ProcessError::Exit(status) = source.as_ref() else {
        panic!("expected the process exit status: {diagnostic}");
    };
    assert_eq!(status.code(), Some(7));
    assert_eq!(diagnostic.matches(&status.to_string()).count(), 1);
}

#[test]
fn spawn_failure_retains_the_original_io_error() {
    let temporary = tempfile::tempdir().unwrap();
    let error = run(
        &mut Command::new(temporary.path().join("missing-command")),
        policy(),
        "missing command fixture",
    )
    .unwrap_err();
    let ProcessError::Command { source, .. } = &error else {
        panic!("expected command context: {error}");
    };
    let source = std::error::Error::source(source.as_ref()).unwrap();
    assert_eq!(
        source.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn stdout_and_stderr_overflow_stop_a_sleeping_process() {
    for descriptor in [1, 2] {
        let temporary = tempfile::tempdir().unwrap();
        let marker = temporary.path().join("pid");
        let script = format!(
            "import os,pathlib,sys,time; pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); os.write({descriptor},b'x'*8192); time.sleep(30)"
        );
        let started = Instant::now();
        let error = run(python(&script).arg(&marker), policy(), "overflow fixture").unwrap_err();
        let ProcessError::Command { source, .. } = &error else {
            panic!("expected a command failure: {error:?}");
        };
        let ProcessError::ByteBudget(stream) = source.as_ref() else {
            panic!("expected a stream byte budget failure: {error:?}");
        };
        assert_eq!(*stream, if descriptor == 1 { "stdout" } else { "stderr" });
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_stopped(&marker);
    }
}

#[test]
fn simultaneous_streams_are_drained_with_bounded_channels() {
    let script = "import os,threading\na=threading.Thread(target=lambda:os.write(1,b'o'*65536))\nb=threading.Thread(target=lambda:os.write(2,b'e'*65536))\na.start(); b.start(); a.join(); b.join()";
    let output = run(
        &mut python(script),
        Policy {
            stdout: 65536,
            stderr: 65536,
            ..policy()
        },
        "concurrent fixture",
    )
    .unwrap();
    assert_eq!(output, vec![b'o'; 65536]);
}

#[test]
fn silent_process_obeys_the_whole_operation_deadline() {
    let temporary = tempfile::tempdir().unwrap();
    let marker = temporary.path().join("pid");
    let script = "import os,pathlib,sys,time; pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); time.sleep(30)";
    let started = Instant::now();
    let error = run(
        python(script).arg(&marker),
        Policy {
            timeout: Duration::from_millis(250),
            ..policy()
        },
        "timeout fixture",
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProcessError::Command { source, .. } if matches!(source.as_ref(), ProcessError::Timeout)
    ));
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_stopped(&marker);
}

#[test]
fn parent_exit_cleans_up_pipe_holding_descendants() {
    let temporary = tempfile::tempdir().unwrap();
    let marker = temporary.path().join("descendant");
    let script = "import os,pathlib,sys,time\np=pathlib.Path(sys.argv[1])\nif os.fork()==0:\n p.write_text(str(os.getpid()))\n time.sleep(30)\n os._exit(0)\nwhile not p.exists(): time.sleep(.001)\nos._exit(0)";
    let started = Instant::now();
    let output = run(python(script).arg(&marker), policy(), "descendant fixture").unwrap();
    assert!(output.is_empty());
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_stopped(&marker);
}

#[test]
fn timeout_stops_readers_when_a_descendant_leaves_the_owned_process_group() {
    let temporary = tempfile::tempdir().unwrap();
    let marker = temporary.path().join("detached");
    let script = concat!(
        "import os,pathlib,sys,time\n",
        "p=pathlib.Path(sys.argv[1])\n",
        "if os.fork()==0:\n",
        " os.setsid()\n",
        " started=pathlib.Path('/proc/self/stat').read_text().rsplit(') ',1)[1].split()[19]\n",
        " p.write_text(str(os.getpid())+' '+started)\n",
        " time.sleep(2)\n",
        " os._exit(0)\n",
        "while not p.exists(): time.sleep(.001)\n",
        "os._exit(0)\n",
    );
    let started = Instant::now();
    let error = run(
        python(script).arg(&marker),
        Policy {
            timeout: Duration::from_millis(200),
            ..policy()
        },
        "detached pipe holder fixture",
    )
    .unwrap_err();
    let elapsed = started.elapsed();
    // The fixture owns only this recorded process instance. A real daemon is
    // outside the runner's process group and is never chased by production cleanup.
    let identity = fs::read_to_string(marker).unwrap();
    let (pid, process_start) = identity.split_once(' ').unwrap();
    if let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) {
        let fields = stat
            .rsplit_once(") ")
            .unwrap()
            .1
            .split_whitespace()
            .collect::<Vec<_>>();
        if fields[0] != "Z" && fields[19] == process_start {
            rustix::process::kill_process(
                Pid::from_raw(pid.parse().unwrap()).unwrap(),
                Signal::KILL,
            )
            .unwrap();
        }
    }
    assert!(matches!(
        error,
        ProcessError::Command { source, .. } if matches!(source.as_ref(), ProcessError::Timeout)
    ));
    assert!(
        elapsed < Duration::from_secs(1),
        "reader cleanup took {elapsed:?}"
    );
}

#[test]
fn live_output_shares_regular_file_offsets_with_parent_writes() {
    let temporary = tempfile::tempdir().unwrap();
    let log = temporary.path().join("combined.log");
    let file = File::create(&log).unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::tests::live_regular_file_fixture",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .stdout(Stdio::from(file.try_clone().unwrap()))
        .stderr(Stdio::from(file))
        .status()
        .unwrap();
    let bytes = fs::read_to_string(log).unwrap();
    assert!(status.success(), "{bytes}");
    assert!(
        bytes.contains(concat!(
            "regular-parent-before\n",
            "regular-child-stdout\n",
            "regular-parent-middle\n",
            "regular-child-stderr\n",
            "regular-parent-after\n",
        )),
        "Child and parent log order changed or bytes were overwritten: {bytes}"
    );
}

#[test]
#[ignore = "executed in an isolated subprocess by the regular-file regression"]
fn live_regular_file_fixture() {
    io::stdout().write_all(b"regular-parent-before\n").unwrap();
    io::stdout().flush().unwrap();
    run(
        &mut python("import os; os.write(1,b'regular-child-stdout\\n')"),
        Policy {
            live_stdout: true,
            ..policy()
        },
        "regular stdout fixture",
    )
    .unwrap();
    io::stderr().write_all(b"regular-parent-middle\n").unwrap();
    io::stderr().flush().unwrap();
    run(
        &mut python("import os; os.write(2,b'regular-child-stderr\\n')"),
        Policy {
            live_stderr: true,
            ..policy()
        },
        "regular stderr fixture",
    )
    .unwrap();
    io::stdout().write_all(b"regular-parent-after\n").unwrap();
    io::stdout().flush().unwrap();
}

#[test]
fn cancellation_is_latched_without_changing_other_library_tests() {
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "process::tests::cancellation_latch_fixture",
            "--ignored",
        ])
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
#[ignore = "executed separately because cancellation is a process-wide CLI latch"]
fn cancellation_latch_fixture() {
    crate::request_cancellation();
    let error = run(
        &mut Command::new("must-not-spawn-after-cancellation"),
        policy(),
        "cancelled fixture",
    )
    .unwrap_err();
    assert!(format!("{error:#}").contains("command cancelled"));
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tokio::time::timeout(Duration::from_millis(100), cancellation())
                .await
                .expect("Previously latched cancellation must wake late listeners");
        });
}
