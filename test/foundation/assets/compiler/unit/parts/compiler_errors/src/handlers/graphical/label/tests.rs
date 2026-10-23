
    use super::*;

    #[test]
    fn plain_fast_paths_match_display_output() {
        let underline = Underline { padding: 3, left: 2, marker: '|', right: 4, line: '^' };
        let mut output = String::new();
        underline.write(&mut output).unwrap();
        assert_eq!(output, underline.to_string());

        let theme = crate::compiler_errors::GraphicalTheme::none();
        for render_mode in
            [LabelRenderMode::SingleLine, LabelRenderMode::BlockFirst, LabelRenderMode::BlockRest]
        {
            let label = LabelText {
                chars: &theme.characters,
                label: "plain label",
                style: Style::Plain,
                render_mode,
            };
            output.clear();
            label.write_unstyled(&mut output).unwrap();
            assert_eq!(output, label.to_string());
        }
    }

    #[test]
    fn repeated_chars_match_standard_output() {
        for c in [' ', '─', '^', 'x', '🐂'] {
            for count in [0, 1, 31, 32, 33, 64, 65] {
                let mut output = String::new();
                if let Some((chunk, char_len)) = match c {
                    ' ' => Some((SPACES, 1)),
                    '─' => Some((UNICODE_BARS, '─'.len_utf8())),
                    '^' => Some((ASCII_CARETS, 1)),
                    _ => None,
                } {
                    write_repeated_chunk(&mut output, chunk, char_len, count).unwrap();
                } else {
                    write_repeated_char(&mut output, c, count).unwrap();
                }
                assert_eq!(output, c.to_string().repeat(count));
            }
        }
    }

    #[test]
    fn padding_matches_standard_output_across_chunk_threshold() {
        for count in [0, 1, MIN_CHUNKED_CHARS - 1, MIN_CHUNKED_CHARS, 64, 65] {
            let mut output = String::new();
            write_padding(&mut output, count).unwrap();
            assert_eq!(output, " ".repeat(count));
        }
    }
