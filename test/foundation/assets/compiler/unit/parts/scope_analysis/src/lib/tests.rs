
    use crate::syntax_arena::Allocator;
    use crate::syntax_tree::{AstKind, ast::VariableDeclarationKind};
    use crate::source_location::SourceType;
    use crate::compiler_strings::{Str, static_ident};

    use super::*;

    /// Create a [`Semantic`] from source code, assuming there are no syntax/semantic errors.
    fn get_semantic<'s, 'a: 's>(
        allocator: &'a Allocator,
        source: &'s str,
        source_type: SourceType,
    ) -> Semantic<'s> {
        let parse = oxc_parser::Parser::new(allocator, source, source_type).parse();
        assert!(parse.diagnostics.is_empty());
        let semantic =
            SemanticBuilder::new().with_build_nodes(true).build(allocator.alloc(parse.program));
        assert!(semantic.diagnostics.is_empty(), "Parse error: {}", semantic.diagnostics[0]);
        semantic.semantic
    }

    #[test]
    fn test_symbols() {
        let source = "
            let a;
            function foo(a) {
                return a + 1;
            }
            let b = a + foo(1);";
        let allocator = Allocator::default();
        let semantic = get_semantic(&allocator, source, SourceType::default());

        let top_level_a = semantic
            .scoping()
            .get_binding(semantic.scoping().root_scope_id(), static_ident!("a"))
            .unwrap();

        let declarator_node = semantic.symbol_declaration(top_level_a);
        match declarator_node.kind() {
            AstKind::VariableDeclarator(_) => {
                let AstKind::VariableDeclaration(declaration) =
                    semantic.nodes().parent_kind(declarator_node.id())
                else {
                    unreachable!();
                };
                assert_eq!(declaration.kind, VariableDeclarationKind::Let);
            }
            kind => panic!("Expected VariableDeclarator for 'let', got {kind:?}"),
        }

        let references = semantic.symbol_references(top_level_a);
        assert_eq!(references.count(), 1);
    }

    #[test]
    fn test_top_level_symbols() {
        let source = "function Fn() {}";
        let allocator = Allocator::default();
        let semantic = get_semantic(&allocator, source, SourceType::default());
        let scopes = semantic.scoping();

        assert!(scopes.get_binding(scopes.root_scope_id(), static_ident!("Fn")).is_some());
    }

    #[test]
    fn repeated_build_with_named_class_expression_and_syntax_checks() {
        let allocator = Allocator::default();
        let source = "export const X = class Base {};";
        let source_type = SourceType::ts();
        let parse = oxc_parser::Parser::new(&allocator, source, source_type).parse();

        assert!(parse.diagnostics.is_empty());

        let first = SemanticBuilder::new_compiler().build(&parse.program);
        assert!(first.diagnostics.is_empty());

        let second = SemanticBuilder::new_compiler().build(&parse.program);
        assert!(second.diagnostics.is_empty());
    }

    #[test]
    fn test_is_global() {
        let source = "
            var a = 0;
            function foo() {
            a += 1;
            }

            var b = a + 2;
        ";
        let allocator = Allocator::default();
        let semantic = get_semantic(&allocator, source, SourceType::default());
        for node in semantic.nodes() {
            if let AstKind::IdentifierReference(id) = node.kind() {
                assert!(!semantic.is_reference_to_global_variable(id));
            }
        }
    }

    #[test]
    fn unresolved_reference_is_tracked_per_identifier() {
        let source = "Promise.resolve(); function f(Promise) { Promise.resolve(); }";
        let allocator = Allocator::default();
        let semantic = get_semantic(&allocator, source, SourceType::default());

        let nodes = semantic
            .scoping()
            .references
            .iter()
            .map(|reference| semantic.nodes.get_node(reference.node_id()))
            .collect::<Vec<_>>();
        let [global_promise, local_promise] = nodes.as_slice() else { unreachable!() };

        // `Promise.resolve()`
        assert!(semantic.is_unresolved_reference(global_promise.id()));
        assert!(semantic.is_reference_to_global_variable(
            global_promise.kind().as_identifier_reference().unwrap()
        ));

        // `function f(Promise) { Promise.resolve(); }`
        assert!(!semantic.is_unresolved_reference(local_promise.id()));
        assert!(!semantic.is_reference_to_global_variable(
            local_promise.kind().as_identifier_reference().unwrap()
        ));
    }

    #[test]
    fn type_alias_gets_reference() {
        let source = "type A = 1; type B = A";
        let allocator = Allocator::default();
        let source_type: SourceType = SourceType::default().with_typescript(true);
        let semantic = get_semantic(&allocator, source, source_type);
        assert_eq!(semantic.scoping().references.len(), 1);
    }

    #[test]
    fn test_reference_resolutions_simple_read_write() {
        let alloc = Allocator::default();
        let target_symbol_name = Str::from("a");
        let typescript = SourceType::ts();
        let sources = [
            // simple cases
            (SourceType::default(), "let a = 1; a = 2", ReferenceFlags::write()),
            (SourceType::default(), "let a = 1, b; b = a", ReferenceFlags::read()),
            (SourceType::default(), "let a = 1, b; b[a]", ReferenceFlags::read()),
            (SourceType::default(), "let a = 1, b = 1, c; c = a + b", ReferenceFlags::read()),
            (SourceType::default(), "function a() { return }; a()", ReferenceFlags::read()),
            (SourceType::default(), "class a {}; new a()", ReferenceFlags::read()),
            (SourceType::default(), "let a; function foo() { return a }", ReferenceFlags::read()),
            // pattern assignment
            (SourceType::default(), "let a = 1, b; b = { a }", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; ({ b } = { a })", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; ({ a } = { b })", ReferenceFlags::write()),
            (SourceType::default(), "let a, b; ([ b ] = [ a ])", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; ([ a ] = [ b ])", ReferenceFlags::write()),
            // property access/mutation
            (SourceType::default(), "let a = { b: 1 }; a.b = 2", ReferenceFlags::read()),
            (SourceType::default(), "let a = { b: 1 }; a.b += 2", ReferenceFlags::read()),
            // parens are pass-through
            (SourceType::default(), "let a = 1, b; b = (a)", ReferenceFlags::read()),
            (SourceType::default(), "let a = 1, b; b = ++(a)", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = ++((((a))))", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = ((++((a))))", ReferenceFlags::read_write()),
            // simple binops/calls for sanity check
            (SourceType::default(), "let a, b; a + b", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; b(a)", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; a = 5", ReferenceFlags::write()),
            // unary op counts as write, but checking continues up tree
            (SourceType::default(), "let a = 1, b; b = ++a", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = --a", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = a++", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1, b; b = a--", ReferenceFlags::read_write()),
            // assignment expressions count as read-write
            (SourceType::default(), "let a = 1, b; b = a += 5", ReferenceFlags::read_write()),
            (SourceType::default(), "let a = 1; a += 5", ReferenceFlags::read_write()),
            (SourceType::default(), "let a, b; b = a = 1", ReferenceFlags::write()),
            (SourceType::default(), "let a, b; b = (a = 1)", ReferenceFlags::write()),
            (SourceType::default(), "let a, b, c; b = c = a", ReferenceFlags::read()),
            // sequences return last read_write in sequence
            (SourceType::default(), "let a, b; b = (0, a++)", ReferenceFlags::read_write()),
            // loops
            (
                SourceType::default(),
                "var a, arr = [1, 2, 3]; for(a in arr) { break }",
                ReferenceFlags::write(),
            ),
            (
                SourceType::default(),
                "var a, obj = { }; for(a of obj) { break }",
                ReferenceFlags::write(),
            ),
            (SourceType::default(), "var a; for(; false; a++) { }", ReferenceFlags::read_write()),
            (SourceType::default(), "var a = 1; while(a < 5) { break }", ReferenceFlags::read()),
            // if statements
            (
                SourceType::default(),
                "let a; if (a) { true } else { false }",
                ReferenceFlags::read(),
            ),
            (
                SourceType::default(),
                "let a, b; if (a == b) { true } else { false }",
                ReferenceFlags::read(),
            ),
            (
                SourceType::default(),
                "let a, b; if (b == a) { true } else { false }",
                ReferenceFlags::read(),
            ),
            // identifiers not in last read_write are also considered a read (at
            // least, or now)
            (SourceType::default(), "let a, b; b = (a, 0)", ReferenceFlags::read()),
            (SourceType::default(), "let a, b; b = (--a, 0)", ReferenceFlags::read_write()),
            (
                SourceType::default(),
                "let a; function foo(a) { return a }; foo(a = 1)",
                //                                        ^ write
                ReferenceFlags::write(),
            ),
            // member expression
            (SourceType::default(), "let a; a.b = 1", ReferenceFlags::read()),
            (SourceType::default(), "let a; let b; b[a += 1] = 1", ReferenceFlags::read_write()),
            (
                SourceType::default(),
                "let a; let b; let c; b[c[a = c['a']] = 'c'] = 'b'",
                //                        ^ write
                ReferenceFlags::write(),
            ),
            (
                SourceType::default(),
                "let a; let b; let c; a[c[b = c['a']] = 'c'] = 'b'",
                ReferenceFlags::read(),
            ),
            (SourceType::default(), "console.log;let a=0;a++", ReferenceFlags::read_write()),
            //                                           ^^^ UpdateExpression is always a read | write
            // typescript
            (typescript, "let a: number = 1; (a as any) = true", ReferenceFlags::write()),
            (typescript, "let a: number = 1; a = true as any", ReferenceFlags::write()),
            (typescript, "let a: number = 1; a = 2 as const", ReferenceFlags::write()),
            (typescript, "let a: number = 1; a = 2 satisfies number", ReferenceFlags::write()),
            (typescript, "let a: number; (a as any) = 1;", ReferenceFlags::write()),
        ];

        for (source_type, source, flags) in sources {
            let semantic = get_semantic(&alloc, source, source_type);
            let a_id = semantic
                .scoping()
                .get_root_binding(target_symbol_name.into())
                .unwrap_or_else(|| {
                    panic!("no references for '{target_symbol_name}' found");
                });
            let a_refs: Vec<_> = semantic.symbol_references(a_id).collect();
            let num_refs = a_refs.len();

            assert!(
                num_refs == 1,
                "expected to find 1 reference to '{target_symbol_name}' but {num_refs} were found\n\nsource:\n{source}"
            );
            let ref_type = a_refs[0];
            if flags.is_write() {
                assert!(
                    ref_type.is_write(),
                    "expected reference to '{target_symbol_name}' to be write\n\nsource:\n{source}"
                );
            } else {
                assert!(
                    !ref_type.is_write(),
                    "expected reference to '{target_symbol_name}' not to have been written to, but it is\n\nsource:\n{source}"
                );
            }
            if flags.is_read() {
                assert!(
                    ref_type.is_read(),
                    "expected reference to '{target_symbol_name}' to be read\n\nsource:\n{source}"
                );
            } else {
                assert!(
                    !ref_type.is_read(),
                    "expected reference to '{target_symbol_name}' not to be read, but it is\n\nsource:\n{source}"
                );
            }
        }
    }
