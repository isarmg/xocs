
    use crate::syntax_arena::Allocator;
    use crate::source_location::SourceType;

    use crate::parser::config::NoTokensLexerConfig;

    use super::super::{Kind, Lexer, UniquePromise};

    #[test]
    fn template_literal_linebreaks() {
        // Note: These cases don't include all `\n`s because that requires no unescaping
        let escapes = [
            // 1 return
            ("\r", "\n"),
            ("\r\n", "\n"),
            // 2 returns
            ("\r\r", "\n\n"),
            ("\r\r\n", "\n\n"),
            ("\r\n\r", "\n\n"),
            ("\r\n\n", "\n\n"),
            ("\n\r", "\n\n"),
            ("\n\r\n", "\n\n"),
            ("\r\n\r\n", "\n\n"),
            // 3 returns
            ("\r\r\r", "\n\n\n"),
            ("\n\r\r", "\n\n\n"),
            ("\n\n\r", "\n\n\n"),
            ("\r\n\r\r", "\n\n\n"),
            ("\r\r\n\r", "\n\n\n"),
            ("\r\r\r\n", "\n\n\n"),
            ("\r\n\r\n\r", "\n\n\n"),
            ("\r\r\n\r\n", "\n\n\n"),
            ("\r\n\r\n\r\n", "\n\n\n"),
        ];

        #[expect(clippy::items_after_statements, clippy::needless_pass_by_value)]
        fn run_test(source_text: String, expected_escaped: String, is_only_part: bool) {
            let allocator = Allocator::default();
            let unique = UniquePromise::new_for_tests_and_benchmarks();
            let mut lexer = Lexer::new(
                &allocator,
                &source_text,
                SourceType::default(),
                NoTokensLexerConfig,
                unique,
            );
            let token = lexer.next_token();
            assert_eq!(
                token.kind(),
                if is_only_part { Kind::NoSubstitutionTemplate } else { Kind::TemplateHead }
            );
            let escaped = lexer.escaped_templates[&token.start()];
            assert_eq!(escaped, Some(expected_escaped.as_str()));
        }

        for (source_fragment, escaped_fragment) in escapes {
            run_test(format!("`{source_fragment}`"), escaped_fragment.to_string(), true);
            run_test(format!("`{source_fragment}${{x}}`"), escaped_fragment.to_string(), false);
            run_test(format!("`{source_fragment}abc`"), format!("{escaped_fragment}abc"), true);
            run_test(
                format!("`{source_fragment}abc${{x}}`"),
                format!("{escaped_fragment}abc"),
                false,
            );
            run_test(format!("`abc{source_fragment}`"), format!("abc{escaped_fragment}"), true);
            run_test(
                format!("`abc{source_fragment}${{x}}`"),
                format!("abc{escaped_fragment}"),
                false,
            );
            run_test(
                format!("`abc{source_fragment}def{source_fragment}ghi`"),
                format!("abc{escaped_fragment}def{escaped_fragment}ghi"),
                true,
            );
            run_test(
                format!("`abc{source_fragment}def{source_fragment}ghi${{x}}`"),
                format!("abc{escaped_fragment}def{escaped_fragment}ghi"),
                false,
            );
            run_test(
                format!("`{source_fragment}abc{source_fragment}def{source_fragment}`"),
                format!("{escaped_fragment}abc{escaped_fragment}def{escaped_fragment}"),
                true,
            );
            run_test(
                format!("`{source_fragment}abc{source_fragment}def{source_fragment}${{x}}`"),
                format!("{escaped_fragment}abc{escaped_fragment}def{escaped_fragment}"),
                false,
            );
        }
    }
