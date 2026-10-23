
    use super::*;

    fn build_test_inputs() -> [SourceMap<'static>; 3] {
        [
            (vec!["foo", "foo2"], "foo.js", Token::new(1, 1, 1, 1, Some(0), Some(0))),
            (vec!["bar"], "bar.js", Token::new(1, 1, 1, 1, Some(0), Some(0))),
            (vec!["abc"], "abc.js", Token::new(1, 2, 2, 2, Some(0), Some(0))),
        ]
        .map(|(names, source, token)| SourceMap {
            names: names.into_iter().map(Cow::Borrowed).collect(),
            sources: vec![source.into()],
            tokens: Box::new([token]),
            ..SourceMap::default()
        })
    }

    fn assert_test_result(concat_sm: SourceMap<'_>) {
        let expected = SourceMap::new(
            None,
            vec![
                Cow::Borrowed("foo"),
                Cow::Borrowed("foo2"),
                Cow::Borrowed("bar"),
                Cow::Borrowed("abc"),
            ],
            None,
            vec![Cow::Borrowed("foo.js"), Cow::Borrowed("bar.js"), Cow::Borrowed("abc.js")],
            vec![],
            vec![
                Token::new(1, 1, 1, 1, Some(0), Some(0)),
                Token::new(3, 1, 1, 1, Some(1), Some(2)),
                Token::new(3, 2, 2, 2, Some(2), Some(3)),
            ]
            .into_boxed_slice(),
            None,
        );

        assert_eq!(concat_sm.tokens, expected.tokens);
        assert_eq!(concat_sm.sources, expected.sources);
        assert_eq!(concat_sm.names, expected.names);
        assert_eq!(
            concat_sm.token_chunks,
            Some(vec![
                TokenChunk::new(0, 1, 0, 0, 0, 0, 0, 0,),
                TokenChunk::new(1, 2, 1, 1, 1, 1, 0, 0,),
                TokenChunk::new(2, 3, 3, 1, 1, 1, 2, 1,)
            ])
        );

        assert_eq!(expected.to_json().mappings, concat_sm.to_json().mappings);
    }

    #[test]
    fn add_sourcemap_in_loop() {
        let [sm1, sm2, sm3] = build_test_inputs();
        let inputs = [(&sm1, 0u32), (&sm2, 2), (&sm3, 2)];
        let mut builder = ConcatSourceMapBuilder::default();
        for (sourcemap, line_offset) in inputs {
            builder.add_sourcemap(sourcemap, line_offset);
        }
        assert_test_result(builder.into_sourcemap());
    }

    #[test]
    fn from_sourcemaps() {
        let [sm1, sm2, sm3] = build_test_inputs();
        let builder = ConcatSourceMapBuilder::from_sourcemaps(&[(&sm1, 0), (&sm2, 2), (&sm3, 2)]);
        assert_test_result(builder.into_sourcemap());
    }

    #[test]
    fn add_sourcemap_owned() {
        // Moving owned maps in must produce the same result as borrowing them.
        let [sm1, sm2, sm3] = build_test_inputs();
        let mut builder = ConcatSourceMapBuilder::default();
        builder.add_sourcemap_owned(sm1, 0);
        builder.add_sourcemap_owned(sm2, 2);
        builder.add_sourcemap_owned(sm3, 2);
        assert_test_result(builder.into_sourcemap());
    }

    #[test]
    fn from_owned_sourcemaps() {
        let [sm1, sm2, sm3] = build_test_inputs();
        let builder =
            ConcatSourceMapBuilder::from_owned_sourcemaps(vec![(sm1, 0), (sm2, 2), (sm3, 2)]);
        assert_test_result(builder.into_sourcemap());
    }

    #[test]
    fn owned_moves_strings_into_owned_sourcemap() {
        // Two owned maps with owned content; the move-in path must preserve every string and
        // renumber `other`'s ids/lines, ending up as a `'static` map with no data lost.
        let a = SourceMap::new(
            None,
            vec![Cow::Owned("name_a".to_string())],
            None,
            vec![Cow::Owned("a.js".to_string())],
            vec![Some(Cow::Owned("a content".to_string()))],
            vec![Token::new(0, 0, 0, 0, Some(0), Some(0))].into_boxed_slice(),
            None,
        );
        let b = SourceMap::new(
            None,
            vec![Cow::Owned("name_b".to_string())],
            None,
            vec![Cow::Owned("b.js".to_string())],
            vec![Some(Cow::Owned("b content".to_string()))],
            vec![Token::new(0, 0, 0, 0, Some(0), Some(0))].into_boxed_slice(),
            None,
        );

        let owned = ConcatSourceMapBuilder::from_owned_sourcemaps(vec![(a, 0), (b, 5)])
            .into_owned_sourcemap();

        assert_eq!(owned.get_sources().collect::<Vec<_>>(), vec!["a.js", "b.js"]);
        assert_eq!(owned.get_names().collect::<Vec<_>>(), vec!["name_a", "name_b"]);
        assert_eq!(owned.get_source_content(0), Some("a content"));
        assert_eq!(owned.get_source_content(1), Some("b content"));
        // `b`'s token is shifted by the line offset and renumbered onto the combined ids.
        assert_eq!(owned.get_token(1), Some(Token::new(5, 0, 0, 0, Some(1), Some(1))));
    }

    #[test]
    fn owned_translates_after_tokenless_map() {
        // A first map that contributes a source but no tokens leaves `self.tokens` empty while the
        // source offset advances; the next map must still renumber its ids (not take the memcpy path).
        let tokenless = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Owned("a.js".to_string())],
            vec![Some(Cow::Owned("a content".to_string()))],
            vec![].into_boxed_slice(),
            None,
        );
        let with_token = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Owned("b.js".to_string())],
            vec![Some(Cow::Owned("b content".to_string()))],
            // Source id 0 within its own map; after concat it must point at "b.js" (combined index 1).
            vec![Token::new(0, 0, 0, 0, Some(0), None)].into_boxed_slice(),
            None,
        );

        let map =
            ConcatSourceMapBuilder::from_owned_sourcemaps(vec![(tokenless, 0), (with_token, 0)])
                .into_sourcemap();

        assert_eq!(map.get_sources().collect::<Vec<_>>(), vec!["a.js", "b.js"]);
        assert_eq!(map.get_token(0).unwrap().get_source_id(), Some(1));
        assert_eq!(map.get_source_content(1), Some("b content"));
    }

    #[test]
    fn owned_pads_missing_source_contents() {
        // First map has a source but no `sourcesContent`; the second map's content must stay attached
        // to its own source rather than shifting onto the first map's source id.
        let no_content = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Owned("a.js".to_string())],
            vec![],
            vec![Token::new(0, 0, 0, 0, Some(0), None)].into_boxed_slice(),
            None,
        );
        let with_content = SourceMap::new(
            None,
            vec![],
            None,
            vec![Cow::Owned("b.js".to_string())],
            vec![Some(Cow::Owned("b content".to_string()))],
            vec![Token::new(0, 0, 0, 0, Some(0), None)].into_boxed_slice(),
            None,
        );

        let map =
            ConcatSourceMapBuilder::from_owned_sourcemaps(vec![(no_content, 0), (with_content, 1)])
                .into_sourcemap();

        assert_eq!(map.get_sources().collect::<Vec<_>>(), vec!["a.js", "b.js"]);
        assert_eq!(map.get_source_content(0), None);
        assert_eq!(map.get_source_content(1), Some("b content"));
    }

    #[test]
    fn keeps_distinct_boundary_tokens() {
        // Boundary dedup only drops a token when it is *identical* to the
        // previous map's last token (same dst/src position and source/name id
        // after offsetting). Here the seam tokens differ, so nothing is
        // dropped — the counterpart to `dedups_identical_boundary_token`.
        let sm1 = SourceMap::new(
            None,
            vec![Cow::Borrowed("name1")],
            None,
            vec![Cow::Borrowed("file1.js")],
            vec![],
            vec![
                Token::new(1, 1, 1, 1, Some(0), Some(0)),
                Token::new(2, 5, 2, 5, Some(0), Some(0)),
            ]
            .into_boxed_slice(),
            None,
        );

        // sm2 has different source_id/name_id after offset, so won't deduplicate
        let sm2 = SourceMap::new(
            None,
            vec![Cow::Borrowed("name2")],
            None,
            vec![Cow::Borrowed("file2.js")],
            vec![],
            vec![
                Token::new(2, 5, 2, 5, Some(0), Some(0)), // Different source/name after offset
                Token::new(3, 10, 3, 10, Some(0), Some(0)),
            ]
            .into_boxed_slice(),
            None,
        );

        let mut builder = ConcatSourceMapBuilder::default();
        builder.add_sourcemap(&sm1, 0);
        builder.add_sourcemap(&sm2, 0);

        let concat_sm = builder.into_sourcemap();

        // Should have 4 tokens (no deduplication because source_id/name_id differ)
        assert_eq!(concat_sm.tokens.len(), 4);
    }

    #[test]
    fn dedups_identical_boundary_token() {
        // When the second map contributes no sources/names, its first token can
        // translate to exactly the first map's last token. The boundary dedup
        // must then drop the duplicate rather than emit it twice.
        let map1 = SourceMap::new(
            None,
            vec![],
            None,
            vec![],
            vec![],
            vec![Token::new(0, 0, 0, 0, None, None), Token::new(2, 3, 0, 0, None, None)]
                .into_boxed_slice(),
            None,
        );
        let map2 = SourceMap::new(
            None,
            vec![],
            None,
            vec![],
            vec![],
            vec![Token::new(0, 3, 0, 0, None, None), Token::new(1, 9, 0, 0, None, None)]
                .into_boxed_slice(),
            None,
        );

        let mut builder = ConcatSourceMapBuilder::default();
        builder.add_sourcemap(&map1, 0);
        // `map2`'s first token shifted by `line_offset` 2 equals `map1`'s last token.
        builder.add_sourcemap(&map2, 2);
        let concat_sm = builder.into_sourcemap();

        // 2 from map1 + 1 from map2 (its first token was deduplicated at the seam).
        assert_eq!(concat_sm.tokens.len(), 3);
        assert_eq!(concat_sm.tokens[1], Token::new(2, 3, 0, 0, None, None));
        assert_eq!(concat_sm.tokens[2], Token::new(3, 9, 0, 0, None, None));
    }
