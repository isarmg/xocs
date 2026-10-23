
    use std::sync::Arc;

    use crate::source_location::Span;

    use super::*;
    use crate::compiler_errors::{Error, GraphicalTheme, NamedSource, OxcDiagnostic};

    #[test]
    fn batch_render_matches_individual_reports() {
        let source =
            Arc::new(NamedSource::new("input.ts", "first();\nsecond();\nthird();\nfourth();\n"));
        let other_source = Arc::new(NamedSource::new("other.ts", "alpha();\nbeta();\n"));
        let diagnostics: Vec<Error> = vec![
            OxcDiagnostic::warn("later")
                .with_label(Span::new(29, 35))
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::error("multi-label")
                .with_labels([Span::new(0, 5), Span::new(19, 24)])
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::warn("earlier")
                .with_label(Span::new(9, 15))
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::error("without source").into(),
            OxcDiagnostic::warn("different source")
                .with_label(Span::new(9, 13))
                .with_source_code(other_source),
        ];
        let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none()).with_links(false);

        let mut expected = String::new();
        for diagnostic in &diagnostics {
            handler.render_report(&mut expected, diagnostic.as_ref()).unwrap();
        }

        let mut actual = String::new();
        handler
            .render_reports(
                &mut actual,
                diagnostics.iter().map(|diagnostic| diagnostic.as_ref() as &dyn Diagnostic),
            )
            .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "exhaustive equivalence check over safe text wrapping code; interpreting every \
                  textwrap case under Miri takes more than 16 minutes"
    )]
    fn fill_fast_path_matches_textwrap() {
        let texts = [
            "",
            "short diagnostic",
            "trailing spaces   ",
            "  leading spaces",
            "two  inner  spaces",
            "Café 火",
            "combining e\u{301}",
            "emoji 🐂",
            "\u{1b}[31mstyled text\u{1b}[0m",
            "\u{1b}]8;;https://example.com\u{1b}\\linked\u{1b}]8;;\u{1b}\\",
            "\u{1b}]0;title\u{7}visible",
            "control\tcharacters\u{7}",
            "incomplete \u{1b}[31",
            "first\nsecond",
        ];
        for width in 0..32 {
            for initial_indent in ["", "  ", "  help: ", "\u{1b}[31m  × \u{1b}[0m"] {
                for text in texts {
                    let opts = textwrap::Options::new(width)
                        .initial_indent(initial_indent)
                        .subsequent_indent("    ");
                    assert_eq!(
                        GraphicalReportHandler::fill(text, opts.clone()),
                        textwrap::fill(text, opts.clone()),
                        "width={width}, indent={initial_indent:?}, text={text:?}"
                    );
                    let mut output = String::new();
                    GraphicalReportHandler::write_fill(&mut output, text, opts.clone()).unwrap();
                    assert_eq!(
                        output,
                        textwrap::fill(text, opts),
                        "streaming: width={width}, indent={initial_indent:?}, text={text:?}"
                    );
                }
            }
        }
    }
