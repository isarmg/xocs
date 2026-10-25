//! Bounded subprocess output, deadlines and ownership of a Linux process group.
use rustix::{
    fs::OFlags,
    process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid},
};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::AsFd,
        unix::{fs::OpenOptionsExt, process::CommandExt},
    },
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProcessError {
    #[error("{operation}: {source}")]
    Io {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("{operation}: {source}")]
    System {
        operation: &'static str,
        #[source]
        source: rustix::io::Errno,
    },
    #[error("Allocate bounded command output: {0}")]
    Allocation(#[from] std::collections::TryReserveError),
    #[error("command cancelled")]
    Cancelled,
    #[error("Command {0} exceeded its byte budget")]
    ByteBudget(&'static str),
    #[error("Command output destination stopped accepting bytes")]
    OutputClosed,
    #[error("Command exceeded its time budget")]
    Timeout,
    #[error("Command failed ({0})")]
    Exit(ExitStatus),
    #[error("Command output reader stopped")]
    ReaderStopped,
    #[error("Command output reader panicked")]
    ReaderPanic,
    #[error("{context}: {source}{diagnostics}")]
    Command {
        context: String,
        #[source]
        source: Box<Self>,
        diagnostics: String,
    },
    #[error("{source}; command cleanup also failed: {cleanup}")]
    Cleanup {
        #[source]
        source: Box<Self>,
        cleanup: Box<Self>,
    },
}

type Result<T> = std::result::Result<T, ProcessError>;

fn combine(result: Result<()>, cleanup: Result<()>) -> Result<()> {
    match (result, cleanup) {
        (Ok(()), result) | (result, Ok(())) => result,
        (Err(source), Err(cleanup)) => Err(ProcessError::Cleanup {
            source: Box::new(source),
            cleanup: Box::new(cleanup),
        }),
    }
}

const CHUNK: usize = 8192;
const DIAGNOSTIC: usize = 16 * 1024;
const POLL: Duration = Duration::from_millis(10);
static CANCELLED: AtomicBool = AtomicBool::new(false);
static CANCELLATION: tokio::sync::Notify = tokio::sync::Notify::const_new();

pub(crate) fn cancel() {
    CANCELLED.store(true, Ordering::Release);
    CANCELLATION.notify_waiters();
}

pub(crate) fn cancelled() -> bool {
    CANCELLED.load(Ordering::Acquire)
}

pub(crate) async fn cancellation() {
    // Register before checking the latch, including notification between the
    // check and await; a previously latched request returns at once.
    let pending = CANCELLATION.notified();
    tokio::pin!(pending);
    pending.as_mut().enable();
    if !cancelled() {
        pending.await;
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Policy {
    timeout: Duration,
    stdout: usize,
    stderr: usize,
    live_stdout: bool,
    live_stderr: bool,
}

impl Policy {
    pub(crate) fn query(stdout: usize) -> Self {
        Self {
            timeout: Duration::from_secs(120),
            stdout,
            stderr: 1024 * 1024,
            live_stdout: false,
            live_stderr: false,
        }
    }

    pub(crate) fn git(stdout: usize) -> Self {
        Self {
            timeout: Duration::from_secs(30),
            ..Self::query(stdout)
        }
    }

    pub(crate) fn build(stdout: usize) -> Self {
        Self {
            timeout: Duration::from_secs(45 * 60),
            stderr: 16 * 1024 * 1024,
            live_stderr: true,
            ..Self::query(stdout)
        }
    }

    pub(crate) fn live_stdout(mut self) -> Self {
        self.live_stdout = true;
        self
    }
}

#[derive(Clone, Copy)]
enum Stream {
    Stdout,
    Stderr,
}

impl Stream {
    fn name(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

enum Event {
    Bytes(Stream, Vec<u8>),
    Closed(Stream),
    Failed(Stream, io::Error),
}

fn reader(mut pipe: impl Read, stream: Stream, sender: SyncSender<Event>) {
    let mut buffer = [0; CHUNK];
    loop {
        let event = match pipe.read(&mut buffer) {
            Ok(0) => Event::Closed(stream),
            Ok(count) => Event::Bytes(stream, buffer[..count].to_vec()),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => Event::Failed(stream, error),
        };
        let finished = !matches!(event, Event::Bytes(..));
        if sender.send(event).is_err() || finished {
            return;
        }
    }
}

struct Output {
    bytes: Vec<u8>,
    maximum: usize,
    live: Option<File>,
    forwarded: usize,
    closed: bool,
}

impl Output {
    fn live_destination(stream: Stream) -> Result<Option<File>> {
        let (original, descriptor) = match stream {
            Stream::Stdout => (rustix::io::dup(io::stdout().as_fd()), 1),
            Stream::Stderr => (rustix::io::dup(io::stderr().as_fd()), 2),
        };
        let original = File::from(original.map_err(|source| ProcessError::System {
            operation: "Duplicate command output",
            source,
        })?);
        if original
            .metadata()
            .map_err(|source| ProcessError::Io {
                operation: "Inspect command output",
                source,
            })?
            .is_file()
        {
            // Share the caller's file offset. An independent append descriptor
            // leaves its offset behind, so later parent writes overwrite logs.
            return Ok(Some(original));
        }
        // Pipes/TTY need an independent file description: setting O_NONBLOCK
        // on a dup would also change the caller's stdout/stderr flags. Sockets
        // and environments without procfs keep bounded capture and diagnostics.
        match OpenOptions::new()
            .write(true)
            .custom_flags(OFlags::NONBLOCK.bits() as i32)
            .open(format!("/proc/self/fd/{descriptor}"))
        {
            Ok(file) => Ok(Some(file)),
            // procfs may be unavailable; sockets cannot be reopened this way.
            Err(source)
                if source.kind() == io::ErrorKind::NotFound
                    || source.raw_os_error() == Some(rustix::io::Errno::NXIO.raw_os_error()) =>
            {
                Ok(None)
            }
            Err(source) => Err(ProcessError::Io {
                operation: "Open command output",
                source,
            }),
        }
    }

    fn new(maximum: usize, live: bool, stream: Stream) -> Result<Self> {
        let live = if live {
            Self::live_destination(stream)?
        } else {
            None
        };
        Ok(Self {
            bytes: Vec::new(),
            maximum,
            live,
            forwarded: 0,
            closed: false,
        })
    }

    fn append(&mut self, bytes: &[u8], stream: Stream) -> Result<()> {
        let count = bytes
            .len()
            .min(self.maximum.saturating_sub(self.bytes.len()));
        let needed = self.bytes.len() + count;
        if needed > self.bytes.capacity() {
            let capacity = self
                .bytes
                .capacity()
                .max(CHUNK)
                .saturating_mul(2)
                .max(needed)
                .min(self.maximum);
            self.bytes.try_reserve_exact(capacity - self.bytes.len())?;
        }
        self.bytes.extend_from_slice(&bytes[..count]);
        if count != bytes.len() {
            return Err(ProcessError::ByteBudget(stream.name()));
        }
        Ok(())
    }

    fn forward(&mut self) -> Result<()> {
        if let Some(file) = &mut self.live
            && self.forwarded < self.bytes.len()
        {
            match file.write(&self.bytes[self.forwarded..]) {
                Ok(0) => return Err(ProcessError::OutputClosed),
                Ok(count) => self.forwarded += count,
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(source) => {
                    return Err(ProcessError::Io {
                        operation: "Forward command output",
                        source,
                    });
                }
            }
        }
        Ok(())
    }

    fn complete(&self) -> bool {
        self.closed && (self.live.is_none() || self.forwarded == self.bytes.len())
    }

    fn diagnostic(&self, stream: Stream) -> String {
        if self.bytes.is_empty() {
            return String::new();
        }
        let start = self.bytes.len().saturating_sub(DIAGNOSTIC);
        format!(
            "\n{}{}:\n{}",
            stream.name(),
            if start > 0 { " (last bytes)" } else { "" },
            String::from_utf8_lossy(&self.bytes[start..])
        )
    }
}

struct Running {
    child: Child,
    group: Pid,
    stopped: bool,
    status: Option<ExitStatus>,
    receiver: Option<Receiver<Event>>,
    readers: Vec<JoinHandle<()>>,
}

impl Running {
    fn spawn(command: &mut Command) -> Result<Self> {
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .map_err(|source| ProcessError::Io {
                operation: "Start command",
                source,
            })?;
        let group = Pid::from_child(&child);
        let (sender, receiver) = mpsc::sync_channel(4);
        let mut running = Self {
            child,
            group,
            stopped: false,
            status: None,
            receiver: Some(receiver),
            readers: Vec::new(),
        };
        let stdout = running
            .child
            .stdout
            .take()
            .expect("invariant: child stdout is piped");
        let stdout_sender = sender.clone();
        running.readers.push(
            thread::Builder::new()
                .name("command-stdout".into())
                .spawn(move || reader(stdout, Stream::Stdout, stdout_sender))
                .map_err(|source| ProcessError::Io {
                    operation: "Start stdout reader",
                    source,
                })?,
        );
        let stderr = running
            .child
            .stderr
            .take()
            .expect("invariant: child stderr is piped");
        running.readers.push(
            thread::Builder::new()
                .name("command-stderr".into())
                .spawn(move || reader(stderr, Stream::Stderr, sender))
                .map_err(|source| ProcessError::Io {
                    operation: "Start stderr reader",
                    source,
                })?,
        );
        Ok(running)
    }

    fn stop(&mut self) -> Result<()> {
        if !self.stopped {
            // The leader is deliberately not reaped by waitid. Its PID keeps
            // ownership of this group unambiguous until descendants are killed.
            self.stopped = true;
            let killed = kill_process_group(self.group, Signal::KILL);
            let fallback = match killed {
                Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
                Err(_) => self.child.kill().map_err(|source| ProcessError::Io {
                    operation: "Stop command leader",
                    source,
                }),
            };
            self.status = Some(self.child.wait().map_err(|source| ProcessError::Io {
                operation: "Wait for command",
                source,
            })?);
            let group = match killed {
                Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
                Err(source) => Err(ProcessError::System {
                    operation: "Stop command process group",
                    source,
                }),
            };
            combine(group, fallback)?;
        }
        Ok(())
    }

    fn collect(
        &mut self,
        deadline: Instant,
        stdout: &mut Output,
        stderr: &mut Output,
    ) -> Result<()> {
        loop {
            if cancelled() {
                return Err(ProcessError::Cancelled);
            }
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or(ProcessError::Timeout)?;
            if self.status.is_none() {
                match waitid(
                    WaitId::Pid(self.group),
                    WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
                ) {
                    Ok(Some(_)) => self.stop()?,
                    Ok(None) => {}
                    Err(rustix::io::Errno::INTR) => continue,
                    Err(source) => {
                        return Err(ProcessError::System {
                            operation: "Observe command exit",
                            source,
                        });
                    }
                }
            }
            stdout.forward()?;
            stderr.forward()?;
            if let Some(status) = self.status
                && stdout.complete()
                && stderr.complete()
            {
                return if status.success() {
                    Ok(())
                } else {
                    Err(ProcessError::Exit(status))
                };
            }
            match self
                .receiver
                .as_ref()
                .expect("invariant: output receiver exists until cleanup")
                .recv_timeout(remaining.min(POLL))
            {
                Ok(Event::Bytes(stream, bytes)) => match stream {
                    Stream::Stdout => stdout.append(&bytes, stream)?,
                    Stream::Stderr => stderr.append(&bytes, stream)?,
                },
                Ok(Event::Closed(Stream::Stdout)) => stdout.closed = true,
                Ok(Event::Closed(Stream::Stderr)) => stderr.closed = true,
                Ok(Event::Failed(stream, source)) => {
                    return Err(ProcessError::Io {
                        operation: match stream {
                            Stream::Stdout => "Read command stdout",
                            Stream::Stderr => "Read command stderr",
                        },
                        source,
                    });
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    if !stdout.closed || !stderr.closed {
                        return Err(ProcessError::ReaderStopped);
                    }
                    thread::sleep(remaining.min(POLL));
                }
            }
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        // Release bounded-channel writers before joining, including on overflow.
        self.receiver.take();
        let stopped = self.stop();
        let mut reader_failed = false;
        for reader in self.readers.drain(..) {
            reader_failed |= reader.join().is_err();
        }
        combine(
            stopped,
            if reader_failed {
                Err(ProcessError::ReaderPanic)
            } else {
                Ok(())
            },
        )
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        if let Err(error) = self.cleanup() {
            eprintln!("Command cleanup failed: {error}");
        }
    }
}

/// Capture both streams while the process runs. Memory is limited to the two
/// policy budgets plus fixed reader chunks and a four-chunk queue. Every exit path
/// kills the owned process group before reaping its leader and joining readers.
pub(crate) fn run(command: &mut Command, policy: Policy, context: &str) -> Result<Vec<u8>> {
    if cancelled() {
        return Err(ProcessError::Cancelled);
    }
    let deadline = Instant::now() + policy.timeout;
    let mut stdout = Output::new(policy.stdout, policy.live_stdout, Stream::Stdout)?;
    let mut stderr = Output::new(policy.stderr, policy.live_stderr, Stream::Stderr)?;
    let mut running = Running::spawn(command).map_err(|source| ProcessError::Command {
        context: context.to_owned(),
        source: Box::new(source),
        diagnostics: String::new(),
    })?;
    let result = running.collect(deadline, &mut stdout, &mut stderr);
    let cleanup = running.cleanup();
    combine(result, cleanup).map_err(|source| ProcessError::Command {
        context: context.to_owned(),
        source: Box::new(source),
        diagnostics: format!(
            "{}{}",
            stderr.diagnostic(Stream::Stderr),
            stdout.diagnostic(Stream::Stdout)
        ),
    })?;
    Ok(stdout.bytes)
}

#[cfg(test)]
#[path = "../../../../test/tools/assets/unit/process.rs"]
mod tests;
