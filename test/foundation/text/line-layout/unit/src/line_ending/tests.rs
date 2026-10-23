
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn non_empty_lines_full_case() {
        assert_eq!(
            NonEmptyLines("LF\nCRLF\r\n\r\n\nunterminated")
                .collect::<Vec<(&str, Option<LineEnding>)>>(),
            vec![
                ("LF", Some(LineEnding::LF)),
                ("CRLF", Some(LineEnding::CRLF)),
                ("unterminated", None),
            ]
        );
    }

    #[test]
    fn non_empty_lines_new_lines_only() {
        assert_eq!(NonEmptyLines("\r\n\n\n\r\n").next(), None);
    }

    #[test]
    fn non_empty_lines_no_input() {
        assert_eq!(NonEmptyLines("").next(), None);
    }
