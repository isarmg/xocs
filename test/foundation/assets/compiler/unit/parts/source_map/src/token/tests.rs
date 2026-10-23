
    use std::borrow::Cow;

    use super::*;

    fn sample_map() -> SourceMap<'static> {
        SourceMap::new(
            None,
            vec![Cow::Borrowed("name0")],
            None,
            vec![Cow::Borrowed("src0.js")],
            vec![Some(Cow::Borrowed("the source content"))],
            vec![Token::new(2, 3, 4, 5, Some(0), Some(0))].into_boxed_slice(),
            None,
        )
    }

    #[test]
    fn token_getters() {
        let token = Token::new(1, 2, 3, 4, Some(5), Some(6));
        assert_eq!(token.get_dst_line(), 1);
        assert_eq!(token.get_dst_col(), 2);
        assert_eq!(token.get_src_line(), 3);
        assert_eq!(token.get_src_col(), 4);
        assert_eq!(token.get_source_id(), Some(5));
        assert_eq!(token.get_name_id(), Some(6));

        let missing = Token::new(0, 0, 0, 0, None, None);
        assert_eq!(missing.get_source_id(), None);
        assert_eq!(missing.get_name_id(), None);
    }

    #[test]
    fn token_translated() {
        let token = Token::new(1, 2, 3, 4, Some(5), Some(6));
        assert_eq!(token.translated(10, 100, 1000), Token::new(11, 2, 3, 4, Some(105), Some(1006)));

        // The missing-id sentinel survives translation rather than wrapping.
        let missing = Token::new(0, 0, 0, 0, None, None);
        let shifted = missing.translated(5, 5, 5);
        assert_eq!(shifted.get_dst_line(), 5);
        assert_eq!(shifted.get_source_id(), None);
        assert_eq!(shifted.get_name_id(), None);
    }

    #[test]
    fn source_view_token_accessors() {
        let sm = sample_map();
        let token = sm.get_source_view_token(0).unwrap();
        assert_eq!(token.get_dst_line(), 2);
        assert_eq!(token.get_dst_col(), 3);
        assert_eq!(token.get_src_line(), 4);
        assert_eq!(token.get_src_col(), 5);
        assert_eq!(token.get_source_id(), Some(0));
        assert_eq!(token.get_name_id(), Some(0));
        assert_eq!(token.get_name(), Some("name0"));
        assert_eq!(token.get_source(), Some("src0.js"));
        assert_eq!(token.get_source_content(), Some("the source content"));
        assert_eq!(token.get_source_and_content(), Some(("src0.js", "the source content")));
        assert_eq!(token.to_tuple(), (Some("src0.js"), 4, 5, Some("name0")));
    }

    #[test]
    fn source_view_token_without_ids() {
        let sm = SourceMap::new(
            None,
            vec![],
            None,
            vec![],
            vec![],
            vec![Token::new(0, 0, 0, 0, None, None)].into_boxed_slice(),
            None,
        );
        let token = sm.get_source_view_token(0).unwrap();
        assert_eq!(token.get_name(), None);
        assert_eq!(token.get_source(), None);
        assert_eq!(token.get_source_content(), None);
        assert_eq!(token.get_source_and_content(), None);
    }
