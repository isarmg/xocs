
    use super::SourceMap;

    #[test]
    fn from_source_map() {
        let mut inner = crate::source_map::SourceMap::new(
            Some("out.js".into()),
            vec!["n0".into()],
            Some("root".into()),
            vec!["a.js".into()],
            vec![Some("content".into())],
            vec![].into_boxed_slice(),
            None,
        );
        inner.set_ignore_list(vec![0]);

        let napi: SourceMap = inner.into();
        assert_eq!(napi.version, 3);
        assert_eq!(napi.file.as_deref(), Some("out.js"));
        assert_eq!(napi.source_root.as_deref(), Some("root"));
        assert_eq!(napi.names, vec!["n0".to_string()]);
        assert_eq!(napi.sources, vec!["a.js".to_string()]);
        assert_eq!(napi.sources_content, Some(vec!["content".to_string()]));
        assert_eq!(napi.ignore_list, Some(vec![0]));
    }

    #[test]
    fn null_source_content_becomes_empty_string() {
        // `sourcesContent` entries that are `null` map to an empty string,
        // because the napi shape uses `Vec<String>` (no inner `Option`).
        let inner = crate::source_map::SourceMap::new(
            None,
            vec![],
            None,
            vec!["a.js".into(), "b.js".into()],
            vec![Some("x".into()), None],
            vec![].into_boxed_slice(),
            None,
        );
        let napi: SourceMap = inner.into();
        assert_eq!(napi.sources_content, Some(vec!["x".to_string(), String::new()]));
    }

    #[test]
    fn from_owned_source_map() {
        let owned = crate::source_map::OwnedSourceMap::from_json_string(
            r#"{"version":3,"sources":["a.js"],"mappings":"","x_google_ignoreList":[0]}"#,
        )
        .unwrap();
        let napi: SourceMap = owned.into();
        assert_eq!(napi.version, 3);
        assert_eq!(napi.sources, ["a.js"]);
        assert_eq!(napi.sources_content, None);
        assert_eq!(napi.ignore_list, Some(vec![0]));
    }
