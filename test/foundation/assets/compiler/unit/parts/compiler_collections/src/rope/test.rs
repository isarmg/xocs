
    use ropey::Rope;

    fn test_line_column(offset: u32, source_text: &str) -> (u32, u32) {
        let rope = Rope::from_str(source_text);
        super::get_line_column(&rope, offset, source_text)
    }

    fn test_offset(line: u32, column: u32, source_text: &str) -> u32 {
        let rope = Rope::from_str(source_text);
        super::get_offset_from_line_and_column(&rope, line, column)
    }

    #[test]
    fn empty_file() {
        assert_eq!(test_line_column(0, ""), (0, 0));
        assert_eq!(test_offset(0, 0, ""), 0);
    }

    #[test]
    fn first_line_start() {
        assert_eq!(test_line_column(0, "foo\nbar\n"), (0, 0));
        assert_eq!(test_offset(0, 0, "foo\nbar\n"), 0);
    }

    #[test]
    fn first_line_middle() {
        assert_eq!(test_line_column(5, "blahblahblah\noops\n"), (0, 5));
        assert_eq!(test_offset(0, 5, "blahblahblah\noops\n"), 5);
    }

    #[test]
    fn later_line_start() {
        assert_eq!(test_line_column(8, "foo\nbar\nblahblahblah"), (2, 0));
        assert_eq!(test_offset(2, 0, "foo\nbar\nblahblahblah"), 8);
    }

    #[test]
    fn later_line_middle() {
        assert_eq!(test_line_column(12, "foo\nbar\nblahblahblah"), (2, 4));
        assert_eq!(test_offset(2, 4, "foo\nbar\nblahblahblah"), 12);
    }

    #[test]
    fn after_2_byte_unicode() {
        assert_eq!("£".len(), 2);
        assert_eq!(utf16_len("£"), 1);
        assert_eq!(test_line_column(4, "£abc"), (0, 3));
        assert_eq!(test_offset(0, 3, "£abc"), 4);
    }

    #[test]
    fn after_3_byte_unicode() {
        assert_eq!("अ".len(), 3);
        assert_eq!(utf16_len("अ"), 1);
        assert_eq!(test_line_column(5, "अabc"), (0, 3));
        assert_eq!(test_offset(0, 3, "अabc"), 5);
    }

    #[test]
    fn after_4_byte_unicode() {
        assert_eq!("🍄".len(), 4);
        assert_eq!(utf16_len("🍄"), 2);
        assert_eq!(test_line_column(6, "🍄abc"), (0, 4));
        assert_eq!(test_offset(0, 4, "🍄abc"), 6);
    }

    #[test]
    fn after_2_byte_unicode_on_previous_line() {
        assert_eq!("£".len(), 2);
        assert_eq!(utf16_len("£"), 1);
        assert_eq!(test_line_column(4, "£\nabc"), (1, 1));
        assert_eq!(test_offset(1, 1, "£\nabc"), 4);
    }

    #[test]
    fn after_3_byte_unicode_on_previous_line() {
        assert_eq!("अ".len(), 3);
        assert_eq!(utf16_len("अ"), 1);
        assert_eq!(test_line_column(5, "अ\nabc"), (1, 1));
        assert_eq!(test_offset(1, 1, "अ\nabc"), 5);
    }

    #[test]
    fn after_4_byte_unicode_on_previous_line() {
        assert_eq!("🍄".len(), 4);
        assert_eq!(utf16_len("🍄"), 2);
        assert_eq!(test_line_column(6, "🍄\nabc"), (1, 1));
        assert_eq!(test_offset(1, 1, "🍄\nabc"), 6);
    }

    #[cfg(test)]
    fn utf16_len(s: &str) -> usize {
        s.encode_utf16().count()
    }
