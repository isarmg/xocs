
    use super::{CodeBuffer, IndentChar};

    #[test]
    fn empty() {
        let code = CodeBuffer::default();
        assert!(code.is_empty());
        assert_eq!(code.len(), 0);
        assert_eq!(String::from(code), "");
    }

    #[test]
    fn as_str() {
        let s = "Hello, world!";
        let mut code = CodeBuffer::with_capacity(s.len());
        code.print_str(s);

        let source = code.as_str();
        assert_eq!(source, s);
    }

    #[test]
    fn into_string() {
        let s = "Hello, world!";
        let mut code = CodeBuffer::with_capacity(s.len());
        code.print_str(s);

        let source = code.into_string();
        assert_eq!(source, s);
    }

    #[test]
    #[expect(clippy::byte_char_slices)]
    fn print_ascii_byte() {
        let mut code = CodeBuffer::new();
        code.print_ascii_byte(b'f');
        code.print_ascii_byte(b'o');
        code.print_ascii_byte(b'o');

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    #[expect(clippy::byte_char_slices)]
    fn print_byte_unchecked() {
        let mut code = CodeBuffer::new();
        // SAFETY: These bytes are all ASCII
        unsafe {
            code.print_byte_unchecked(b'f');
            code.print_byte_unchecked(b'o');
            code.print_byte_unchecked(b'o');
        }

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    #[expect(clippy::byte_char_slices)]
    fn print_bytes_unchecked() {
        let mut code = CodeBuffer::new();
        // SAFETY: These bytes are all ASCII
        unsafe { code.print_bytes_unchecked(b"foo") };

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    fn print_strs_array() {
        // Standard operation
        let mut code = CodeBuffer::new();
        let name = "foo";
        let value = "123";
        code.print_strs_array(["const ", name, " = ", value, ";"]);
        assert_eq!(String::from(code), "const foo = 123;");

        // Empty array
        let mut code = CodeBuffer::new();
        code.print_strs_array([]);
        assert_eq!(String::from(code), "");

        // Array of empty strings
        let mut code = CodeBuffer::new();
        code.print_strs_array(["", "", ""]);
        assert_eq!(String::from(code), "");

        // Array containing empty and non-empty strings
        let mut code = CodeBuffer::new();
        code.print_strs_array(["", "foo", "", "bar", "", "qux", ""]);
        assert_eq!(String::from(code), "foobarqux");
    }

    #[test]
    #[expect(clippy::byte_char_slices)]
    fn print_bytes_iter_unchecked() {
        let mut code = CodeBuffer::new();
        // SAFETY: These bytes are all ASCII
        unsafe { code.print_bytes_iter_unchecked([b'f', b'o', b'o']) };

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    #[expect(clippy::byte_char_slices)]
    fn print_ascii_bytes() {
        let mut code = CodeBuffer::new();
        code.print_ascii_bytes([b'f', b'o', b'o']);

        assert_eq!(code.len(), 3);
        assert_eq!(code.as_bytes(), &[b'f', b'o', b'o']);
        assert_eq!(String::from(code), "foo");
    }

    #[test]
    fn peek_nth_char_back() {
        let mut code = CodeBuffer::new();
        code.print_str("bar");

        assert_eq!(code.peek_nth_char_back(0), Some('r'));
        assert_eq!(code.peek_nth_char_back(1), Some('a'));
        assert_eq!(code.peek_nth_char_back(2), Some('b'));
        assert_eq!(code.peek_nth_char_back(3), None);
    }

    #[test]
    fn peek_nth_byte_back() {
        let mut code = CodeBuffer::new();
        code.print_str("bar");

        assert_eq!(code.peek_nth_byte_back(0), Some(b'r'));
        assert_eq!(code.peek_nth_byte_back(1), Some(b'a'));
        assert_eq!(code.peek_nth_byte_back(2), Some(b'b'));
        assert_eq!(code.peek_nth_byte_back(3), None);
    }

    #[test]
    fn last_byte() {
        let mut code = CodeBuffer::new();
        assert_eq!(code.last_byte(), None);
        code.print_str("bar");
        assert_eq!(code.last_byte(), Some(b'r'));
    }

    #[test]
    fn last_char() {
        let mut code = CodeBuffer::new();
        assert_eq!(code.last_char(), None);
        code.print_str("bar");
        assert_eq!(code.last_char(), Some('r'));
    }

    #[test]
    fn test_cached_indent_tabs() {
        let mut code = CodeBuffer::with_indent(IndentChar::Tab, 1);
        code.print_indent(2);
        assert_eq!(code.into_string(), "\t\t");
    }

    #[test]
    fn test_cached_indent_spaces_width_2() {
        let mut code = CodeBuffer::with_indent(IndentChar::Space, 2);
        code.print_indent(2);
        assert_eq!(code.into_string(), "    "); // 2 levels * 2 spaces = 4 spaces
    }

    #[test]
    fn test_cached_indent_spaces_width_4() {
        let mut code = CodeBuffer::with_indent(IndentChar::Space, 4);
        code.print_indent(2);
        assert_eq!(code.into_string(), "        "); // 2 levels * 4 spaces = 8 spaces
    }
