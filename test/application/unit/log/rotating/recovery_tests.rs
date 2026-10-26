use super::*;
use crate::log::{CommonEvent, LogFilter, QueryLimits, query};
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

// Resource-limit failures are confined to a subprocess, never the parallel
// test runner. All files belong to that subprocess's temporary directory.
fn in_child() -> bool {
    let name = std::thread::current()
        .name()
        .expect("test harness names each recovery test")
        .to_owned();
    if std::env::var("LOG_RECOVERY_TEST").as_deref() == Ok(&name) {
        return true;
    }
    let status = Command::new(std::env::current_exe().expect("test executable is available"))
        .args(["--exact", &name, "--nocapture"])
        .env("LOG_RECOVERY_TEST", &name)
        .status()
        .expect("recovery test child starts");
    assert!(status.success());
    false
}

#[allow(unsafe_code)]
fn limited<T>(resource: libc::__rlimit_resource_t, bytes: u64, f: impl FnOnce() -> T) -> T {
    // SAFETY: this is an isolated child process; valid rlimit pointers, and
    // the original limits and signal disposition are restored after f.
    unsafe {
        let mut old = std::mem::zeroed::<libc::rlimit>();
        assert_eq!(libc::getrlimit(resource, &mut old), 0);
        let handler = libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
        let new = libc::rlimit {
            rlim_cur: bytes,
            ..old
        };
        assert_eq!(libc::setrlimit(resource, &new), 0);
        let result = f();
        assert_eq!(libc::setrlimit(resource, &old), 0);
        libc::signal(libc::SIGXFSZ, handler);
        result
    }
}

fn fixture() -> (tempfile::TempDir, RotatingLogFile, LogRecord) {
    let directory = tempfile::tempdir().expect("fixture creates a temporary log directory");
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .expect("fixture owns its log directory");
    let sink = RotatingLogFile::open(
        directory.path(),
        "events",
        LogRetention {
            file_bytes: MAX_RECORD_BYTES as u64,
            archives: 2,
        },
    )
    .expect("fixture log directory and retention are valid");
    let record = LogRecord::common("example", CommonEvent::RuntimeStarted)
        .expect("fixture uses a valid service identity");
    (directory, sink, record)
}

#[test]
fn partial_append_rolls_back_and_later_records_remain_queryable() {
    if !in_child() {
        return;
    }
    let (directory, mut sink, record) = fixture();
    let active = directory.path().join("events.jsonl");
    sink.write(&record).unwrap();
    let before = fs::read(&active).unwrap();
    let error = limited(libc::RLIMIT_FSIZE, before.len() as u64 + 17, || {
        sink.write(&record)
    });
    assert!(matches!(error, Err(LogError::Io(_))));
    assert_eq!(fs::read(&active).unwrap(), before);
    sink.write(&record).unwrap();
    let bytes = fs::read(&active).unwrap();
    assert_eq!(
        query(&bytes[..], LogFilter::default(), QueryLimits::default())
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn failed_replacement_creation_retries_without_rotating_archives_twice() {
    if !in_child() {
        return;
    }
    let (directory, mut sink, record) = fixture();
    let active = directory.path().join("events.jsonl");
    let line = record.json_line().unwrap();
    while fs::metadata(&active).unwrap().len() + line.len() as u64 <= MAX_RECORD_BYTES as u64 {
        sink.write(&record).unwrap();
    }
    let before = fs::read(&active).unwrap();
    let error = limited(libc::RLIMIT_NOFILE, 0, || sink.write(&record));
    assert!(matches!(error, Err(LogError::Io(_))));
    assert!(!active.exists());
    let archived = directory.path().join("events.1.jsonl");
    assert_eq!(fs::read(&archived).unwrap(), before);
    sink.write(&record).unwrap();
    assert_eq!(fs::read(&archived).unwrap(), before);
    assert_eq!(fs::read(&active).unwrap(), line);
    assert!(!directory.path().join("events.2.jsonl").exists());
    let padded = record.with_attribute("padding", "x".repeat(4096)).unwrap();
    for _ in 0..30 {
        sink.write(&padded).unwrap();
    }
    for name in ["events.jsonl", "events.1.jsonl", "events.2.jsonl"] {
        let bytes = fs::read(directory.path().join(name)).unwrap();
        assert!(bytes.len() <= MAX_RECORD_BYTES);
        assert!(
            !query(&bytes[..], LogFilter::default(), QueryLimits::default())
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn directory_rename_keeps_appends_and_rotation_on_the_held_directory() {
    let parent = tempfile::tempdir().unwrap();
    let original = parent.path().join("original");
    let moved = parent.path().join("moved");
    fs::create_dir(&original).unwrap();
    fs::set_permissions(&original, fs::Permissions::from_mode(0o700)).unwrap();
    let mut sink = RotatingLogFile::open(
        &original,
        "events",
        LogRetention {
            file_bytes: MAX_RECORD_BYTES as u64,
            archives: 2,
        },
    )
    .unwrap();
    fs::rename(&original, &moved).unwrap();
    fs::create_dir(&original).unwrap();
    let record = LogRecord::common("example", CommonEvent::RuntimeStarted)
        .unwrap()
        .with_attribute("padding", "x".repeat(4096))
        .unwrap();
    for _ in 0..30 {
        sink.write(&record).unwrap();
    }
    assert_eq!(fs::read_dir(&original).unwrap().count(), 0);
    for name in ["events.jsonl", "events.1.jsonl", "events.2.jsonl"] {
        let bytes = fs::read(moved.join(name)).unwrap();
        assert!(bytes.len() <= MAX_RECORD_BYTES);
        assert!(
            !query(&bytes[..], LogFilter::default(), QueryLimits::default())
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn dropping_writer_releases_the_lease_even_while_a_descriptor_alias_survives() {
    let (directory, sink, _) = fixture();
    // A concurrent process spawn can inherit this description until exec.
    let inherited = sink.directory.file.try_clone().unwrap();
    drop(sink);
    let replacement = RotatingLogFile::open(
        directory.path(),
        "events",
        LogRetention {
            file_bytes: MAX_RECORD_BYTES as u64,
            archives: 2,
        },
    )
    .unwrap();
    drop(inherited);
    assert!(matches!(
        RotatingLogFile::open(directory.path(), "events", LogRetention::default()),
        Err(LogError::WriterLock(_))
    ));
    drop(replacement);
    RotatingLogFile::open(directory.path(), "events", LogRetention::default()).unwrap();
}

#[test]
fn explicit_writer_release_hands_off_ownership_while_a_descriptor_alias_survives() {
    let (directory, sink, _) = fixture();
    let inherited = sink.directory.file.try_clone().unwrap();
    sink.release().unwrap();
    let replacement = RotatingLogFile::open(
        directory.path(),
        "events",
        LogRetention {
            file_bytes: MAX_RECORD_BYTES as u64,
            archives: 2,
        },
    )
    .unwrap();
    drop(inherited);
    assert!(matches!(
        RotatingLogFile::open(directory.path(), "events", LogRetention::default()),
        Err(LogError::WriterLock(_))
    ));
    replacement.release().unwrap();
}
