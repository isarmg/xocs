
    use crate::compiler_errors::{Error, NamedSource, OxcDiagnostic};

    use super::Info;

    #[test]
    fn includes_source_name_for_unlabelled_diagnostic() {
        let error: Error = OxcDiagnostic::error("Something went wrong")
            .with_source_code(NamedSource::new("file.js", ""));

        assert_eq!(Info::new(&error).filename, "file.js");
    }
