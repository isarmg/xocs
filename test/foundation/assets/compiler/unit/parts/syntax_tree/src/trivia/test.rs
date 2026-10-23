
    use super::*;

    #[test]
    fn test_comments_range() {
        let comments = vec![
            Comment::new(0, 4, CommentKind::Line),
            Comment::new(5, 9, CommentKind::Line),
            Comment::new(10, 13, CommentKind::Line),
            Comment::new(14, 17, CommentKind::Line),
            Comment::new(18, 23, CommentKind::Line),
        ]
        .into_boxed_slice();
        let full_len = comments.len();
        assert_eq!(comments_range(&comments, ..).count(), full_len);
        assert_eq!(comments_range(&comments, 1..).count(), full_len.saturating_sub(1));
        assert_eq!(comments_range(&comments, ..18).count(), full_len.saturating_sub(1));
        assert_eq!(comments_range(&comments, ..=18).count(), full_len);
    }

    #[test]
    fn test_is_inside_comment() {
        let comments = vec![
            Comment::new(0, 4, CommentKind::Line),
            Comment::new(10, 20, CommentKind::SingleLineBlock),
        ]
        .into_boxed_slice();

        assert!(is_inside_comment(&comments, 2));
        assert!(!is_inside_comment(&comments, 5));
        assert!(is_inside_comment(&comments, 15));
        assert!(!is_inside_comment(&comments, 21));
    }

    #[test]
    fn test_get_comment_at() {
        let comments = vec![
            Comment::new(0, 4, CommentKind::Line),
            Comment::new(10, 20, CommentKind::SingleLineBlock),
        ]
        .into_boxed_slice();

        // Inside first comment
        let comment = get_comment_at(&comments, 2);
        let comment = comment.expect("Expected a comment at position 2");
        assert_eq!(comment.span.start, 0);
        assert_eq!(comment.span.end, 4);

        // Between comments
        assert!(get_comment_at(&comments, 5).is_none());

        // Inside second comment
        let comment = get_comment_at(&comments, 15);
        let comment = comment.expect("Expected a comment at position 15");
        assert_eq!(comment.span.start, 10);
        assert_eq!(comment.span.end, 20);

        // After all comments
        assert!(get_comment_at(&comments, 21).is_none());

        // Boundary cases: start positions are included
        assert!(get_comment_at(&comments, 0).is_some()); // Start of first comment
        assert!(get_comment_at(&comments, 10).is_some()); // Start of second comment

        // Boundary cases: end positions are excluded
        assert!(get_comment_at(&comments, 4).is_none()); // End of first comment
        assert!(get_comment_at(&comments, 20).is_none()); // End of second comment
    }
