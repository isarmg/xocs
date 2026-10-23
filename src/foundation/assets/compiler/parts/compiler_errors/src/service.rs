use std::{
    borrow::Cow,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
};

use crate::borrowed_text::CowUtils;
use url_encoding::percent::AsciiSet;
#[cfg(not(windows))]
use std::fs::canonicalize as strict_canonicalize;

use crate::compiler_errors::{
    Error, NamedSource, OxcDiagnostic, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult},
};

pub type DiagnosticSender = mpsc::Sender<Vec<Error>>;
type DiagnosticReceiver = mpsc::Receiver<Vec<Error>>;

/// Listens for diagnostics sent over a [channel](DiagnosticSender) by some job, and
/// formats/reports them to the user.
///
/// [`DiagnosticService`] is designed to support multi-threaded jobs that may produce
/// reports. These jobs can send messages to the service over its multi-producer,
/// single-consumer channel.
///
/// # Example
/// ```rust,ignore
/// use std::{io, thread};
/// use oxc_diagnostics::{OxcDiagnostic, DiagnosticService, GraphicalReportHandler};
///
/// // Create a service with a graphical reporter
/// let (mut service, sender) = DiagnosticService::new(Box::new(GraphicalReportHandler::new()));
///
/// // Spawn a thread that does work and reports diagnostics
/// thread::spawn(move || {
///     sender.send(vec![OxcDiagnostic::error("Something went wrong").into()]).unwrap();
///
///     // The service will stop listening when all senders are dropped.
///     // No explicit termination signal is needed.
/// });
///
/// // Listen for and process messages
/// service.run(&mut io::stdout().lock());
/// ```
pub struct DiagnosticService {
    reporter: Box<dyn DiagnosticReporter>,

    /// Whether to suppress warnings and report only errors.
    quiet: bool,

    /// Whether to suppress all diagnostics.
    silent: bool,

    /// Warning threshold used to determine the exit status.
    max_warnings: Option<usize>,

    receiver: DiagnosticReceiver,
}

impl DiagnosticService {
    /// Create a new [`DiagnosticService`] that will render and report diagnostics using the
    /// provided [`DiagnosticReporter`].
    pub fn new(reporter: Box<dyn DiagnosticReporter>) -> (Self, DiagnosticSender) {
        let (sender, receiver) = mpsc::channel();
        (Self { reporter, quiet: false, silent: false, max_warnings: None, receiver }, sender)
    }

    /// Set to `true` to only report errors and ignore warnings.
    ///
    /// Use [`with_silent`](DiagnosticService::with_silent) to disable reporting entirely.
    ///
    /// Default: `false`
    #[must_use]
    pub fn with_quiet(mut self, yes: bool) -> Self {
        self.quiet = yes;
        self
    }

    /// Set to `true` to disable reporting entirely.
    ///
    /// Use [`with_quiet`](DiagnosticService::with_quiet) to only disable reporting on warnings.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn with_silent(mut self, yes: bool) -> Self {
        self.silent = yes;
        self
    }

    /// Specify a warning threshold, which can be used to force exit with an error status if there
    /// are too many warning-level rule violations in your project. Errors do not count towards the
    /// warning limit.
    ///
    /// Use [`DiagnosticResult`](DiagnosticResult::max_warnings_exceeded) to check if too
    /// many warnings have been received.
    ///
    /// Default: [`None`]
    #[must_use]
    pub fn with_max_warnings(mut self, max_warnings: Option<usize>) -> Self {
        self.max_warnings = max_warnings;
        self
    }

    /// Check if the max warning threshold, as set by
    /// [`with_max_warnings`](DiagnosticService::with_max_warnings), has been exceeded.
    fn max_warnings_exceeded(&self, warnings_count: usize) -> bool {
        self.max_warnings.is_some_and(|max_warnings| warnings_count > max_warnings)
    }

    /// Attach the source code and path to diagnostics, converting them into [`Error`]s.
    pub fn wrap_diagnostics<C: AsRef<Path>, P: AsRef<Path>>(
        cwd: C,
        path: P,
        source_text: &str,
        diagnostics: Vec<OxcDiagnostic>,
    ) -> Vec<Error> {
        // TODO: This causes snapshots to fail when running tests through a JetBrains terminal.
        let is_jetbrains =
            std::env::var("TERMINAL_EMULATOR").is_ok_and(|x| x.eq("JetBrains-JediTerm"));

        let path_ref = path.as_ref();
        let path_display = if is_jetbrains { from_file_path(path_ref) } else { None }
            .unwrap_or_else(|| {
                let relative_path =
                    path_ref.strip_prefix(cwd).unwrap_or(path_ref).to_string_lossy();
                let normalized_path = relative_path.cow_replace('\\', "/");
                normalized_path.to_string()
            });

        let source = Arc::new(NamedSource::new(path_display, source_text.to_owned()));
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.with_source_code(Arc::clone(&source)))
            .collect()
    }

    /// # Panics
    ///
    /// * When the writer fails to write
    ///
    /// The result is also passed to the reporter, but remains part of this API because callers use
    /// it to determine their exit status.
    pub fn run(&mut self, writer: &mut dyn Write) -> DiagnosticResult {
        let mut warnings_count: usize = 0;
        let mut errors_count: usize = 0;
        let supports_minified_file_fallback = self.reporter.supports_minified_file_fallback();

        while let Ok(diagnostics) = self.receiver.recv() {
            let mut diagnostics_to_render = Vec::with_capacity(diagnostics.len());
            for diagnostic in diagnostics {
                match diagnostic.severity() {
                    Some(Severity::Warning) => {
                        warnings_count += 1;
                        // `--quiet` follows ESLint by suppressing warnings but not errors.
                        if self.quiet {
                            continue;
                        }
                    }
                    Some(Severity::Error) | None => {
                        errors_count += 1;
                    }
                    Some(Severity::Advice) => {}
                }

                if !self.silent {
                    diagnostics_to_render.push(diagnostic);
                }
            }

            if diagnostics_to_render.is_empty() {
                continue;
            }

            let mut is_minified = false;
            let mut path = None;
            if supports_minified_file_fallback {
                self.reporter.render_errors_until(
                    diagnostics_to_render,
                    &mut |source_name, rendered| {
                        // Setting to 1200 because graphical output may contain ansi escape codes
                        // and other decorations.
                        if rendered.lines().any(|line| line.len() >= 1200) {
                            is_minified = true;
                            path = source_name.map(ToString::to_string);
                            false
                        } else {
                            writer
                                .write_all(rendered.as_bytes())
                                .or_else(Self::check_for_writer_error)
                                .unwrap();
                            true
                        }
                    },
                );
            } else {
                self.reporter.render_errors(diagnostics_to_render, &mut |rendered| {
                    writer
                        .write_all(rendered.as_bytes())
                        .or_else(Self::check_for_writer_error)
                        .unwrap();
                });
            }

            if is_minified {
                let mut diagnostic = OxcDiagnostic::warn("File is too long to fit on the screen");
                if let Some(path) = path {
                    diagnostic = diagnostic.with_help(format!("{path} seems like a minified file"));
                }
                if let Some(err_str) = self.reporter.render_error(diagnostic.into()) {
                    writer
                        .write_all(err_str.as_bytes())
                        .or_else(Self::check_for_writer_error)
                        .unwrap();
                }
            }
        }

        let result = DiagnosticResult::new(
            warnings_count,
            errors_count,
            self.max_warnings_exceeded(warnings_count),
        );

        if let Some(finish_output) = self.reporter.finish(&result) {
            writer
                .write_all(finish_output.as_bytes())
                .or_else(Self::check_for_writer_error)
                .unwrap();
        }

        writer.flush().or_else(Self::check_for_writer_error).unwrap();

        result
    }

    fn check_for_writer_error(error: std::io::Error) -> Result<(), std::io::Error> {
        // Do not panic when the process is killed (e.g. piping into `less`).
        if matches!(
            error.kind(),
            ErrorKind::Interrupted | ErrorKind::BrokenPipe | ErrorKind::WouldBlock
        ) {
            Ok(())
        } else {
            Err(error)
        }
    }
}

// The following from_file_path and strict_canonicalize implementations are from tower-lsp-community/tower-lsp-server
// available under the MIT License or Apache 2.0 License.
//
// Copyright (c) 2023 Eyal Kalderon
// https://github.com/tower-lsp-community/tower-lsp-server/blob/85506ddcbd108c514438e0b62e0eb858c812adcf/src/uri_ext.rs

const ASCII_SET: AsciiSet =
    // RFC3986 allows only alphanumeric characters, `-`, `.`, `_`, and `~` in the path.
    url_encoding::percent::NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~')
        // we do not want path separators to be percent-encoded
        .remove(b'/');

fn from_file_path<A: AsRef<Path>>(path: A) -> Option<String> {
    let path = path.as_ref();

    let fragment = if path.is_absolute() {
        Cow::Borrowed(path)
    } else {
        match strict_canonicalize(path) {
            Ok(path) => Cow::Owned(path),
            Err(_) => return None,
        }
    };

    if cfg!(windows) {
        // we want to write a triple-slash path for Windows paths
        // it's a shorthand for `file://localhost/C:/Windows` with the `localhost` omitted.
        let mut components = fragment.components();
        let drive = components.next();

        if let Some(drive) = drive {
            Some(format!(
                "file:///{}{}",
                drive.as_os_str().to_string_lossy().cow_replace('\\', "/"),
                // Skip encoding ":" in the drive "C:/".
                url_encoding::percent::utf8_percent_encode(
                    &components.collect::<PathBuf>().to_string_lossy().cow_replace('\\', "/"),
                    &ASCII_SET
                )
            ))
        } else {
            Some(format!(
                "file:///{}",
                url_encoding::percent::utf8_percent_encode(
                    &components.collect::<PathBuf>().to_string_lossy().cow_replace('\\', "/"),
                    &ASCII_SET
                )
            ))
        }
    } else {
        Some(format!(
            "file://{}",
            url_encoding::percent::utf8_percent_encode(&fragment.to_string_lossy(), &ASCII_SET)
        ))
    }
}

/// On Windows, rewrites the wide path prefix `\\?\C:` to `C:`
/// Source: <https://stackoverflow.com/a/70970317>
#[inline]
#[cfg(windows)]
fn strict_canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
    use std::io;

    fn impl_(path: &Path) -> std::io::Result<PathBuf> {
        let head = path.components().next().ok_or(io::Error::other("empty path"))?;
        let disk_;
        let head = if let std::path::Component::Prefix(prefix) = head {
            if let std::path::Prefix::VerbatimDisk(disk) = prefix.kind() {
                disk_ = format!("{}:", disk as char);
                Path::new(&disk_)
                    .components()
                    .next()
                    .ok_or(io::Error::other("failed to parse disk component"))?
            } else {
                head
            }
        } else {
            head
        };
        Ok(std::iter::once(head).chain(path.components().skip(1)).collect())
    }

    let canon = std::fs::canonicalize(path)?;
    impl_(&canon)
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_errors/src/service/tests.rs"]
#[cfg(test)]
mod tests;
