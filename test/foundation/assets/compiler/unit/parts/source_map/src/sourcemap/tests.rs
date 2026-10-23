
    use super::*;

    #[test]
    fn lookup_token() {
        let input = r#"{
            "version": 3,
            "sources": ["coolstuff.js"],
            "sourceRoot": "x",
            "names": ["x","alert"],
            "mappings": "AAAA,GAAIA,GAAI,EACR,IAAIA,GAAK,EAAG,CACVC,MAAM"
        }"#;
        let sm = SourceMap::from_json_string(input).unwrap();
        let lookup_table = sm.generate_lookup_table();
        assert_eq!(
            sm.lookup_source_view_token(&lookup_table, 0, 0).unwrap().to_tuple(),
            (Some("coolstuff.js"), 0, 0, None)
        );
        assert_eq!(
            sm.lookup_source_view_token(&lookup_table, 0, 3).unwrap().to_tuple(),
            (Some("coolstuff.js"), 0, 4, Some("x"))
        );
        assert_eq!(
            sm.lookup_source_view_token(&lookup_table, 0, 24).unwrap().to_tuple(),
            (Some("coolstuff.js"), 2, 8, None)
        );

        // Lines continue out to infinity
        assert_eq!(
            sm.lookup_source_view_token(&lookup_table, 0, 1000).unwrap().to_tuple(),
            (Some("coolstuff.js"), 2, 8, None)
        );

        assert!(sm.lookup_source_view_token(&lookup_table, 1000, 0).is_none());
        // Plain `lookup_token` (not the source-view wrapper) on the same table.
        assert!(sm.lookup_token(&lookup_table, 0, 0).is_some());
    }

    #[test]
    fn source_view_token() {
        let sm = SourceMap::new(
            None,
            vec![Cow::Borrowed("foo")],
            None,
            vec![Cow::Borrowed("foo.js")],
            vec![],
            vec![Token::new(1, 1, 1, 1, Some(0), Some(0))].into_boxed_slice(),
            None,
        );
        let mut source_view_tokens = sm.get_source_view_tokens();
        assert_eq!(
            source_view_tokens.next().unwrap().to_tuple(),
            (Some("foo.js"), 1, 1, Some("foo"))
        );
        // Indexed source-view token accessor, including out-of-range.
        assert!(sm.get_source_view_token(0).is_some());
        assert!(sm.get_source_view_token(99).is_none());
    }

    #[test]
    fn mut_sourcemap() {
        let mut sm = SourceMap::default();
        sm.set_file("index.js");
        sm.set_sources(vec!["foo.js"]);
        sm.set_source_contents(vec![Some("foo")]);

        assert_eq!(sm.get_file(), Some("index.js"));
        assert_eq!(sm.get_source(0), Some("foo.js"));
        assert_eq!(sm.get_source_content(0), Some("foo"));
    }

    #[test]
    fn from_json_value() {
        // `SourceMap::from_json` consumes an already-deserialized `JSONSourceMap`,
        // a different path from the borrowed `from_json_string`.
        let json = SourceMap::from_json_string(
            r#"{"version":3,"sources":["a.js"],"names":[],"mappings":"AAAA"}"#,
        )
        .unwrap()
        .to_json();
        let sm = SourceMap::from_json(json).unwrap();
        assert_eq!(sm.get_source(0), Some("a.js"));
    }

    #[test]
    fn to_data_url() {
        let sm =
            SourceMap::from_json_string(r#"{"version":3,"sources":[],"names":[],"mappings":""}"#)
                .unwrap();
        assert!(sm.to_data_url().starts_with("data:application/json;charset=utf-8;base64,"));
    }

    #[test]
    fn source_and_content() {
        let sm = SourceMap::from_json_string(
            r#"{"version":3,"sources":["a.js"],"sourcesContent":["CONTENT"],"names":[],"mappings":"AAAA"}"#,
        )
        .unwrap();
        assert_eq!(sm.get_source_and_content(0), Some(("a.js", "CONTENT")));
        // Out-of-range source id short-circuits before reading content.
        assert_eq!(sm.get_source_and_content(99), None);

        // Source present but its content is absent: the second `?` returns `None`.
        let no_content = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Borrowed("a.js")],
            vec![],
            vec![].into_boxed_slice(),
            None,
        );
        assert_eq!(no_content.get_source(0), Some("a.js"));
        assert_eq!(no_content.get_source_and_content(0), None);
    }

    #[test]
    fn parts_roundtrip() {
        let sm = SourceMap::from_json_string(
            r#"{"version":3,"file":"f.js","sources":["a.js"],"names":["n"],"mappings":"AAAAA"}"#,
        )
        .unwrap();
        let parts = sm.into_parts();
        assert_eq!(parts.file.as_deref(), Some("f.js"));
        // Reassemble via `from_parts` and via the `From<SourceMapParts>` impl.
        let rebuilt = SourceMap::from_parts(parts);
        assert_eq!(rebuilt.get_file(), Some("f.js"));
        let from_parts: SourceMap = rebuilt.into_parts().into();
        assert_eq!(from_parts.get_source(0), Some("a.js"));
    }

    #[test]
    fn empty_lookup_table() {
        // No tokens => empty table, and lookups return `None`.
        let sm = SourceMap::default();
        assert!(sm.generate_lookup_table().is_empty());
        assert!(sm.lookup_token(&[], 0, 0).is_none());
    }

    #[test]
    fn lookup_before_first_token() {
        // The only token starts at column 5; looking up column 0 falls before
        // it, exercising the `checked_sub(1)` underflow guard in
        // `greatest_lower_bound`.
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Borrowed("a.js")],
            vec![],
            vec![Token::new(0, 5, 0, 0, Some(0), None)].into_boxed_slice(),
            None,
        );
        let table = sm.generate_lookup_table();
        assert!(sm.lookup_token(&table, 0, 0).is_none());
    }

    #[test]
    fn lookup_with_duplicate_columns() {
        // Several tokens share the same generated position (line 0, col 7). A
        // lookup there finds an exact match, then walks back over the equal-key
        // run in `greatest_lower_bound` and returns the *first* duplicate.
        let mut tokens = vec![Token::new(0, 0, 9, 9, Some(0), None)];
        for src_line in 1..=5 {
            tokens.push(Token::new(0, 7, src_line, 0, Some(0), None));
        }
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Borrowed("a.js")],
            vec![],
            tokens.into_boxed_slice(),
            None,
        );
        let table = sm.generate_lookup_table();
        // The earliest token at column 7 wins (src_line 1).
        assert_eq!(sm.lookup_token(&table, 0, 7).unwrap().get_src_line(), 1);
    }
