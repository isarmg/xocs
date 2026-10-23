
    use std::path::Path;

    use crate::syntax_tree::ast::{
        CommentKind, Expression, Statement, TSNamespaceDeclarationBody, TSNamespaceDeclarationKind,
    };
    use crate::source_location::GetSpan;

    use super::*;

    #[test]
    fn parse_program_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.program.is_empty());
        assert!(ret.diagnostics.is_empty());
        assert!(!ret.is_flow_language);
    }

    #[test]
    fn parse_expression_smoke_test() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "a";
        let expr = Parser::new(&allocator, source, source_type).parse_expression().unwrap();
        assert!(matches!(expr, Expression::Identifier(_)));
    }

    #[test]
    fn parse_expression_rejects_trailing_tokens() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        for source in ["a b", "a;", "let x = 1"] {
            assert!(Parser::new(&allocator, source, source_type).parse_expression().is_err());
        }
    }

    #[test]
    fn flow_error() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let sources = [
            "// @flow\nasdf adsf",
            "/* @flow */\n asdf asdf",
            "/**
             * @flow
             */
             asdf asdf
             ",
            "/* @flow */ super;",
        ];
        for source in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.is_flow_language);
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(ret.diagnostics.first().unwrap().to_string(), "Flow is not supported");
        }
    }

    #[test]
    fn ts_module_declaration() {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(Path::new("module.ts")).unwrap();
        let source =
            "declare module 'test'; namespace Foo.Bar {} module Baz {} declare module 'raw' {}";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert!(ret.diagnostics.is_empty());

        let Statement::TSExternalModuleDeclaration(external) = &ret.program.body[0] else {
            panic!("expected external module declaration");
        };
        assert_eq!(external.id.value, "test");
        assert!(external.body.is_none());
        assert!(external.declare);

        let Statement::TSNamespaceDeclaration(namespace) = &ret.program.body[1] else {
            panic!("expected namespace declaration");
        };
        assert_eq!(namespace.id.name, "Foo");
        assert_eq!(namespace.kind, TSNamespaceDeclarationKind::Namespace);
        let TSNamespaceDeclarationBody::TSNamespaceDeclaration(inner) = &namespace.body else {
            panic!("expected dotted namespace declaration");
        };
        assert_eq!(inner.id.name, "Bar");

        let Statement::TSNamespaceDeclaration(module) = &ret.program.body[2] else {
            panic!("expected identifier module declaration");
        };
        assert_eq!(module.id.name, "Baz");
        assert_eq!(module.kind, TSNamespaceDeclarationKind::Module);

        let Statement::TSExternalModuleDeclaration(external) = &ret.program.body[3] else {
            panic!("expected external module declaration");
        };
        assert_eq!(external.id.value, "raw");
        assert!(external.body.is_some());
        assert!(external.declare);
    }

    #[test]
    fn directives() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let sources = [
            ("import x from 'foo'; 'use strict';", 2),
            ("export {x} from 'foo'; 'use strict';", 2),
            (";'use strict';", 2),
        ];
        for (source, body_length) in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.program.directives.is_empty(), "{source}");
            assert_eq!(ret.program.body.len(), body_length, "{source}");
        }
    }

    #[test]
    fn v8_intrinsics() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        {
            let source = "%DebugPrint('Raging against the Dying Light')";
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = Parser::new(&allocator, source, source_type).with_options(opts).parse();
            assert!(ret.diagnostics.is_empty());

            if let Some(Statement::ExpressionStatement(expr_stmt)) = ret.program.body.first() {
                if let Expression::V8IntrinsicExpression(expr) = &expr_stmt.expression {
                    assert_eq!(expr.span().source_text(source), source);
                } else {
                    panic!("Expected V8IntrinsicExpression");
                }
            } else {
                panic!("Expected ExpressionStatement");
            }
        }
        {
            let source = "%DebugPrint(...illegalSpread)";
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = Parser::new(&allocator, source, source_type).with_options(opts).parse();
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(
                ret.diagnostics[0].to_string(),
                "V8 runtime calls cannot have spread elements as arguments"
            );
        }
        {
            let source = "%DebugPrint('~~')";
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert_eq!(ret.diagnostics.len(), 1);
            assert_eq!(ret.diagnostics[0].to_string(), "Unexpected token");
        }
        {
            // https://github.com/oxc-project/oxc/issues/12121
            let source = "interface Props extends %enuProps {}";
            let source_type = SourceType::default().with_typescript(true);
            // Should not panic whether `allow_v8_intrinsics` is set or not.
            let opts = ParseOptions { allow_v8_intrinsics: true, ..ParseOptions::default() };
            let ret = Parser::new(&allocator, source, source_type).with_options(opts).parse();
            assert_eq!(ret.diagnostics.len(), 1);
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert_eq!(ret.diagnostics.len(), 1);
        }
    }

    #[test]
    fn comments() {
        let allocator = Allocator::default();
        let source_type = SourceType::default().with_typescript(true);
        let sources = [
            ("// line comment", CommentKind::Line),
            ("/* line comment */", CommentKind::SingleLineBlock),
            (
                "type Foo = ( /* Require properties which are not generated automatically. */ 'bar')",
                CommentKind::SingleLineBlock,
            ),
        ];
        for (source, kind) in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            let comments = &ret.program.comments;
            assert_eq!(comments.len(), 1, "{source}");
            assert_eq!(comments.first().unwrap().kind, kind, "{source}");
        }
    }

    #[test]
    fn hashbang() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let source = "#!/usr/bin/node\n;";
        let ret = Parser::new(&allocator, source, source_type).parse();
        assert_eq!(ret.program.hashbang.unwrap().value.as_str(), "/usr/bin/node");
    }

    #[test]
    fn unambiguous() {
        let allocator = Allocator::default();
        let source_type = SourceType::unambiguous();
        assert!(source_type.is_unambiguous());
        let sources = ["import x from 'foo';", "export {x} from 'foo';", "import.meta"];
        for source in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.program.source_type.is_module());
        }

        let sources = ["", "import('foo')"];
        for source in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(ret.program.source_type.is_script());
        }
    }

    #[test]
    fn binary_file() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();

        // U+FFFD as a standalone token — file appears to be binary
        let ret = Parser::new(&allocator, "\u{FFFD}", source_type).parse();
        assert!(ret.program.is_empty());
        assert_eq!(ret.diagnostics.len(), 1);
        assert_eq!(ret.diagnostics[0].to_string(), "File appears to be binary.");

        // U+FFFD inside string literals — should parse fine
        let ret = Parser::new(&allocator, "\"oops \u{FFFD} oops\";", source_type).parse();
        assert!(!ret.program.is_empty());
        assert!(ret.diagnostics.is_empty());
    }

    #[test]
    fn memory_leak() {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let sources = ["2n", ";'1234567890123456789012345678901234567890'"];
        for source in sources {
            let ret = Parser::new(&allocator, source, source_type).parse();
            assert!(!ret.program.body.is_empty());
        }
    }

    // Source with length MAX_LEN + 1 fails to parse.
    // Skip this test on 32-bit systems as impossible to allocate a string longer than `isize::MAX`.
    // Also skip running under Miri since it takes so long.
    #[cfg(target_pointer_width = "64")]
    #[cfg(not(miri))]
    #[test]
    fn overlong_source() {
        use std::{
            alloc::{self, Layout},
            ptr::NonNull,
            slice, str,
        };

        /// A string that has a length of `MAX_LEN + 1`, and is entirely zeros.
        ///
        /// We need to create a `&str` with `MAX_LEN + 1` length, but don't want to write 4 GiB of data,
        /// as it's too slow. This type uses `alloc_zeroed` which on most platforms will just create zeroed pages
        /// without actually writing any data, and so is much faster.
        struct ZeroedString {
            ptr: NonNull<u8>,
        }

        impl ZeroedString {
            const LEN: usize = MAX_LEN + 1;
            const PAGE_SIZE: usize = 4096;
            const LAYOUT: Layout = match Layout::from_size_align(Self::LEN, Self::PAGE_SIZE) {
                Ok(layout) => layout,
                Err(_) => panic!("Failed to create layout"),
            };

            fn new() -> Self {
                // SAFETY: `LAYOUT` is valid and non-zero size.
                let ptr = unsafe { alloc::alloc_zeroed(Self::LAYOUT) };
                let Some(ptr) = NonNull::new(ptr) else {
                    panic!("Failed to allocate {} bytes", Self::LEN);
                };
                Self { ptr }
            }

            fn as_str(&self) -> &str {
                // SAFETY: `self.ptr` is pointer to start of `LEN` initialized and zeroed bytes.
                // A slice consisting entirely of zeros is valid UTF-8.
                unsafe {
                    str::from_utf8_unchecked(slice::from_raw_parts(self.ptr.as_ptr(), Self::LEN))
                }
            }
        }

        impl Drop for ZeroedString {
            fn drop(&mut self) {
                // SAFETY: `self.ptr` is address of an allocation made with `LAYOUT`
                unsafe { alloc::dealloc(self.ptr.as_ptr(), Self::LAYOUT) };
            }
        }

        // Create long source text (MAX_LEN + 1 bytes)
        let zeroed_string = ZeroedString::new();
        let source_text = zeroed_string.as_str();

        // Attempt to parse the source text
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, SourceType::default()).parse();

        // Parsing should fail
        assert!(ret.program.is_empty());
        assert!(ret.fatal_error);
        assert_eq!(ret.diagnostics.len(), 1);
        assert_eq!(
            ret.diagnostics.first().unwrap().to_string(),
            "Source length exceeds 4 GiB limit"
        );
    }

    // Source with length MAX_LEN parses OK.
    // This test takes over 1 minute on an M1 Macbook Pro unless compiled in release mode.
    // `not(debug_assertions)` is a proxy for detecting release mode.
    // Also skip running under Miri since it takes so long.
    #[cfg(not(debug_assertions))]
    #[cfg(not(miri))]
    #[test]
    fn legal_length_source() {
        // Build a string MAX_LEN bytes long which doesn't take too long to parse
        let head = "const x = 1;\n/*";
        let foot = "*/\nconst y = 2;\n";
        let mut source = "x".repeat(MAX_LEN);
        source.replace_range(..head.len(), head);
        source.replace_range(MAX_LEN - foot.len().., foot);
        assert_eq!(source.len(), MAX_LEN);

        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &source, SourceType::default()).parse();
        assert!(!ret.fatal_error);
        assert!(ret.diagnostics.is_empty());
        assert_eq!(ret.program.body.len(), 2);
    }

    // Where the lexer cannot lex any further it emits `Undetermined`, and no `Eof` follows.
    // That final token ends the input just as `Eof` does, so it is discarded, and discarding it
    // must not take the real token before it with it.
    #[test]
    fn tokens_when_lexing_ends_in_error() {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, "foo();\n'unterminated", SourceType::default())
            .with_config(config::TokensParserConfig)
            .parse();

        assert!(!ret.fatal_error);
        assert_eq!(ret.diagnostics.len(), 1);
        assert_eq!(ret.diagnostics.first().unwrap().to_string(), "Unterminated string");

        let tokens = ret
            .tokens
            .iter()
            .map(|token| (token.kind(), token.start(), token.end()))
            .collect::<Vec<_>>();
        assert_eq!(
            tokens,
            [
                (Kind::Ident, 0, 3),
                (Kind::LParen, 3, 4),
                (Kind::RParen, 4, 5),
                (Kind::Semicolon, 5, 6),
            ]
        );
    }

    #[test]
    fn tokens_after_unambiguous_await_reparse() {
        for source in [
            "await /x/u; export {};",
            // Reparsing merges `/a/`, but splits `/b/g`, increasing the token count.
            "await /a/ /b/g; export {};",
            "before(); await /a/ /b/g; between(); await /x/u; export {}; tail();",
            "before(); await /x/u; after(); export {}; tail();",
            "before(); await /x/u; between(); await /y/g; export {}; tail();",
            "await /x/u\nawait /y/g\nexport {};",
            "await\nawait /x/u; export {};",
            "before(); await\nawait /x/u; between(); await\nawait /y/g; export {}; tail();",
            "const x = await /x/u; between(); const y = await /y/g; export {};",
            "/before/.test(await /x/u); /after/.test('after'); export {};",
            "await /x/u, import.meta;",
            "#!/usr/bin/env node\n'use strict';\nawait /* regexp */ /x/u; export {};",
            "export {}; await /x/u;",
        ] {
            let allocator = Allocator::default();
            let parse = |source_type| {
                Parser::new(&allocator, source, source_type)
                    .with_config(config::TokensParserConfig)
                    .parse()
            };
            let unambiguous = parse(SourceType::unambiguous());
            let module = parse(SourceType::mjs());
            assert_eq!(unambiguous.tokens.as_slice(), module.tokens.as_slice(), "{source}");
        }
    }

    // A fatal error fast-forwards the lexer to end of file. Re-lexing the `}` of a template
    // substitution after that would derive a token position from the moved cursor, and so
    // overwrite an unrelated token in the collected stream.
    #[test]
    fn tokens_when_template_substitution_fails() {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, "`${}`", SourceType::default())
            .with_config(config::TokensParserConfig)
            .parse();

        assert!(ret.fatal_error);
        assert!(ret.tokens.is_empty());
    }
