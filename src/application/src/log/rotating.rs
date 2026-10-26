//! A held private directory and an exclusive writer lease own log publication.
//! Existing files are checked when the sink opens; subsequent operations use
//! the directory descriptor and the writer's byte count.

use crate::log::{LogError, LogLine, LogRecord, LogRetention, MAX_RECORD_BYTES};
use rustix::fs::{AtFlags, FlockOperation, Mode, OFlags, flock, openat, renameat, unlinkat};
use std::{
    fs::{File, Metadata, OpenOptions},
    io::{self, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

pub struct RotatingLogFile {
    directory: DirectoryLease,
    stem: String,
    active_name: String,
    custom_name: bool,
    file: File,
    bytes: u64,
    state: WriteState,
    retention: LogRetention,
}

struct DirectoryLease {
    file: File,
    held: bool,
}

impl DirectoryLease {
    fn acquire(file: File) -> Result<Self, LogError> {
        flock(&file, FlockOperation::NonBlockingLockExclusive)
            .map_err(|source| LogError::WriterLock(io::Error::from(source)))?;
        Ok(Self { file, held: true })
    }

    fn release(&mut self) -> Result<(), LogError> {
        if self.held {
            self.file.unlock().map_err(LogError::WriterRelease)?;
            self.held = false;
        }
        Ok(())
    }
}

impl Drop for DirectoryLease {
    fn drop(&mut self) {
        if let Err(error) = self.release() {
            // A sink may be dropped while its logger is locked. Stderr is the
            // final diagnostic sink; its own failure cannot be reported further.
            drop(writeln!(
                io::stderr().lock(),
                "log writer lease release failed: {error}"
            ));
        }
    }
}

#[derive(Clone, Copy)]
enum WriteState {
    Active,
    MoveArchives { next: u8 },
    ArchiveActive,
    CreateActive,
    SyncDirectory,
    Failed,
}

impl RotatingLogFile {
    /// The caller creates the dedicated directory explicitly. Opening this
    /// sink never deletes unknown entries or repairs permissions. Existing
    /// files outside the declared retention policy are rejected.
    pub fn open(
        path: impl AsRef<Path>,
        stem: &str,
        retention: LogRetention,
    ) -> Result<Self, LogError> {
        let path = path.as_ref();
        if !path.is_absolute()
            || stem.is_empty()
            || stem.len() > 64
            || !stem
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Err(LogError::UnsafeStorage);
        }
        Self::open_named(path, stem, &format!("{stem}.jsonl"), false, retention)
    }

    /// Use an explicit active filename in an existing private directory.
    /// Archives are `<filename>.1` through the declared retention count.
    pub fn open_file(
        active_path: impl AsRef<Path>,
        retention: LogRetention,
    ) -> Result<Self, LogError> {
        let active_path = active_path.as_ref();
        let path = active_path.parent().ok_or(LogError::UnsafeStorage)?;
        let name = active_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(LogError::UnsafeStorage)?;
        if !active_path.is_absolute()
            || name.is_empty()
            || name.len() > 128
            || name.starts_with('.')
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
        {
            return Err(LogError::UnsafeStorage);
        }
        Self::open_named(path, name, name, true, retention)
    }

    fn open_named(
        path: &Path,
        stem: &str,
        active_name: &str,
        custom_name: bool,
        retention: LogRetention,
    ) -> Result<Self, LogError> {
        if retention.file_bytes < MAX_RECORD_BYTES as u64
            || retention.archives == 0
            || retention.archives > 32
        {
            return Err(LogError::InvalidLimits);
        }
        let directory = OpenOptions::new()
            .read(true)
            .custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as i32)
            .open(path)?;
        let metadata = directory.metadata()?;
        if !metadata.is_dir()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o777 != 0o700
        {
            return Err(LogError::UnsafeStorage);
        }
        // Active names and archive names can overlap across constructors.
        // One lease for the dedicated directory owns the entire namespace.
        let directory = DirectoryLease::acquire(directory)?;
        // Refuse a configuration that silently strands archives from an older
        // larger policy. Operators must change retention through an explicit
        // cleanup action, rather than normal startup deleting extra files.
        let mut entries = rustix::fs::Dir::read_from(&directory.file).map_err(io::Error::from)?;
        while let Some(entry) = entries.read() {
            let entry = entry.map_err(io::Error::from)?;
            let name = entry.file_name().to_str().ok();
            let index = name
                .and_then(|name| name.strip_prefix(&format!("{stem}.")))
                .and_then(|name| {
                    if custom_name {
                        Some(name)
                    } else {
                        name.strip_suffix(".jsonl")
                    }
                });
            if let Some(spelling) = index {
                let index: u8 = spelling.parse().map_err(LogError::ArchiveIndex)?;
                if index == 0 || index > retention.archives || spelling != index.to_string() {
                    return Err(LogError::UnsafeStorage);
                }
            }
        }
        let file = open_private(&directory.file, active_name, false)?;
        let bytes = file.metadata()?.len();
        if bytes > retention.file_bytes {
            return Err(LogError::InvalidLimits);
        }
        let sink = Self {
            directory,
            stem: stem.into(),
            active_name: active_name.into(),
            custom_name,
            file,
            bytes,
            state: WriteState::Active,
            retention,
        };
        for index in 1..=retention.archives {
            if let Some(metadata) = sink.archive_metadata(index)?
                && metadata.len() > retention.file_bytes
            {
                return Err(LogError::InvalidLimits);
            }
        }
        Ok(sink)
    }

    pub fn write(&mut self, record: &LogRecord) -> Result<(), LogError> {
        let line = record.encode()?;
        self.write_line(&line)
    }

    /// Hand off writer ownership after completing all writes. Appended records
    /// retain the normal log durability policy; Drop also releases the lease.
    ///
    /// # Errors
    /// Returns [`LogError::WriterRelease`] if the descriptor cannot be unlocked.
    pub fn release(mut self) -> Result<(), LogError> {
        self.directory.release()
    }

    pub(super) fn write_line(&mut self, line: &LogLine) -> Result<(), LogError> {
        let line = line.0.as_slice();
        self.finish_rotation()?;
        // Both lengths are bounded when accepted, so subtraction also handles
        // a retention limit of u64::MAX without overflow.
        if line.len() as u64 > self.retention.file_bytes - self.bytes {
            self.rotate()?;
        }
        if let Err(error) = self.file.write_all(line) {
            // Preserve complete records. A failed rollback makes this writer
            // unusable until the caller opens and validates it again.
            if let Err(rollback) = self.file.set_len(self.bytes) {
                self.state = WriteState::Failed;
                return Err(LogError::AppendRollback {
                    source: error,
                    rollback,
                });
            }
            return Err(error.into());
        }
        self.bytes += line.len() as u64;
        Ok(())
    }

    fn rotate(&mut self) -> Result<(), LogError> {
        self.file.sync_all()?;
        match unlinkat(
            &self.directory.file,
            self.archive_path(self.retention.archives),
            AtFlags::empty(),
        ) {
            Ok(()) | Err(rustix::io::Errno::NOENT) => {}
            Err(error) => return Err(io::Error::from(error).into()),
        }
        self.state = WriteState::MoveArchives {
            next: self.retention.archives - 1,
        };
        self.finish_rotation()
    }

    fn finish_rotation(&mut self) -> Result<(), LogError> {
        loop {
            self.state = match self.state {
                WriteState::Active => return Ok(()),
                WriteState::MoveArchives { next: 0 } => WriteState::ArchiveActive,
                WriteState::MoveArchives { next } => {
                    match renameat(
                        &self.directory.file,
                        self.archive_path(next),
                        &self.directory.file,
                        self.archive_path(next + 1),
                    ) {
                        Ok(()) | Err(rustix::io::Errno::NOENT) => {}
                        Err(error) => return Err(io::Error::from(error).into()),
                    }
                    WriteState::MoveArchives { next: next - 1 }
                }
                WriteState::ArchiveActive => {
                    renameat(
                        &self.directory.file,
                        &self.active_name,
                        &self.directory.file,
                        self.archive_path(1),
                    )
                    .map_err(io::Error::from)?;
                    WriteState::CreateActive
                }
                WriteState::CreateActive => {
                    // Keep the archived descriptor until creation succeeds.
                    self.file = open_private(&self.directory.file, &self.active_name, true)?;
                    self.bytes = 0;
                    WriteState::SyncDirectory
                }
                WriteState::SyncDirectory => {
                    self.directory.file.sync_all()?;
                    WriteState::Active
                }
                WriteState::Failed => return Err(LogError::WriterFailed),
            };
        }
    }

    fn archive_path(&self, index: u8) -> PathBuf {
        PathBuf::from(if self.custom_name {
            format!("{}.{index}", self.stem)
        } else {
            format!("{}.{index}.jsonl", self.stem)
        })
    }
    fn archive_metadata(&self, index: u8) -> Result<Option<Metadata>, LogError> {
        match openat(
            &self.directory.file,
            self.archive_path(index),
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(file) => {
                let metadata = File::from(file).metadata()?;
                require_private(&metadata)?;
                Ok(Some(metadata))
            }
            Err(rustix::io::Errno::NOENT) => Ok(None),
            Err(error) => Err(io::Error::from(error).into()),
        }
    }
}

fn open_private(directory: &File, name: &str, create_new: bool) -> Result<File, LogError> {
    let mut flags = OFlags::RDWR
        | OFlags::APPEND
        | OFlags::CREATE
        | OFlags::NOFOLLOW
        | OFlags::NONBLOCK
        | OFlags::CLOEXEC;
    if create_new {
        flags |= OFlags::EXCL;
    }
    let file = File::from(
        openat(directory, name, flags, Mode::RUSR | Mode::WUSR).map_err(io::Error::from)?,
    );
    require_private(&file.metadata()?)?;
    Ok(file)
}
fn require_private(metadata: &Metadata) -> Result<(), LogError> {
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.mode() & 0o777 != 0o600
        || metadata.uid() != rustix::process::geteuid().as_raw()
    {
        Err(LogError::UnsafeStorage)
    } else {
        Ok(())
    }
}
#[path = "../../../../test/application/unit/log/rotating/recovery_tests.rs"]
#[cfg(all(test, target_os = "linux"))]
mod recovery_tests;
