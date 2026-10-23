
    use super::*;
    use crate::Parser;

    #[test]
    fn text_merge_stream_indent() {
        let source = r#"
    first line
    second line
"#;
        let parser = TextMergeStream::new(Parser::new(source));
        let text_events: Vec<_> = parser.filter(|e| matches!(e, Event::Text(_))).collect();
        assert_eq!(
            text_events,
            [Event::Text("first line\nsecond line\n".into())]
        );
    }

    #[test]
    fn text_merge_with_offset_indent() {
        let source = r#"
    first line
    second line
"#;
        let parser = TextMergeWithOffset::new(Parser::new(source).into_offset_iter());
        let text_events: Vec<_> = parser
            .filter(|e| matches!(e, (Event::Text(_), _)))
            .collect();
        assert_eq!(
            text_events,
            [(Event::Text("first line\nsecond line\n".into()), 5..32)]
        );
    }

    #[test]
    fn text_merge_empty_is_discarded() {
        let events = [
            Event::Rule,
            Event::Text("".into()),
            Event::Text("".into()),
            Event::Rule,
        ];
        let result: Vec<_> = TextMergeStream::new(events.into_iter()).collect();
        assert_eq!(result, [Event::Rule, Event::Rule]);
    }
