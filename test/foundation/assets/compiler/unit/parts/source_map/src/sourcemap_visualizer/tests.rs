
    use std::borrow::Cow;

    use super::*;
    use crate::source_map::Token;

    #[test]
    fn get_url() {
        let sm =
            SourceMap::from_json_string(r#"{"version":3,"sources":[],"names":[],"mappings":""}"#)
                .unwrap();
        let url = SourcemapVisualizer::new("code", &sm).get_url();
        assert!(url.starts_with("https://evanw.github.io/source-map-visualization/#"));
    }

    #[test]
    fn no_source_contents() {
        // Sources present but no `sourcesContent` at all.
        let sm = SourceMap::from_json_string(
            r#"{"version":3,"sources":["a.js"],"names":[],"mappings":"AAAA"}"#,
        )
        .unwrap();
        assert_eq!(SourcemapVisualizer::new("a", &sm).get_text(), "[no source contents]\n");
    }

    #[test]
    fn skips_tokens_without_resolvable_source() {
        // First token has no source id, second points at a source whose content
        // is `None`; both are skipped, and only the third (valid) token prints.
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Borrowed("a.js"), Cow::Borrowed("b.js")],
            vec![Some(Cow::Borrowed("hello\n")), None],
            vec![
                Token::new(0, 0, 0, 0, None, None),
                Token::new(0, 1, 0, 0, Some(1), None),
                Token::new(0, 2, 0, 0, Some(0), None),
            ]
            .into_boxed_slice(),
            None,
        );
        let text = SourcemapVisualizer::new("hello\n", &sm).get_text();
        assert!(text.contains("- a.js"), "{text}");
        assert!(!text.contains("- b.js"), "{text}");
    }

    #[test]
    fn handles_crlf_line_endings() {
        // CRLF source content exercises the `\r\n` peek branch in the line table.
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Borrowed("a.js")],
            vec![Some(Cow::Borrowed("aa\r\nbb\r\n"))],
            vec![Token::new(0, 0, 0, 0, Some(0), None), Token::new(1, 0, 1, 0, Some(0), None)]
                .into_boxed_slice(),
            None,
        );
        let text = SourcemapVisualizer::new("aa\r\nbb\r\n", &sm).get_text();
        assert!(text.contains("- a.js"), "{text}");
    }

    #[test]
    fn handles_unicode_line_separators() {
        for separator in ['\u{2028}', '\u{2029}'] {
            let code = format!("é{separator}😀");
            let sm = SourceMap::new(
                None,
                vec![],
                None,
                vec!["a.js".into()],
                vec![Some(code.as_str().into())],
                vec![Token::new(0, 0, 0, 0, Some(0), None), Token::new(1, 0, 1, 0, Some(0), None)]
                    .into_boxed_slice(),
                None,
            );
            let first_line = format!("é{separator}");
            assert_eq!(
                SourcemapVisualizer::new(&code, &sm).get_text(),
                format!(
                    "- a.js\n(0:0) {first_line:?} --> (0:0) {first_line:?}\n(1:0) \"😀\" --> (1:0) \"😀\"\n"
                )
            );
        }
    }

    #[test]
    fn handles_columns_inside_surrogate_pairs() {
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec!["a.js".into()],
            vec![Some("😀x".into())],
            vec![Token::new(0, 0, 0, 0, Some(0), None), Token::new(0, 1, 0, 1, Some(0), None)]
                .into_boxed_slice(),
            None,
        );
        assert_eq!(
            SourcemapVisualizer::new("😀x", &sm).get_text(),
            "- a.js\n(0:0) \"�\" --> (0:0) \"�\"\n(0:1) \"�x\" --> (0:1) \"�x\"\n"
        );
    }

    #[test]
    fn skips_token_with_out_of_range_source() {
        // A token references a source id past the end of `sources`; the
        // visualizer skips it via the `get_source` guard rather than panicking.
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Borrowed("a.js")],
            vec![Some(Cow::Borrowed("aa\n"))],
            vec![Token::new(0, 0, 0, 0, Some(5), None)].into_boxed_slice(),
            None,
        );
        assert_eq!(SourcemapVisualizer::new("aa\n", &sm).get_text(), "");
    }
