
    use super::LabeledSpan;
    use crate::source_location::Span;

    #[test]
    fn labeled_spans_preserve_label_and_primary_state() {
        let label = LabeledSpan::new_primary_with_span(Some("here".to_string()), 3..7);
        assert_eq!(label.label(), Some("here"));
        assert_eq!(label.offset(), 3);
        assert_eq!(label.len(), 4);
        assert!(label.primary());
        assert_eq!(label.span(), Span::new(3, 7));
    }

    #[test]
    fn moving_a_label_preserves_its_length() {
        let mut label = LabeledSpan::underline(3..7);
        label.set_span_offset(10);
        assert_eq!(label.span(), Span::new(10, 14));
    }
