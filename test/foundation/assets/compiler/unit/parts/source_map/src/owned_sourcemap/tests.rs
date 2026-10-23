
    use super::*;

    const JSON: &str = r#"{
        "version": 3,
        "file": "out.js",
        "sourceRoot": "root",
        "names": ["n0"],
        "sources": ["a.js"],
        "sourcesContent": ["AC"],
        "mappings": "AAAAA",
        "ignoreList": [0],
        "debugId": "dbg"
    }"#;

    #[test]
    fn accessors() {
        let sm = OwnedSourceMap::from_json_string(JSON).unwrap();
        assert_eq!(sm.get_file(), Some("out.js"));
        assert_eq!(sm.get_source_root(), Some("root"));
        assert_eq!(sm.get_ignore_list(), Some(&[0][..]));
        assert_eq!(sm.get_debug_id(), Some("dbg"));
        assert_eq!(sm.get_names().collect::<Vec<_>>(), vec!["n0"]);
        assert_eq!(sm.get_sources().collect::<Vec<_>>(), vec!["a.js"]);
        assert_eq!(sm.get_source_contents().collect::<Vec<_>>(), vec![Some("AC")]);
        assert_eq!(sm.get_name(0), Some("n0"));
        assert_eq!(sm.get_source(0), Some("a.js"));
        assert_eq!(sm.get_source_content(0), Some("AC"));
        assert_eq!(sm.get_source_and_content(0), Some(("a.js", "AC")));
        assert!(sm.get_token(0).is_some());
        assert_eq!(sm.get_tokens().count(), 1);
        assert!(sm.get_source_view_token(0).is_some());
        assert_eq!(sm.get_source_view_tokens().count(), 1);
    }

    #[test]
    fn mutators() {
        let mut sm = OwnedSourceMap::from_json_string(JSON).unwrap();
        sm.set_file("new.js");
        assert_eq!(sm.get_file(), Some("new.js"));
        sm.set_sources(vec!["b.js"]);
        assert_eq!(sm.get_source(0), Some("b.js"));
        sm.set_source_contents(vec![Some("new content")]);
        assert_eq!(sm.get_source_content(0), Some("new content"));
        sm.set_ignore_list(vec![]);
        assert_eq!(sm.get_ignore_list(), Some(&[][..]));
        sm.set_debug_id("newdbg");
        assert_eq!(sm.get_debug_id(), Some("newdbg"));
    }

    #[test]
    fn lookup() {
        let sm = OwnedSourceMap::from_json_string(JSON).unwrap();
        let table = sm.generate_lookup_table();
        assert!(sm.lookup_token(&table, 0, 0).is_some());
        assert!(sm.lookup_source_view_token(&table, 0, 0).is_some());
    }

    #[test]
    fn serialize() {
        let sm = OwnedSourceMap::from_json_string(JSON).unwrap();
        assert_eq!(sm.to_json().file.as_deref(), Some("out.js"));
        assert!(sm.to_json_string().contains("\"file\":\"out.js\""));
        assert!(sm.to_data_url().starts_with("data:application/json;charset=utf-8;base64,"));
    }

    #[test]
    fn from_json_value() {
        let json = SourceMap::from_json_string(JSON).unwrap().to_json();
        let sm = OwnedSourceMap::from_json(json).unwrap();
        assert_eq!(sm.get_file(), Some("out.js"));
    }

    #[test]
    fn conversions_and_inner_access() {
        let inner = SourceMap::from_json_string(JSON).unwrap().into_owned();

        // `new` + borrow accessors.
        let owned = OwnedSourceMap::new(inner.clone());
        assert_eq!(owned.as_source_map().get_file(), Some("out.js"));

        // `From<SourceMap>` + mutable inner access.
        let mut from_sm = OwnedSourceMap::from(inner.clone());
        from_sm.as_source_map_mut().set_file("z.js");
        assert_eq!(from_sm.get_file(), Some("z.js"));

        // `Deref` / `DerefMut` to the inner `SourceMap`.
        let deref: &SourceMap<'static> = &owned;
        assert_eq!(deref.get_file(), Some("out.js"));
        let mut owned_mut = owned.clone();
        let deref_mut: &mut SourceMap<'static> = &mut owned_mut;
        deref_mut.set_file("deref.js");
        assert_eq!(owned_mut.get_file(), Some("deref.js"));

        // `into_inner` + `From<OwnedSourceMap>` + `SourceMap::into_owned_sourcemap`.
        let back: SourceMap<'static> = owned.into_inner();
        let round: SourceMap<'static> = OwnedSourceMap::from(back.clone()).into();
        assert_eq!(round.get_file(), Some("out.js"));
        assert_eq!(back.into_owned_sourcemap().get_file(), Some("out.js"));
    }

    #[test]
    fn parts_roundtrip() {
        let owned = OwnedSourceMap::from_json_string(JSON).unwrap();
        let parts = owned.into_parts();
        assert_eq!(parts.file.as_deref(), Some("out.js"));
        assert_eq!(parts.ignore_list.as_deref(), Some(&[0][..]));
        let rebuilt = OwnedSourceMap::from_parts(parts);
        assert_eq!(rebuilt.get_file(), Some("out.js"));
        assert_eq!(rebuilt.get_source(0), Some("a.js"));
        assert_eq!(rebuilt.get_ignore_list(), Some(&[0][..]));
    }
