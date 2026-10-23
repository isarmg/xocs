
    use crate::syntax_arena::Allocator;
    use crate::syntax_tree::{ast::*, builder::AstBuilder};
    use crate::source_location::{GetSpan, SourceType, Span};

    use super::Utf8ToUtf16;

    #[test]
    fn translate_ast() {
        let allocator = Allocator::new();
        let ast = AstBuilder::new(&allocator);

        let mut program = Program::new(
            Span::new(0, 15),
            SourceType::default(),
            ";'🤨' // 🤨",
            [Comment::new(8, 15, CommentKind::Line)],
            None,
            [],
            [
                Statement::new_empty_statement(Span::new(0, 1), &ast),
                Statement::new_expression_statement(
                    Span::new(1, 7),
                    Expression::new_string_literal(Span::new(1, 7), "🤨", None, &ast),
                    &ast,
                ),
            ],
            &ast,
        );

        let span_converter = Utf8ToUtf16::new(program.source_text);
        span_converter.convert_program(&mut program);
        span_converter.convert_comments(&mut program.comments);

        assert_eq!(program.span, Span::new(0, 11));
        assert_eq!(program.body[1].span(), Span::new(1, 5));
        let Statement::ExpressionStatement(expr_stmt) = &program.body[1] else { unreachable!() };
        let Expression::StringLiteral(s) = &expr_stmt.expression else { unreachable!() };
        assert_eq!(s.span, Span::new(1, 5));
        assert_eq!(program.comments[0].span, Span::new(6, 11));

        // Check converting back from UTF-16 to UTF-8
        let convert_back = |utf16_offset: u32| {
            let mut utf8_offset = utf16_offset;
            span_converter.convert_offset_back(&mut utf8_offset);
            utf8_offset
        };

        assert_eq!(convert_back(0), 0);
        assert_eq!(convert_back(2), 2);
        assert_eq!(convert_back(4), 6);
        assert_eq!(convert_back(9), 11);
        assert_eq!(convert_back(11), 15);
    }

    #[test]
    fn translate_offsets() {
        assert_eq!('_'.len_utf8(), 1);
        assert_eq!('_'.len_utf16(), 1);
        assert_eq!('£'.len_utf8(), 2);
        assert_eq!('£'.len_utf16(), 1);
        assert_eq!('ऊ'.len_utf8(), 3);
        assert_eq!('ऊ'.len_utf16(), 1);
        assert_eq!('🤨'.len_utf8(), 4);
        assert_eq!('🤨'.len_utf16(), 2);

        let cases: &[(&str, &[(u32, u32)])] = &[
            // 1-byte
            ("_", &[(0, 0), (1, 1)]),
            // 2-byte
            ("£", &[(0, 0), (2, 1)]),
            ("£_", &[(0, 0), (2, 1), (3, 2)]),
            ("_£", &[(0, 0), (1, 1), (3, 2)]),
            ("_£_", &[(0, 0), (1, 1), (3, 2), (4, 3)]),
            ("_££_", &[(0, 0), (1, 1), (3, 2), (5, 3), (6, 4)]),
            ("_£_£_", &[(0, 0), (1, 1), (3, 2), (4, 3), (6, 4), (7, 5)]),
            // 3-byte
            ("ऊ", &[(0, 0), (3, 1)]),
            ("ऊ_", &[(0, 0), (3, 1), (4, 2)]),
            ("_ऊ", &[(0, 0), (1, 1), (4, 2)]),
            ("_ऊ_", &[(0, 0), (1, 1), (4, 2), (5, 3)]),
            ("_ऊऊ_", &[(0, 0), (1, 1), (4, 2), (7, 3), (8, 4)]),
            ("_ऊ_ऊ_", &[(0, 0), (1, 1), (4, 2), (5, 3), (8, 4), (9, 5)]),
            // 4-byte
            ("🤨", &[(0, 0), (4, 2)]),
            ("🤨_", &[(0, 0), (4, 2), (5, 3)]),
            ("_🤨", &[(0, 0), (1, 1), (5, 3)]),
            ("_🤨_", &[(0, 0), (1, 1), (5, 3), (6, 4)]),
            ("_🤨🤨_", &[(0, 0), (1, 1), (5, 3), (9, 5), (10, 6)]),
            ("_🤨_🤨_", &[(0, 0), (1, 1), (5, 3), (6, 4), (10, 6), (11, 7)]),
        ];

        // Convert cases to `Vec`
        let mut cases_vec = cases
            .iter()
            .map(|&(text, translations)| (text, translations.to_vec()))
            .collect::<Vec<_>>();

        // Create 1 long string containing 99 repeats of each test case, concatenated
        let repeats = 99u32;
        let mut texts = String::new();
        for (text, _) in cases {
            for _i in 0..repeats {
                texts.push_str(text);
            }
        }

        // Generate more test cases for each of the defined cases repeated 99 times.
        // Each case references a slice of the large `texts` string.
        // Reason we do that is so that these string slices have uneven alignments, to exercise all parts
        // of `build_translations`, which handles unaligned header/tail differently from the main body
        // of the source text.
        // The number of repeats is 99, for the same reason - to ensure each string slice begins at
        // a memory address which is not evenly aligned.
        let mut offset = 0;
        for &(text, translations) in cases {
            let end_offset = offset + text.len() * (repeats as usize);
            let repeated_text = &texts[offset..end_offset];

            let (len_utf8, len_utf16) = *translations.last().unwrap();
            assert_eq!(text.len(), len_utf8 as usize);

            let mut repeated_translations = vec![];
            for i in 0..repeats {
                for &(offset_utf8, offset_utf16) in translations {
                    repeated_translations
                        .push((offset_utf8 + len_utf8 * i, offset_utf16 + len_utf16 * i));
                }
            }

            cases_vec.push((repeated_text, repeated_translations));

            offset = end_offset;
        }

        for (text, translations) in cases_vec {
            let table = Utf8ToUtf16::new(text);
            let converter = table.converter();
            if let Some(mut converter) = converter {
                // Iterate in forwards order
                for &(utf8_offset, expected_utf16_offset) in &translations {
                    let mut utf16_offset = utf8_offset;
                    converter.convert_offset(&mut utf16_offset);
                    assert_eq!(utf16_offset, expected_utf16_offset);
                }

                // Iterate in backwards order
                for &(utf8_offset, expected_utf16_offset) in translations.iter().rev() {
                    let mut utf16_offset = utf8_offset;
                    converter.convert_offset(&mut utf16_offset);
                    assert_eq!(utf16_offset, expected_utf16_offset);
                }

                // Convert back from UTF-16 to UTF-8
                for &(expected_utf8_offset, utf16_offset) in &translations {
                    let mut utf8_offset = utf16_offset;
                    converter.convert_offset_back(&mut utf8_offset);
                    assert_eq!(utf8_offset, expected_utf8_offset);
                }
            } else {
                // No Unicode chars. All offsets should be the same.
                for &(utf8_offset, expected_utf16_offset) in &translations {
                    assert_eq!(utf8_offset, expected_utf16_offset);
                }
            }
        }
    }

    #[test]
    fn translate_offsets_after_trimmed_bom() {
        const BOM_LEN: u32 = 3;

        // Source text has the BOM trimmed off the start, but spans are still relative to the file
        // including the BOM, so the offsets in the table are too.
        let table = Utf8ToUtf16::new_with_offset("/*x*/'é'", BOM_LEN);
        let mut converter = table.converter().unwrap();

        // (UTF-8 offset including BOM, UTF-16 offset excluding BOM)
        let offsets = [(3u32, 0u32), (8, 5), (9, 6), (11, 7), (12, 8)];
        for (utf8_offset, expected_utf16_offset) in offsets {
            let mut offset = utf8_offset;
            converter.convert_offset(&mut offset);
            assert_eq!(offset, expected_utf16_offset);
        }

        for (expected_utf8_offset, utf16_offset) in offsets {
            let mut offset = utf16_offset;
            converter.convert_offset_back(&mut offset);
            assert_eq!(offset, expected_utf8_offset);
        }
    }

    #[test]
    fn translate_offsets_with_trimmed_prefix() {
        // Linter trims a BOM, or the leading newline of a Vue/Astro/Svelte fragment, off the source
        // text before building the table. The trimmed prefix contributes no UTF-16 characters, but
        // spans remain relative to the untrimmed source. So an offset in the trimmed source has to
        // convert the same way it would with nothing trimmed.
        //
        // 1 = `\n` after `<script>`, 2 = `\r\n` after `<script>`, 3 = BOM.
        const TRIMMED_LENS: [u32; 3] = [1, 2, 3];

        let sources = [
            // Non-ASCII character at offset 0, and 1 byte in. These produce a table entry whose
            // `utf8_offset` is lower than its `utf16_difference` if the trimmed prefix is not added
            // back on, which makes `convert_offset_slow` underflow - a panic in debug builds, and a
            // `u32::MAX` offset in release builds.
            "£abcd",
            "ऊabcd",
            "🤨abcd",
            "a£bcd",
            "aऊbcd",
            // A comment immediately before a 2-byte character. The comment's end offset and the
            // string's start offset are the same offset, which is why the original bug report saw
            // this on comments.
            "/*x*/'£';",
            // 2-byte, 3-byte and 4-byte characters in one source. An error of up to 3 bytes in the
            // table is invisible if all the non-ASCII characters are 4 bytes wide, because every
            // offset it corrupts then falls inside a character, and so is never the start or end of
            // a span.
            "_£_ऊ_🤨_",
        ];

        // Also test each source repeated, as a slice of one long string. The slices have uneven
        // alignments and are longer than a chunk, so between them they exercise the unaligned head,
        // the aligned body, and the tail of `build_translations`. Same trick as `translate_offsets`,
        // for the same reason.
        let repeats = 9;
        let mut texts = String::new();
        for source_text in sources {
            for _i in 0..repeats {
                texts.push_str(source_text);
            }
        }

        let mut cases = sources.to_vec();
        let mut start_offset = 0;
        for source_text in sources {
            let end_offset = start_offset + source_text.len() * repeats;
            cases.push(&texts[start_offset..end_offset]);
            start_offset = end_offset;
        }

        #[expect(clippy::cast_possible_truncation)]
        for source_text in cases {
            // UTF-16 offset of a UTF-8 offset in `source_text`, calculated independently of
            // `Utf8ToUtf16`. Only defined on character boundaries.
            let expected_utf16_offset =
                |utf8_offset: usize| source_text[..utf8_offset].encode_utf16().count() as u32;
            let boundaries =
                || (0..=source_text.len()).filter(|&offset| source_text.is_char_boundary(offset));

            for trimmed_len in TRIMMED_LENS {
                let table = Utf8ToUtf16::new_with_offset(source_text, trimmed_len);
                let mut converter = table.converter().unwrap();

                // Ascending order - how spans are converted when walking an AST straight from parser
                for utf8_offset in boundaries() {
                    let mut offset = utf8_offset as u32 + trimmed_len;
                    converter.convert_offset(&mut offset);
                    let expected = expected_utf16_offset(utf8_offset);
                    assert_eq!(
                        offset, expected,
                        "{source_text:?} trimmed {trimmed_len} at {utf8_offset}"
                    );
                }

                // Descending order, through the same converter. Takes `find_range_before`, which
                // relies on the table being sorted by `utf8_offset`.
                for utf8_offset in boundaries().rev() {
                    let mut offset = utf8_offset as u32 + trimmed_len;
                    converter.convert_offset(&mut offset);
                    let expected = expected_utf16_offset(utf8_offset);
                    assert_eq!(
                        offset, expected,
                        "{source_text:?} trimmed {trimmed_len} at {utf8_offset} (descending)"
                    );
                }

                // Back from UTF-16 to UTF-8 - the direction diagnostics and fixes from JS plugins
                // are converted in
                let converter = table.converter().unwrap();
                for utf8_offset in boundaries() {
                    let mut offset = expected_utf16_offset(utf8_offset);
                    converter.convert_offset_back(&mut offset);
                    let expected = utf8_offset as u32 + trimmed_len;
                    assert_eq!(
                        offset, expected,
                        "{source_text:?} trimmed {trimmed_len} back to {utf8_offset}"
                    );
                }

                // Offsets which are not on a character boundary have no UTF-16 offset to compare
                // against, but they must still convert exactly as they do with nothing trimmed.
                // Every source contains a non-ASCII character, so `new` always yields a converter.
                let whole_table = Utf8ToUtf16::new(source_text);
                let mut whole = whole_table.converter().unwrap();
                let mut trimmed = table.converter().unwrap();
                for utf8_offset in 0..=(source_text.len() as u32) {
                    let mut expected = utf8_offset;
                    whole.convert_offset(&mut expected);

                    let mut offset = utf8_offset + trimmed_len;
                    trimmed.convert_offset(&mut offset);

                    assert_eq!(
                        offset, expected,
                        "{source_text:?} trimmed {trimmed_len} at {utf8_offset}"
                    );
                }
            }
        }
    }

    // Check assumptions about how many UTF-16 chars result from different UTF-8 character sequences,
    // which are relied on by `build_table`
    #[test]
    fn char_lengths() {
        macro_rules! assert_utf8_bytes_eq {
            ($c:expr, $bytes:expr) => {{
                let mut buffer = [0; 4];
                let bytes = $c.encode_utf8(&mut buffer).as_bytes();
                assert!($bytes == bytes);
            }};
        }

        // All 1-byte UTF-8 character sequences = 1 x UTF-16 character.
        // First byte is 0x00 - 0x7F.
        let min_1_byte_char = char::from_u32(0).unwrap();
        assert_eq!(min_1_byte_char.len_utf8(), 1);
        assert_eq!(min_1_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_1_byte_char, [0x00]);
        let max_1_byte_char = char::from_u32(0x7F).unwrap();
        assert_eq!(max_1_byte_char.len_utf8(), 1);
        assert_eq!(max_1_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_1_byte_char, [0x7F]);

        // All 2-byte UTF-8 character sequences = 1 x UTF-16 character
        // First byte is 0xC2 - 0xDF.
        let min_2_byte_char = char::from_u32(0x80).unwrap();
        assert_eq!(min_2_byte_char.len_utf8(), 2);
        assert_eq!(min_2_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_2_byte_char, [0xC2, 0x80]);
        let max_2_byte_char = char::from_u32(0x7FF).unwrap();
        assert_eq!(max_2_byte_char.len_utf8(), 2);
        assert_eq!(max_2_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_2_byte_char, [0xDF, 0xBF]);

        // All 3-byte UTF-8 character sequences = 1 x UTF-16 character
        // First byte is 0xE0 - 0xEF.
        let min_3_byte_char = char::from_u32(0x800).unwrap();
        assert_eq!(min_3_byte_char.len_utf8(), 3);
        assert_eq!(min_3_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_3_byte_char, [0xE0, 0xA0, 0x80]);
        let max_3_byte_char = char::from_u32(0xFFFF).unwrap();
        assert_eq!(max_3_byte_char.len_utf8(), 3);
        assert_eq!(max_3_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_3_byte_char, [0xEF, 0xBF, 0xBF]);

        // All 4-byte UTF-8 character sequences = 2 x UTF-16 characters
        // First byte is 0xF0 - 0xF4.
        let min_4_byte_char = char::from_u32(0x10000).unwrap();
        assert_eq!(min_4_byte_char.len_utf8(), 4);
        assert_eq!(min_4_byte_char.len_utf16(), 2);
        assert_utf8_bytes_eq!(min_4_byte_char, [0xF0, 0x90, 0x80, 0x80]);
        let max_4_byte_char = char::MAX;
        assert_eq!(max_4_byte_char.len_utf8(), 4);
        assert_eq!(max_4_byte_char.len_utf16(), 2);
        assert_utf8_bytes_eq!(max_4_byte_char, [0xF4, 0x8F, 0xBF, 0xBF]);
    }
