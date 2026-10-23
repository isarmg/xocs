
    use std::cell::Cell;

    use super::*;
    use crate::source_location::Span;
    use crate::syntax_types::node::NodeId;

    // Note: These tests verify the logic of the methods.
    // Integration tests using real parsed AST are in the linter crate.

    #[test]
    fn test_has_argument_with_span_returns_false_for_non_call_expressions() {
        // Test that non-CallExpression/NewExpression AstKinds always return false
        let test_span = Span::new(0, 5);

        let num_lit = NumericLiteral {
            span: test_span,
            node_id: Cell::new(NodeId::DUMMY),
            value: 42.0,
            raw: None,
            base: crate::syntax_types::number::NumberBase::Decimal,
        };
        let num_kind = AstKind::NumericLiteral(&num_lit);
        assert!(!num_kind.has_argument_with_span(test_span));

        let bool_lit =
            BooleanLiteral { span: test_span, node_id: Cell::new(NodeId::DUMMY), value: true };
        let bool_kind = AstKind::BooleanLiteral(&bool_lit);
        assert!(!bool_kind.has_argument_with_span(test_span));
    }

    #[test]
    fn test_is_callee_with_span_returns_false_for_non_call_expressions() {
        // Test that non-CallExpression/NewExpression AstKinds always return false
        let test_span = Span::new(0, 5);

        let num_lit = NumericLiteral {
            span: test_span,
            node_id: Cell::new(NodeId::DUMMY),
            value: 42.0,
            raw: None,
            base: crate::syntax_types::number::NumberBase::Decimal,
        };
        let num_kind = AstKind::NumericLiteral(&num_lit);
        assert!(!num_kind.is_callee_with_span(test_span));

        let bool_lit =
            BooleanLiteral { span: test_span, node_id: Cell::new(NodeId::DUMMY), value: true };
        let bool_kind = AstKind::BooleanLiteral(&bool_lit);
        assert!(!bool_kind.is_callee_with_span(test_span));
    }
