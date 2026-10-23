
    use super::*;

    #[test]
    fn build() {
        let mut builder = SourceMapBuilder::default();
        builder.set_source_and_content("baz.js", "");
        builder.add_name("x");
        builder.set_file("file");

        let sm = builder.into_sourcemap();
        assert_eq!(sm.get_source(0), Some("baz.js"));
        assert_eq!(sm.get_name(0), Some("x"));
        assert_eq!(sm.get_file(), Some("file"));

        let expected = r#"{"version":3,"file":"file","names":["x"],"sources":["baz.js"],"sourcesContent":[""],"mappings":""}"#;
        assert_eq!(expected, sm.to_json_string());
    }

    #[test]
    fn dedup() {
        let mut builder = SourceMapBuilder::default();
        let id_a = builder.add_name("foo");
        let id_b = builder.add_name("bar");
        let id_a_again = builder.add_name("foo");
        let id_b_again = builder.add_name("bar");
        assert_eq!(id_a, id_a_again);
        assert_eq!(id_b, id_b_again);
        assert_ne!(id_a, id_b);

        let src_a = builder.add_source_and_content("a.js", "content a");
        let src_b = builder.add_source_and_content("b.js", "content b");
        let src_a_again = builder.add_source_and_content("a.js", "different content (ignored)");
        assert_eq!(src_a, src_a_again);
        assert_ne!(src_a, src_b);

        let sm = builder.into_sourcemap();
        assert_eq!(sm.get_names().collect::<Vec<_>>(), vec!["foo", "bar"]);
        assert_eq!(sm.get_sources().collect::<Vec<_>>(), vec!["a.js", "b.js"]);
        // Source content for the first add wins; the second add returns the
        // existing id without overwriting.
        assert_eq!(sm.get_source_content(src_a), Some("content a"));
    }

    #[test]
    fn add_token_and_chunks() {
        let mut builder = SourceMapBuilder::default();
        let name_id = builder.add_name("n");
        let source_id = builder.add_source_and_content("s.js", "src");
        builder.add_token(0, 0, 0, 0, Some(source_id), Some(name_id));
        builder.add_token(0, 4, 0, 4, Some(source_id), None);
        builder.set_token_chunks(vec![TokenChunk::new(0, 2, 0, 0, 0, 0, 0, 0)]);

        let sm = builder.into_sourcemap();
        assert!(sm.token_chunks.is_some());
        assert_eq!(sm.get_tokens().count(), 2);
        assert_eq!(sm.get_token(0), Some(Token::new(0, 0, 0, 0, Some(0), Some(0))));
    }

    #[test]
    fn into_owned_sourcemap() {
        let mut builder = SourceMapBuilder::default();
        builder.set_file("f.js");
        let name_id = builder.add_name("n");
        let source_id = builder.add_source_and_content("s.js", "src");
        builder.add_token(0, 0, 0, 0, Some(source_id), Some(name_id));
        builder.set_token_chunks(vec![TokenChunk::new(0, 1, 0, 0, 0, 0, 0, 0)]);

        let owned = builder.into_owned_sourcemap();
        assert_eq!(owned.get_file(), Some("f.js"));
        assert_eq!(owned.get_name(0), Some("n"));
        assert_eq!(owned.get_source(0), Some("s.js"));
        assert_eq!(owned.get_source_content(0), Some("src"));
        assert_eq!(owned.get_tokens().count(), 1);
    }

    #[test]
    fn into_owned_sourcemap_without_chunks() {
        // No token chunks set: exercises the `None` branch of the chunk shrink.
        let owned = SourceMapBuilder::default().into_owned_sourcemap();
        assert_eq!(owned.get_tokens().count(), 0);
        assert!(owned.as_source_map().token_chunks.is_none());
    }
