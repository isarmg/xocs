
    use super::*;

    #[test]
    fn builder_ascii() {
        assert_mapping("", &[(0, 0, 0)]);
        assert_mapping("a", &[(0, 0, 0), (1, 0, 1)]);
        assert_mapping("\n", &[(0, 0, 0), (1, 1, 0)]);
        assert_mapping("a\n", &[(0, 0, 0), (1, 0, 1), (2, 1, 0)]);
        assert_mapping("\na", &[(0, 0, 0), (1, 1, 0), (2, 1, 1)]);
        assert_mapping(
            "ab\ncd\n\nef",
            &[
                (0, 0, 0),
                (1, 0, 1),
                (2, 0, 2),
                (3, 1, 0),
                (4, 1, 1),
                (5, 1, 2),
                (6, 2, 0),
                (7, 3, 0),
                (8, 3, 1),
                (9, 3, 2),
            ],
        );

        assert_mapping("\r", &[(0, 0, 0), (1, 1, 0)]);
        assert_mapping("\r\r", &[(0, 0, 0), (1, 1, 0), (2, 2, 0)]);
        assert_mapping("a\ra", &[(0, 0, 0), (1, 0, 1), (2, 1, 0), (3, 1, 1)]);

        assert_mapping("\r\n", &[(0, 0, 0), (1, 0, 1), (2, 1, 0)]);
        assert_mapping("\r\n\r\n", &[(0, 0, 0), (1, 0, 1), (2, 1, 0), (3, 1, 1), (4, 2, 0)]);
        assert_mapping("a\r\na", &[(0, 0, 0), (1, 0, 1), (2, 0, 2), (3, 1, 0), (4, 1, 1)]);
    }

    #[test]
    fn builder_unicode() {
        assert_mapping("Ö", &[(0, 0, 0), (2, 0, 1)]);
        assert_mapping("ÖÖ", &[(0, 0, 0), (2, 0, 1), (4, 0, 2)]);
        assert_mapping("Ö\n", &[(0, 0, 0), (2, 0, 1), (3, 1, 0)]);
        assert_mapping("ÖÖ\n", &[(0, 0, 0), (2, 0, 1), (4, 0, 2), (5, 1, 0)]);
        assert_mapping("\nÖ", &[(0, 0, 0), (1, 1, 0), (3, 1, 1)]);
        assert_mapping("\nÖÖ", &[(0, 0, 0), (1, 1, 0), (3, 1, 1), (5, 1, 2)]);
        assert_mapping("Ö\nÖ", &[(0, 0, 0), (2, 0, 1), (3, 1, 0), (5, 1, 1)]);
        assert_mapping("\nÖÖ\n", &[(0, 0, 0), (1, 1, 0), (3, 1, 1), (5, 1, 2), (6, 2, 0)]);
        assert_mapping("Ö\ra", &[(0, 0, 0), (2, 0, 1), (3, 1, 0), (4, 1, 1)]);
        assert_mapping("Ö\r\na", &[(0, 0, 0), (2, 0, 1), (3, 0, 2), (4, 1, 0), (5, 1, 1)]);
    }

    #[test]
    fn builder_with_unordered_position() {
        assert_mapping("\na\nb", &[(4, 2, 1), (0, 0, 0), (1, 1, 0), (2, 1, 1), (3, 2, 0)]);
    }

    fn assert_mapping(source: &str, mappings: &[(u32, u32, u32)]) {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), source);
        for (position, expected_line, expected_col) in mappings.iter().copied() {
            let (line, col) = builder.search_original_line_and_column(position);
            assert_eq!(
                builder.search_original_line_and_column(position),
                (expected_line, expected_col),
                "Incorrect mapping for '{source}' - position {position} = line {line}, column {col}"
            );
        }
    }

    #[test]
    fn add_source_mapping() {
        fn create_mappings(source: &str, line: u32, column: u32) {
            let mut builder = SourcemapBuilder::new(Path::new("x.js"), source);
            let output: Vec<u8> = source.as_bytes().into();
            for (i, _ch) in source.char_indices() {
                #[expect(clippy::cast_possible_truncation)]
                builder.add_source_mapping(&output, i as u32, None);
                assert!(
                    builder.generated_line == line && builder.generated_column == column,
                    "Incorrect generated mapping for '{source}' ({:?}) starting at {i} - line {}, column {}",
                    source.as_bytes(),
                    builder.generated_line,
                    builder.generated_column
                );
                assert_eq!(builder.last_generated_update, source.len());
            }
        }

        create_mappings("", 0, 0);
        create_mappings("abc", 0, 3);
        create_mappings("\n", 1, 0);
        create_mappings("\n\n\n", 3, 0);
        create_mappings("\r", 1, 0);
        create_mappings("\r\r\r", 3, 0);
        create_mappings("\r\n", 1, 0);
        create_mappings("\r\n\r\n\r\n", 3, 0);
        create_mappings("\nabc", 1, 3);
        create_mappings("abc\n", 1, 0);
        create_mappings("\rabc", 1, 3);
        create_mappings("abc\r", 1, 0);
        create_mappings("\r\nabc", 1, 3);
        create_mappings("abc\r\n", 1, 0);
        create_mappings("ÖÖ\nÖ\nÖÖÖ", 2, 3);
        create_mappings("\u{2028}", 1, 0);
        create_mappings("\u{2029}", 1, 0);
        create_mappings("a\u{2028}b", 1, 1);
        create_mappings("a\u{2029}b", 1, 1);
        create_mappings("`\u{2028}`", 1, 1);
        create_mappings("`\u{2029}`", 1, 1);

        // Long lines (>= 8 bytes) to exercise the SWAR boring-byte fast-forward in
        // `update_generated_line_and_column`, with line breaks / unicode at various offsets.
        create_mappings("abcdefghijklmnop", 0, 16);
        create_mappings("abcdefgh\nijklmnop", 1, 8);
        create_mappings("abcdefghijklmnop\n", 1, 0);
        create_mappings("\nabcdefghijklmnop", 1, 16);
        create_mappings("aaaaaaaaaa\r\nbbbbbbbbbb", 1, 10);
        create_mappings("aaaaaaaaaa\rbbbbbbbbbb", 1, 10);
        create_mappings("aaaaaaaaÖbbbbbbbb", 0, 17);
        create_mappings("aaaaaaaaaaaaaaaa\u{2028}bbbbbbbbbbbbbbbb", 1, 16);
        create_mappings("abcdefghijklmnopqrstuvwxyz0123456789", 0, 36);
        // Line breaks straddling the 8-byte SWAR word boundary (`\r` at index 7).
        create_mappings("aaaaaaa\r\nbbbbbbbb", 1, 8);
        create_mappings("aaaaaaa\rbbbbbbbb", 1, 8);
    }

    #[test]
    fn add_source_mapping_for_name() {
        let output = b"ac";
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "ab");
        builder.add_source_mapping_for_name(output, Span::new(0, 1), "a");
        builder.add_source_mapping_for_name(output, Span::new(1, 2), "c");
        let sm = builder.into_sourcemap();
        // The name `a` not change.
        assert_eq!(
            sm.get_source_view_token(0_u32)
                .as_ref()
                .and_then(crate::source_map::SourceViewToken::get_name),
            None
        );
        // The name `b` -> `c`, save `b` to token.
        assert_eq!(
            sm.get_source_view_token(1_u32)
                .as_ref()
                .and_then(crate::source_map::SourceViewToken::get_name),
            Some("b")
        );
    }

    #[test]
    fn add_source_mapping_for_name_from_private_identifier() {
        // A transform can print a plain identifier where the source had a private one.
        // The `#` is part of the region the mapping describes, so it is part of the name recorded.
        let output = b"foo";
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "#foo");
        builder.add_source_mapping_for_name(output, Span::new(0, 4), "foo");
        let sm = builder.into_sourcemap();
        assert_eq!(
            sm.get_source_view_token(0_u32)
                .as_ref()
                .and_then(crate::source_map::SourceViewToken::get_name),
            Some("#foo")
        );
    }

    #[test]
    fn add_source_mapping_for_private_name() {
        let output = b"#a#c";
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "#a#b");
        builder.add_source_mapping_for_private_name(output, Span::new(0, 2), "a");
        builder.add_source_mapping_for_private_name(output, Span::new(2, 4), "c");
        let sm = builder.into_sourcemap();
        // The name `#a` not change.
        assert_eq!(
            sm.get_source_view_token(0_u32)
                .as_ref()
                .and_then(crate::source_map::SourceViewToken::get_name),
            None
        );
        // The name `#b` -> `#c`, save `#b` to token, `#` included.
        assert_eq!(
            sm.get_source_view_token(1_u32)
                .as_ref()
                .and_then(crate::source_map::SourceViewToken::get_name),
            Some("#b")
        );
    }

    #[test]
    fn add_source_mapping_for_private_name_from_plain_identifier() {
        // A transform can print a private identifier where the source had a plain one.
        // The token still differs from the source text, so the original name is recorded.
        let output = b"#foo";
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "ffoo");
        builder.add_source_mapping_for_private_name(output, Span::new(0, 4), "foo");
        let sm = builder.into_sourcemap();
        assert_eq!(
            sm.get_source_view_token(0_u32)
                .as_ref()
                .and_then(crate::source_map::SourceViewToken::get_name),
            Some("ffoo")
        );
    }

    #[test]
    fn add_source_mapping_for_unordered_position() {
        let output = b"";
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), "ab");
        builder.add_source_mapping(output, 1, None);
        builder.add_source_mapping(output, 0, None);
        let sm = builder.into_sourcemap();
        assert_eq!(sm.get_tokens().count(), 2);
    }

    static SOURCE: &str = "a\nbc\ndef";
    static MAPPINGS: &[(u32, u32)] = &[
        (0, 0), // 'a'
        (0, 1), // '\n'
        (1, 0), // 'b'
        (1, 1), // 'c'
        (1, 2), // '\n'
        (2, 0), // 'd'
        (2, 1), // 'e'
        (2, 2), // 'f'
        (2, 3), // EOF
    ];

    #[test]
    fn test_search_original_line_and_column_sequential() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), SOURCE);

        #[expect(clippy::cast_possible_truncation)]
        for (pos, (expected_line, expected_col)) in MAPPINGS.iter().copied().enumerate() {
            let (line, col) = builder.search_original_line_and_column(pos as u32);
            assert_eq!((line, col), (expected_line, expected_col), "Mismatch at position {pos}");
        }
    }

    #[test]
    fn test_search_original_line_and_column_reverse_sequential() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), SOURCE);

        #[expect(clippy::cast_possible_truncation)]
        for (pos, (expected_line, expected_col)) in MAPPINGS.iter().copied().enumerate().rev() {
            let (line, col) = builder.search_original_line_and_column(pos as u32);
            assert_eq!((line, col), (expected_line, expected_col), "Mismatch at position {pos}");
        }
    }

    #[test]
    fn test_search_original_line_and_column_non_sequential() {
        let mut builder = SourcemapBuilder::new(Path::new("x.js"), SOURCE);

        let indexes = [8, 0, 7, 1, 6, 2, 5, 3, 4];

        for pos in indexes {
            let (expected_line, expected_col) = MAPPINGS[pos as usize];
            let (line, col) = builder.search_original_line_and_column(pos);
            assert_eq!((line, col), (expected_line, expected_col), "Mismatch at position {pos}");
        }
    }
