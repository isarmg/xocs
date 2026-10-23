
    use crate::syntax_arena::{Allocator, ArenaBox};
    use crate::syntax_tree::ast::{ImportDeclarationSpecifier, ImportOrExportKind, ImportPhase, Statement};
    use crate::source_location::{GetSpan, SourceType, Span};

    use crate::parser::Parser;
    #[test]
    fn test_parse_import_declaration() {
        let src = "import foo from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "foo");
        });

        let src = "import type foo from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "foo");
        });

        let src = "import type from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "type");
        });

        let src = "import type from from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "from");
            assert!(matches!(specifiers[0], ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));
        });

        let src = "import type type from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "type");
            assert!(matches!(specifiers[0], ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));
        });

        let src = "import type { type } from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "type");
        });

        let src = "import './a'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_none());
        });

        let src = "import type, { bar } from 'foo';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "type");
            assert_eq!(specifiers[1].name(), "bar");
        });

        let src = "import { foo, bar } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "foo");
            assert_eq!(specifiers[1].name(), "bar");
        });

        let src = "import type { foo, bar } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "foo");
            assert_eq!(specifiers[1].name(), "bar");
        });

        let src = "import type { from, type } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 2);
            assert_eq!(specifiers[0].name(), "from");
            assert_eq!(specifiers[1].name(), "type");
        });

        let src = "import defaultItem, { type from, type } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 3);
            assert_eq!(specifiers[0].name(), "defaultItem");
            assert_eq!(specifiers[1].name(), "from");
            assert_eq!(specifiers[2].name(), "type");
        });

        let src = "import { type as as } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = &specifiers[0] {
                assert_eq!(specifier.local.name, "as");
                assert_eq!(specifier.imported.name(), "type");
            } else {
                panic!("Expected ImportSpecifier, found: {:?}", specifiers[0]);
            }
        });

        let src = "import { type as as x } from 'baz';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert!(decl.specifiers.is_some());
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = &specifiers[0] {
                assert_eq!(specifier.import_kind, ImportOrExportKind::Type);
                assert_eq!(specifier.imported.name(), "as");
                // The first `as` (14-16) is the imported name; the second `as` (17-19) is the keyword
                assert_eq!(specifier.imported.span(), Span::new(14, 16));
                assert_eq!(specifier.local.name, "x");
                assert_eq!(specifier.local.span, Span::new(20, 21));
            } else {
                panic!("Expected ImportSpecifier, found: {:?}", specifiers[0]);
            }
        });

        let src = "import * as foo from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "foo");
        });

        let src = "import type * as foo from 'bar';";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Type);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "foo");
        });

        let src = "import defer * as ns from 'x'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Defer));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "ns");
        });

        let src = "import source x from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Source));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "x");
        });

        let src = "import source from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, None);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "source");
        });

        let src = "import source from from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Source));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "from");
        });

        let src = "import source as from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Source));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "as");
        });

        let src = "import source source from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, Some(ImportPhase::Source));
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "source");
        });

        let src = "import defer from 'source'";
        parse_and_assert_import_declarations(src, |declarations| {
            assert_eq!(declarations.len(), 1);
            let decl = declarations[0];
            assert_eq!(decl.import_kind, ImportOrExportKind::Value);
            assert_eq!(decl.phase, None);
            let specifiers = decl.specifiers.as_ref().unwrap();
            assert_eq!(specifiers.len(), 1);
            assert_eq!(specifiers[0].name(), "defer");
        });

        let src = "import foo = bar";
        parse_and_assert_statements(src, |statements| {
            if let Statement::TSImportEqualsDeclaration(decl) = statements[0] {
                assert_eq!(decl.import_kind, ImportOrExportKind::Value);
                assert_eq!(decl.id.name, "foo");
            } else {
                panic!("Expected TSImportEqualsDeclaration, found: {:?}", statements[0]);
            }
        });

        // `import type foo = bar` builds the AST but is a semantic error (TS1392:
        // an import alias cannot use `import type`), reported by the parser.
        let src = "import type foo = bar";
        parse_and_assert_statements_with_error(src, |statements| {
            if let Statement::TSImportEqualsDeclaration(decl) = statements[0] {
                assert_eq!(decl.import_kind, ImportOrExportKind::Type);
                assert_eq!(decl.id.name, "foo");
            } else {
                panic!("Expected TSImportEqualsDeclaration, found: {:?}", statements[0]);
            }
        });

        let src = "import type from = require('./a')";
        parse_and_assert_statements(src, |statements| {
            if let Statement::TSImportEqualsDeclaration(decl) = statements[0] {
                assert_eq!(decl.import_kind, ImportOrExportKind::Type);
                assert_eq!(decl.id.name, "from");
            } else {
                panic!("Expected TSImportEqualsDeclaration, found: {:?}", statements[0]);
            }
        });

        let src = "import from = b";
        parse_and_assert_statements(src, |statements| {
            if let Statement::TSImportEqualsDeclaration(decl) = statements[0] {
                assert_eq!(decl.import_kind, ImportOrExportKind::Value);
                assert_eq!(decl.id.name, "from");
            } else {
                panic!("Expected TSImportEqualsDeclaration, found: {:?}", statements[0]);
            }
        });
    }

    // https://github.com/oxc-project/oxc/issues/11505
    #[test]
    fn test_type_from_js_file() {
        let src = "import type from '../type.js'";
        let source_type = SourceType::default().with_typescript(false);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, src, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Failed to parse source: {src:?}, error: {:?}",
            ret.diagnostics
        );
        let declarations = ret.program.body.iter().collect::<Vec<_>>();
        assert_eq!(declarations.len(), 1);
        let Statement::ImportDeclaration(decl) = declarations[0] else {
            panic!("Expected ImportDeclaration, found: {:?}", declarations[0]);
        };
        assert_eq!(decl.import_kind, ImportOrExportKind::Value);
        assert!(decl.specifiers.is_some());
        let specifiers = decl.specifiers.as_ref().unwrap();
        assert_eq!(specifiers[0].name(), "type");
    }

    fn parse_and_assert_statements(
        src: &'static str,
        // takes a function which accepts the list of statements
        f: fn(Vec<&crate::syntax_tree::ast::Statement<'_>>) -> (),
    ) {
        let source_type = SourceType::default().with_typescript(true);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, src, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Failed to parse source: {src:?}, error: {:?}",
            ret.diagnostics
        );
        f(ret.program.body.iter().collect::<Vec<_>>());
    }

    /// Like [`parse_and_assert_statements`] but for a recoverable error: the parser
    /// reports a diagnostic yet still builds the AST, which `f` asserts on.
    fn parse_and_assert_statements_with_error(
        src: &'static str,
        f: fn(Vec<&crate::syntax_tree::ast::Statement<'_>>) -> (),
    ) {
        let source_type = SourceType::default().with_typescript(true);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, src, source_type).parse();
        assert!(!ret.diagnostics.is_empty(), "Expected a parse error for source: {src:?}");
        f(ret.program.body.iter().collect::<Vec<_>>());
    }

    fn parse_and_assert_import_declarations(
        src: &'static str,
        // takes a function which accepts the list of statements
        f: fn(Vec<&ArenaBox<'_, crate::syntax_tree::ast::ImportDeclaration<'_>>>) -> (),
    ) {
        let source_type = SourceType::default().with_typescript(true);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, src, source_type).parse();
        assert!(
            ret.diagnostics.is_empty(),
            "Failed to parse source: {src:?}, error: {:?}",
            ret.diagnostics
        );
        let statements =
            ret.program
                .body
                .iter()
                .filter_map(|s| {
                    if let Statement::ImportDeclaration(decl) = s { Some(decl) } else { None }
                })
                .collect::<Vec<_>>();
        f(statements);
    }
