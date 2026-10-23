
    use std::{
        path::PathBuf,
        sync::{
            Arc, Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use crate::compiler_errors::{
        Error, NamedSource, OxcDiagnostic,
        reporter::{DiagnosticReporter, DiagnosticResult},
        service::from_file_path,
    };

    use super::DiagnosticService;

    fn with_schema(path: &str) -> String {
        const EXPECTED_SCHEMA: &str = if cfg!(windows) { "file:///" } else { "file://" };
        format!("{EXPECTED_SCHEMA}{path}")
    }

    #[test]
    #[cfg(windows)]
    fn test_idempotent_canonicalization() {
        use crate::compiler_errors::service::strict_canonicalize;
        use std::path::Path;

        let lhs = strict_canonicalize(Path::new(".")).unwrap();
        let rhs = strict_canonicalize(&lhs).unwrap();
        assert_eq!(lhs, rhs);
    }

    #[test]
    #[cfg(unix)]
    fn test_path_to_uri() {
        let paths = [
            PathBuf::from("/some/path/to/file.txt"),
            PathBuf::from("/some/path/to/file with spaces.txt"),
            PathBuf::from("/some/path/[[...rest]]/file.txt"),
            PathBuf::from("/some/path/to/файл.txt"),
            PathBuf::from("/some/path/to/文件.txt"),
        ];

        let expected = [
            with_schema("/some/path/to/file.txt"),
            with_schema("/some/path/to/file%20with%20spaces.txt"),
            with_schema("/some/path/%5B%5B...rest%5D%5D/file.txt"),
            with_schema("/some/path/to/%D1%84%D0%B0%D0%B9%D0%BB.txt"),
            with_schema("/some/path/to/%E6%96%87%E4%BB%B6.txt"),
        ];

        for (path, expected) in paths.iter().zip(expected) {
            let uri = from_file_path(path).unwrap();
            assert_eq!(uri.clone(), expected);
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_path_to_uri_windows() {
        let paths = [
            PathBuf::from("C:\\some\\path\\to\\file.txt"),
            PathBuf::from("C:\\some\\path\\to\\file with spaces.txt"),
            PathBuf::from("C:\\some\\path\\[[...rest]]\\file.txt"),
            PathBuf::from("C:\\some\\path\\to\\файл.txt"),
            PathBuf::from("C:\\some\\path\\to\\文件.txt"),
        ];

        let expected = [
            with_schema("C:/some/path/to/file.txt"),
            with_schema("C:/some/path/to/file%20with%20spaces.txt"),
            with_schema("C:/some/path/%5B%5B...rest%5D%5D/file.txt"),
            with_schema("C:/some/path/to/%D1%84%D0%B0%D0%B9%D0%BB.txt"),
            with_schema("C:/some/path/to/%E6%96%87%E4%BB%B6.txt"),
        ];

        for (path, expected) in paths.iter().zip(expected) {
            let uri = from_file_path(path).unwrap();
            assert_eq!(uri, expected);
        }
    }

    #[test]
    fn preserves_long_lines_for_single_line_reporters() {
        #[derive(Default)]
        struct LongLineReporter {
            fallback_enabled: bool,
        }

        impl DiagnosticReporter for LongLineReporter {
            fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
                None
            }

            fn supports_minified_file_fallback(&self) -> bool {
                self.fallback_enabled
            }

            fn render_error(&mut self, error: Error) -> Option<String> {
                let message = error.to_string();
                if message == "original diagnostic" {
                    Some(format!("{}\n", "x".repeat(1200)))
                } else {
                    Some(format!("{message}\n"))
                }
            }
        }
        let (mut service, sender) =
            DiagnosticService::new(Box::new(LongLineReporter { fallback_enabled: false }));
        sender.send(vec![OxcDiagnostic::warn("original diagnostic").into()]).unwrap();
        drop(sender);

        let mut output = Vec::new();
        service.run(&mut output);
        let output = String::from_utf8(output).unwrap();

        assert_eq!(output, format!("{}\n", "x".repeat(1200)));
    }

    struct BatchReporter(Arc<Mutex<Vec<usize>>>);

    impl DiagnosticReporter for BatchReporter {
        fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
            None
        }

        fn render_errors(&mut self, errors: Vec<Error>, emit: &mut dyn FnMut(&str)) {
            self.0.lock().unwrap().push(errors.len());
            let output =
                errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>().join(",");
            emit(&output);
        }

        fn supports_minified_file_fallback(&self) -> bool {
            false
        }

        fn render_error(&mut self, _error: Error) -> Option<String> {
            None
        }
    }

    #[test]
    fn renders_a_filtered_batch() {
        let batch_sizes = Arc::new(Mutex::new(Vec::new()));
        let (service, sender) =
            DiagnosticService::new(Box::new(BatchReporter(Arc::clone(&batch_sizes))));
        let mut service = service.with_quiet(true);
        sender
            .send(vec![
                OxcDiagnostic::warn("hidden warning").into(),
                OxcDiagnostic::error("visible error").into(),
            ])
            .unwrap();
        drop(sender);

        let mut output = Vec::new();
        let result = service.run(&mut output);

        assert_eq!(String::from_utf8(output).unwrap(), "visible error");
        assert_eq!(*batch_sizes.lock().unwrap(), [1]);
        assert_eq!(result.warnings_count(), 1);
        assert_eq!(result.errors_count(), 1);
    }

    struct MinifiedBatchReporter(Arc<AtomicUsize>);

    impl DiagnosticReporter for MinifiedBatchReporter {
        fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
            None
        }

        fn render_error(&mut self, error: Error) -> Option<String> {
            let message = error.to_string();
            if message == "File is too long to fit on the screen" {
                Some(format!("{message}: {}\n", error.help().unwrap_or_default()))
            } else {
                self.0.fetch_add(1, Ordering::Relaxed);
                Some(if message == "second" {
                    format!("{}\n", "x".repeat(1200))
                } else {
                    format!("{message}\n")
                })
            }
        }
    }

    #[test]
    fn stops_an_oversized_batch_at_the_minified_file_diagnostic() {
        let rendered = Arc::new(AtomicUsize::new(0));
        let (mut service, sender) =
            DiagnosticService::new(Box::new(MinifiedBatchReporter(Arc::clone(&rendered))));
        sender
            .send(vec![
                OxcDiagnostic::warn("first")
                    .with_source_code(NamedSource::new("normal.js", "source")),
                OxcDiagnostic::warn("second")
                    .with_source_code(NamedSource::new("minified.js", "source")),
                OxcDiagnostic::warn("third").into(),
            ])
            .unwrap();
        drop(sender);

        let mut output = Vec::new();
        let result = service.run(&mut output);

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "first\nFile is too long to fit on the screen: minified.js seems like a minified file\n"
        );
        assert_eq!(rendered.load(Ordering::Relaxed), 2);
        assert_eq!(result.warnings_count(), 3);
    }
