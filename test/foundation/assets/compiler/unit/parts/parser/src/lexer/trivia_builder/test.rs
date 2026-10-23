
    use crate::syntax_arena::Allocator;
    use crate::syntax_tree::{Comment, CommentContent, CommentKind, CommentPosition, ast::CommentNewlines};
    use crate::source_location::{SourceType, Span};

    use crate::parser::Parser;

    fn get_comments(source_text: &str) -> Vec<Comment> {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(ret.diagnostics.is_empty());
        ret.program.comments.iter().copied().collect::<Vec<_>>()
    }

    fn get_comments_typescript(source_text: &str) -> Vec<Comment> {
        let allocator = Allocator::default();
        let source_type = SourceType::default().with_typescript(true);
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(ret.diagnostics.is_empty());
        ret.program.comments.iter().copied().collect::<Vec<_>>()
    }

    #[test]
    fn comment_attachments() {
        let source_text = "
        /* Leading 1 */
        // Leading 2
        /* Leading 3 */ token /* Trailing 1 */ // Trailing 2
        // Leading of EOF token
        ";
        let comments = get_comments(source_text);
        let expected = [
            Comment {
                span: Span::new(9, 24),
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Leading,
                attached_to: 70,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(33, 45),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 70,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(54, 69),
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Leading,
                attached_to: 70,
                newlines: CommentNewlines::Leading,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(76, 92),
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Trailing,
                attached_to: 75,
                newlines: CommentNewlines::None,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(93, 106),
                kind: CommentKind::Line,
                position: CommentPosition::Trailing,
                attached_to: 75,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(115, 138),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 147,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
        ];

        assert_eq!(comments.len(), expected.len());
        for (comment, expected) in comments.iter().copied().zip(expected) {
            assert_eq!(comment, expected, "{}", comment.content_span().source_text(source_text));
        }
    }

    #[test]
    fn comment_attachments2() {
        let source_text = "#!/usr/bin/env node
/* Leading 1 */
token /* Trailing 1 */
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(20, 35),
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Leading,
                attached_to: 36,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(42, 58),
                kind: CommentKind::SingleLineBlock,
                position: CommentPosition::Trailing,
                attached_to: 41,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn html_comment_content() {
        let allocator = Allocator::default();
        let source = "<!--a\n-->\n<!--";
        let ret = Parser::new(&allocator, source, SourceType::script()).parse();
        assert!(ret.diagnostics.is_empty());
        assert_eq!(ret.program.comments.len(), 3);
        let expected = [(Span::new(0, 5), "a"), (Span::new(6, 9), ""), (Span::new(10, 14), "")];
        for (comment, (span, content)) in ret.program.comments.iter().zip(expected) {
            assert!(comment.is_line());
            assert_eq!(comment.span, span);
            assert_eq!(comment.content_span().source_text(source), content);
        }
    }

    #[test]
    fn html_close_comments_after_irregular_line_terminators_are_leading() {
        for line_terminator in ['\u{2028}', '\u{2029}'] {
            let allocator = Allocator::default();
            let source_text = format!("foo();{line_terminator}--> comment\nbar();");
            let source_type = SourceType::default().with_script(true);
            let ret = Parser::new(&allocator, &source_text, source_type).parse();
            assert!(ret.diagnostics.is_empty());

            let comments = &ret.program.comments;
            assert_eq!(comments.len(), 1);
            assert!(comments[0].is_leading());
            assert!(comments[0].preceded_by_newline());
            let bar_start = u32::try_from(source_text.find("bar").unwrap()).unwrap();
            assert_eq!(comments[0].attached_to, bar_start);
        }
    }

    #[test]
    fn comments_around_binary_operands_attach_to_their_adjacent_tokens() {
        let source_text = "/* a leading */ a /* a trailing */ + /* b leading */ b /* b trailing */";
        let comments = get_comments(source_text);
        let expected = [
            (CommentPosition::Leading, 16),
            (CommentPosition::Trailing, 17),
            (CommentPosition::Leading, 53),
            (CommentPosition::Trailing, 54),
        ];

        assert_eq!(comments.len(), expected.len());
        for (comment, (position, attached_to)) in comments.iter().zip(expected) {
            assert_eq!(comment.position, position);
            assert_eq!(comment.attached_to, attached_to);
        }
    }

    #[test]
    fn infix_comment_attachment_respects_line_and_unary_boundaries() {
        for (source_text, position, attached_to) in [
            ("a /* trailing */\n+ b", CommentPosition::Trailing, 1),
            ("a\n/* leading */ +b", CommentPosition::Leading, 16),
            ("void /* leading */ +value", CommentPosition::Leading, 19),
            ("a /* webpackFoo: 1 */ + b", CommentPosition::Trailing, 1),
            ("foo(); /* trailing */", CommentPosition::Trailing, 6),
        ] {
            let comments = get_comments(source_text);
            assert_eq!(comments.len(), 1, "{source_text}");
            assert_eq!(comments[0].position, position, "{source_text}");
            assert_eq!(comments[0].attached_to, attached_to, "{source_text}");
        }
    }

    #[test]
    fn trailing_comment_attachment_survives_arrow_lookahead() {
        let source_text = "const f = (value /* trailing */\n) => value;";
        let comments = get_comments(source_text);
        let value_end = u32::try_from(source_text.find("value ").unwrap() + "value".len()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Trailing);
        assert_eq!(comments[0].attached_to, value_end);
    }

    #[test]
    fn comment_attachments3() {
        let source_text = "
/*
 * A
 **/
/*
 * B
 **/
 token
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(1, 13),
                kind: CommentKind::MultiLineBlock,
                position: CommentPosition::Leading,
                attached_to: 28,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(14, 26),
                kind: CommentKind::MultiLineBlock,
                position: CommentPosition::Leading,
                attached_to: 28,
                newlines: CommentNewlines::Leading | CommentNewlines::Trailing,
                content: CommentContent::None,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn legal_comment_after_code_is_attached_to_next_token() {
        let source_text = "foo();/**
 * @license MIT
 **/
function bar() {}";
        let comments = get_comments(source_text);
        let function_start = u32::try_from(source_text.find("function").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, function_start);
        assert!(comments[0].is_legal());
        assert!(comments[0].followed_by_newline());
    }

    #[test]
    fn legal_line_comment_after_code_is_attached_to_next_token() {
        let source_text = "foo();//! @license MIT\nfunction bar() {}";
        let comments = get_comments(source_text);
        let function_start = u32::try_from(source_text.find("function").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, function_start);
        assert!(comments[0].is_legal());
        assert!(comments[0].followed_by_newline());
    }

    // Annotation comments mark the *next* token, so they must stay leading even
    // when they sit directly after a previous statement with no preceding newline
    // (which is what codegen produces in `minify` mode). Without this, pass 2 of
    // an idempotency test would re-classify the annotation as trailing of the
    // previous token, drop its `attached_to`, and the codegen would fall back to
    // the canonical literal — diverging from pass 1's verbatim output.
    #[test]
    fn no_side_effects_block_comment_after_code_is_attached_to_next_token() {
        let source_text = "function foo() {}/* #__NO_SIDE_EFFECTS__ */\nfunction bar() {}";
        let comments = get_comments(source_text);
        let bar_start = u32::try_from(source_text.rfind("function").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, bar_start);
        assert!(comments[0].is_no_side_effects());
    }

    #[test]
    fn no_side_effects_line_comment_after_code_is_attached_to_next_token() {
        let source_text = "foo();// @__NO_SIDE_EFFECTS__\nfunction bar() {}";
        let comments = get_comments(source_text);
        let function_start = u32::try_from(source_text.find("function").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, function_start);
        assert!(comments[0].is_no_side_effects());
        assert!(comments[0].followed_by_newline());
    }

    #[test]
    fn pure_block_comment_after_code_is_attached_to_next_token() {
        let source_text = "foo();/* @__PURE__ */new Bar()";
        let comments = get_comments(source_text);
        let new_start = u32::try_from(source_text.find("new").unwrap()).unwrap();

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].position, CommentPosition::Leading);
        assert_eq!(comments[0].attached_to, new_start);
        assert!(comments[0].is_pure());
    }

    #[test]
    fn property_key_comment_after_code_is_attached_to_next_literal() {
        for (source_text, literal) in [
            ("work();/* #__KEY__ */\n\"_field\";", "\"_field\""),
            ("work();// @__KEY__\n`_field`;", "`_field`"),
        ] {
            let comments = get_comments(source_text);
            let literal_start = u32::try_from(source_text.find(literal).unwrap()).unwrap();

            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].position, CommentPosition::Leading);
            assert_eq!(comments[0].attached_to, literal_start);
            assert!(comments[0].is_property_key_annotation());
            assert!(comments[0].is_annotation());
        }
    }

    #[test]
    fn leading_comments_after_eq() {
        let source_text = "
            const v1 = // Leading comment 1
            foo();
            function foo(param =// Leading comment 2
            new Foo()
            ) {}
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(24, 44),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 57,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(96, 116),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 129,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn leading_comments_after_left_parenthesis() {
        let source_text = "
            call(// Leading comment 1
                arguments)
            (// Leading comment 2
                arguments)
        ";
        let comments = get_comments(source_text);
        let expected = vec![
            Comment {
                span: Span::new(18, 38),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 55,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
            Comment {
                span: Span::new(79, 99),
                kind: CommentKind::Line,
                position: CommentPosition::Leading,
                attached_to: 116,
                newlines: CommentNewlines::Trailing,
                content: CommentContent::None,
            },
        ];
        assert_eq!(comments, expected);
    }

    #[test]
    fn leading_comments_after_colon() {
        // A line comment right after a conditional `:` anchors to the following
        // alternate (leading), not the previous token — otherwise it is dropped.
        let source_text = "v = cond ? a : // Leading comment\nb;";
        let comments = get_comments(source_text);
        let expected = vec![Comment {
            span: Span::new(15, 33),
            kind: CommentKind::Line,
            position: CommentPosition::Leading,
            attached_to: 34,
            newlines: CommentNewlines::Trailing,
            content: CommentContent::None,
        }];
        assert_eq!(comments, expected);
    }

    #[test]
    fn pure_comments_not_applied() {
        let cases = [
            "/* #__PURE__ */ React.createElement;",
            "/* @__PURE__ */ someVariable;",
            "/* #__PURE__ */ 42;",
            "!/* #__PURE__ */ x;",
            // Non-expression statements
            "/* #__PURE__ */ function foo() {}",
            "/* #__PURE__ */ class Foo {}",
            "/* #__PURE__ */ var x = foo();",
            // Pure comment before `=` in variable declarator
            "const foo /* #__PURE__ */ = pureOperation();",
            // Pure comment before object literal (triggers parser lookahead/rewind for arrow detection)
            "export const X = /* @__PURE__ */ { a: 1 };",
            "foo /* #__PURE__ */ = pureOperation();",
            "foo /* #__PURE__ */ + pureOperation();",
            "foo /* #__PURE__ */ && pureOperation();",
            "foo /* #__PURE__ */ in pureOperation();",
            "foo /* #__PURE__ */ as T;",
            "foo /* #__PURE__ */ satisfies T;",
            "foo /* #__PURE__ */ ? bar : baz;",
            "foo /* #__PURE__ */, bar;",
            "({ x /*#__PURE__*/: sideEffect() });",
            "true ? x /*#__PURE__*/ : sideEffect();",
            "foo /* #__PURE__ */++;",
            "foo /* #__PURE__ */--;",
            "foo /* #__PURE__ */.bar;",
            "foo /* #__PURE__ */[bar];",
            "foo /* #__PURE__ */();",
            "foo /* #__PURE__ */?.bar;",
            "foo /* #__PURE__ */!;",
            r"foo /* #__PURE__ */ `bar`;",
            "42 /* #__PURE__ */ + foo;",
            "(foo) /* #__PURE__ */ + bar;",
            "[foo] /* #__PURE__ */ + bar;",
            "foo\n/* #__PURE__ */ + bar;",
            "foo // #__PURE__\n+ bar;",
            "(a = foo /* #__PURE__ */ + bar)",
        ];
        for source_text in cases {
            let comments = get_comments_typescript(source_text);
            assert_eq!(comments[0].content, CommentContent::PureNotApplied, "{source_text}");
        }
    }

    #[test]
    fn pure_comment_applied_after_lookahead() {
        // The ordinary comment is read during arrow-function lookahead before the parser rewinds.
        // Re-lexing the pure comment must recover its index rather than the later comment's index.
        let source_text = "export const X = /* @__PURE__ */ foo(/* comment */);";
        let comments = get_comments(source_text);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].content, CommentContent::Pure, "{source_text}");
        assert_eq!(comments[1].content, CommentContent::None, "{source_text}");
    }

    #[test]
    fn pure_comment_applied() {
        let cases = [
            "/* #__PURE__ */ foo();",
            "/* #__PURE__ */ (foo)();",
            "/* #__PURE__ */ (new Foo());",
            "a + /* #__PURE__ */ <T>foo(), bar;",
            "x = /* #__PURE__ */ pureOperation() || y;",
            "y || (x = /* #__PURE__ */ pureOperation());",
            "y || /* #__PURE__ */ pureOperation();",
            "function f() {} /* #__PURE__ */ (foo)();",
            "/*#__PURE__*/ [foo][0]()",
            "if (cond) /*#__PURE__*/ (foo)();",
            "foo++\n/*#__PURE__*/ (bar)();",
            "function f() {} /*#__PURE__*/ [foo][0]();",
        ];
        for source_text in cases {
            let comments = get_comments_typescript(source_text);
            assert_eq!(comments[0].content, CommentContent::Pure, "{source_text}");
        }
    }

    #[test]
    fn multiple_pure_comments_applied() {
        let source_text = "export const X = /*#__PURE__*/ /* comment */ /*@__PURE__*/ foo();";
        let comments = get_comments(source_text);

        assert_eq!(comments.len(), 3);
        assert_eq!(comments[0].content, CommentContent::Pure);
        assert_eq!(comments[1].content, CommentContent::None);
        assert_eq!(comments[2].content, CommentContent::Pure);
    }

    #[test]
    fn pure_comment_applied_on_member_chain() {
        // Rollup/esbuild treat PURE as applying to the innermost call/new even when
        // member access wraps it; member-access side effects are a separate concern.
        let cases = [
            "/*#__PURE__*/ test().a.b.c;",
            "/*#__PURE__*/ new Foo().a;",
            "/*#__PURE__*/ test()[0].b;",
            "class C { #bar; m() { /*#__PURE__*/ this.foo().#bar; } }",
            // Chain expressions with member root
            "/*#__PURE__*/ foo()?.a.b;",
            "/*#__PURE__*/ foo?.().a.b;",
            "/*#__PURE__*/ foo?.()[0];",
        ];
        for source_text in cases {
            let comments = get_comments(source_text);
            assert_eq!(comments[0].content, CommentContent::Pure, "{source_text}");
        }
    }

    #[test]
    fn annotation_comments_track_application_independently() {
        // The first pure comment is invalid (before `foo`), the second is valid (before `bar()`).
        let source_text = "/*#__PURE__*/ foo + /*#__PURE__*/ bar()";
        let comments = get_comments(source_text);

        assert_eq!(comments.len(), 2, "{source_text}");
        assert_eq!(comments[0].content, CommentContent::PureNotApplied, "{source_text}");
        assert_eq!(comments[1].content, CommentContent::Pure, "{source_text}");
        let source_text = concat!(
            "/*#__NO_SIDE_EFFECTS__*/ value;",
            "/*#__NO_SIDE_EFFECTS__*/ /* comment */ /*@__NO_SIDE_EFFECTS__*/ function foo() {}",
        );
        let comments = get_comments(source_text);
        assert_eq!(comments.len(), 4);
        assert_eq!(comments[0].content, CommentContent::NoSideEffectsNotApplied,);
        assert_eq!(comments[1].content, CommentContent::NoSideEffects);
        assert_eq!(comments[2].content, CommentContent::None);
        assert_eq!(comments[3].content, CommentContent::NoSideEffects);
    }

    #[test]
    fn no_side_effects_comments_not_applied() {
        let cases = [
            "/* #__NO_SIDE_EFFECTS__ */",
            "/* @__NO_SIDE_EFFECTS__ */ assert.ok(true);",
            "/* #__NO_SIDE_EFFECTS__ */ const foo = 1, bar = () => {};",
            "/* #__NO_SIDE_EFFECTS__ */ class Foo {}",
            "/* #__NO_SIDE_EFFECTS__ */ let foo = () => {};",
            "/* #__NO_SIDE_EFFECTS__ */ var foo = function() {};",
            "const foo /* #__NO_SIDE_EFFECTS__ */ = () => {};",
        ];
        for source_text in cases {
            let comments = get_comments(source_text);
            assert_eq!(
                comments[0].content,
                CommentContent::NoSideEffectsNotApplied,
                "{source_text}"
            );
        }
    }

    #[test]
    fn no_side_effects_comment_applied() {
        let cases = [
            "/* #__NO_SIDE_EFFECTS__ */ function foo() {}",
            "/* #__NO_SIDE_EFFECTS__ */ async function foo() {}",
            "/* #__NO_SIDE_EFFECTS__ */ export function foo() {}",
            "export default /* #__NO_SIDE_EFFECTS__ */ function foo() {}",
            "const foo = /* #__NO_SIDE_EFFECTS__ */ function() {};",
            "const foo = /* #__NO_SIDE_EFFECTS__ */ () => {};",
            "/* #__NO_SIDE_EFFECTS__ */ const foo = () => {};",
            "/* #__NO_SIDE_EFFECTS__ */ export const foo = () => {};",
            "[/* #__NO_SIDE_EFFECTS__ */ function() {}];",
        ];
        for source_text in cases {
            let comments = get_comments(source_text);
            assert_eq!(comments[0].content, CommentContent::NoSideEffects, "{source_text}");
        }
    }

    #[test]
    fn block_comment_line_terminators() {
        for (separator, kind) in [
            ("\u{2028}", CommentKind::MultiLineBlock),
            ("\u{2029}", CommentKind::MultiLineBlock),
            ("\n", CommentKind::MultiLineBlock),
            ("\r", CommentKind::MultiLineBlock),
            ("\r\n", CommentKind::MultiLineBlock),
            ("", CommentKind::SingleLineBlock),
            ("\u{2027}", CommentKind::SingleLineBlock),
            ("\u{202a}", CommentKind::SingleLineBlock),
            ("\u{00a0}", CommentKind::SingleLineBlock),
        ] {
            // A newline before the comment must not affect its kind.
            for prefix in ["", "let a;", "let a;\n"] {
                let source = format!("{prefix}/* first{separator}second */ let b;");
                let comments = get_comments(&source);
                assert_eq!(comments.len(), 1, "{source:?}");
                assert_eq!(comments[0].kind, kind, "{source:?}");
            }
        }
    }

    #[test]
    fn comment_parsing() {
        let data = [
            ("/*! legal */", CommentContent::Legal),
            ("/* @preserve */", CommentContent::Legal),
            ("/* @license */", CommentContent::Legal),
            ("/* foo @preserve */", CommentContent::Legal),
            ("/* foo @license */", CommentContent::Legal),
            ("/* foo @preserve*/", CommentContent::Legal),
            ("/* foo @license*/", CommentContent::Legal),
            ("/* foo @licensed*/", CommentContent::Legal),
            ("/* foo @License*/", CommentContent::None),
            // spellchecker:disable-next-line
            ("/* foo @licens*/", CommentContent::None),
            // spellchecker:disable-next-line
            ("/* foo @preserv*/", CommentContent::None),
            ("/* @foo @preserve */", CommentContent::Legal),
            ("/* @foo @license */", CommentContent::Legal),
            ("/** foo @preserve */", CommentContent::JsdocLegal),
            ("/** foo @license */", CommentContent::JsdocLegal),
            ("/** foo @license*/", CommentContent::JsdocLegal),
            ("// foo @license", CommentContent::Legal),
            ("/** jsdoc */", CommentContent::Jsdoc),
            ("/**/", CommentContent::None),
            ("/***/", CommentContent::None),
            ("/*@*/", CommentContent::None),
            ("/*@xreserve*/", CommentContent::None),
            ("/*@preserve*/", CommentContent::Legal),
            ("/*@voidzeroignoreme*/", CommentContent::None),
            ("/****/", CommentContent::None),
            ("/* @vite-ignore */", CommentContent::Vite),
            ("/* @vite-xxx */", CommentContent::Vite),
            ("/* webpackChunkName: 'my-chunk-name' */", CommentContent::Webpack),
            ("/* webpack */", CommentContent::None),
            ("/* @__PURE__ */", CommentContent::PureNotApplied),
            ("/* @__NO_SIDE_EFFECTS__ */", CommentContent::NoSideEffectsNotApplied),
            ("/* #__PURE__ */", CommentContent::PureNotApplied),
            ("/* #__NO_SIDE_EFFECTS__ */", CommentContent::NoSideEffectsNotApplied),
            ("/* @__KEY__ */", CommentContent::PropertyKey),
            ("/* #__KEY__ */", CommentContent::PropertyKey),
            ("/*\u{a0}@__KEY__\u{a0}*/", CommentContent::PropertyKey),
            ("//@__KEY__", CommentContent::PropertyKey),
            ("// #__KEY__", CommentContent::PropertyKey),
            ("/**\n * @__KEY__\n */", CommentContent::Jsdoc),
            ("/* __KEY__ */", CommentContent::None),
            ("/* @ __KEY__ */", CommentContent::None),
            ("/* @__key__ */", CommentContent::None),
            ("/* @__KEY___ */", CommentContent::None),
            ("/* @__KEY__ extra */", CommentContent::None),
            ("/* turbopackOptional: true */", CommentContent::Turbopack),
            ("/* v8 ignore next */", CommentContent::CoverageIgnore),
            ("/* v8 ignore filename */", CommentContent::CoverageIgnore),
            ("/* c8 ignore file */", CommentContent::CoverageIgnore),
            ("/* v8 ignore file */", CommentContent::CoverageIgnoreFile),
            ("// v8 ignore file", CommentContent::CoverageIgnoreFile),
            ("/* v8 ignore file -- @preserve */", CommentContent::CoverageIgnoreFile),
            ("/* istanbul ignore file */", CommentContent::CoverageIgnoreFile),
            ("// istanbul ignore file -- generated", CommentContent::CoverageIgnoreFile),
        ];

        for (source_text, expected) in data {
            let comments = get_comments(source_text);
            assert_eq!(comments.len(), 1, "{source_text}");
            assert_eq!(comments[0].content, expected, "{source_text}");
        }
    }
