
    use super::*;

    /// Test wrappers that implement `GatherNodeParts` for testing purposes
    struct TestNode<'a, const LEN: usize>([&'a str; LEN]);

    impl<'a, const LEN: usize> GatherNodeParts<'a> for TestNode<'a, LEN> {
        fn gather<F: FnMut(&str)>(&self, f: &mut F) {
            for s in self.0 {
                f(s);
            }
        }
    }

    macro_rules! test_single {
        ($input:expr, $output:expr, $input_len:expr, $output_len:expr) => {{
            assert_eq!($input.len(), $input_len);
            assert_eq!($output.len(), $output_len);
            let node = TestNode([$input]);
            assert_eq!(get_var_name_from_node(&node), $output);
        }};
    }

    macro_rules! test_double {
        ($input1:expr, $input2:expr, $output:expr, $input1_len:expr, $input2_len:expr, $output_len:expr) => {{
            assert_eq!($input1.len(), $input1_len);
            assert_eq!($input2.len(), $input2_len);
            assert_eq!($output.len(), $output_len);
            let node = TestNode([$input1, $input2]);
            assert_eq!(get_var_name_from_node(&node), $output);
        }};
    }

    #[test]
    fn test_get_var_name_truncation_limits_to_approximately_20_bytes() {
        // ASCII
        test_single!("a", "a", 1, 1);
        test_single!("abcde", "abcde", 5, 5);
        test_single!("abcdefghijklmnopqrs", "abcdefghijklmnopqrs", 19, 19);
        test_single!("abcdefghijklmnopqrst", "abcdefghijklmnopqrst", 20, 20);
        test_single!("abcdefghijklmnopqrstu", "abcdefghijklmnopqrst", 21, 20);
        test_single!("abcdefghijklmnopqrstuvwxyz", "abcdefghijklmnopqrst", 26, 20);

        // 2-byte UTF-8 (Greek)
        test_single!("α", "α", 2, 2);
        test_single!("αβγδεζηθι", "αβγδεζηθι", 18, 18);
        test_single!("xαβγδεζηθι", "xαβγδεζηθι", 19, 19);
        test_single!("αβγδεζηθικ", "αβγδεζηθικ", 20, 20);
        test_single!("xαβγδεζηθικ", "xαβγδεζηθι", 21, 19);
        test_single!("αβγδεζηθικλ", "αβγδεζηθικ", 22, 20);
        test_single!("xαβγδεζηθικλ", "xαβγδεζηθι", 23, 19);
        test_single!("αβγδεζηθικλμνξοπρστυφ", "αβγδεζηθικ", 42, 20);
        test_single!("xαβγδεζηθικλμνξοπρστυφ", "xαβγδεζηθι", 43, 19);

        // 3-byte UTF-8 (Korean)
        test_single!("가", "가", 3, 3);
        test_single!("가나다라마바", "가나다라마바", 18, 18);
        test_single!("x가나다라마바", "x가나다라마바", 19, 19);
        test_single!("xx가나다라마바", "xx가나다라마바", 20, 20);
        test_single!("가나다라마바사", "가나다라마바", 21, 18);
        test_single!("x가나다라마바사", "x가나다라마바", 22, 19);
        test_single!("xx가나다라마바사", "xx가나다라마바", 23, 20);
        test_single!("가나다라마바사아자차", "가나다라마바", 30, 18);
        test_single!("x가나다라마바사아자차", "x가나다라마바", 31, 19);
        test_single!("xx가나다라마바사아자차", "xx가나다라마바", 32, 20);

        // 4-byte UTF-8 (CJK Ext B)
        test_single!("𠀀", "𠀀", 4, 4);
        test_single!("𠀀𠀁𠀂𠀃", "𠀀𠀁𠀂𠀃", 16, 16);
        test_single!("x𠀀𠀁𠀂𠀃", "x𠀀𠀁𠀂𠀃", 17, 17);
        test_single!("xx𠀀𠀁𠀂𠀃", "xx𠀀𠀁𠀂𠀃", 18, 18);
        test_single!("xxx𠀀𠀁𠀂𠀃", "xxx𠀀𠀁𠀂𠀃", 19, 19);
        test_single!("𠀀𠀁𠀂𠀃𠀄", "𠀀𠀁𠀂𠀃𠀄", 20, 20);
        test_single!("x𠀀𠀁𠀂𠀃𠀄", "x𠀀𠀁𠀂𠀃", 21, 17);
        test_single!("xx𠀀𠀁𠀂𠀃𠀄", "xx𠀀𠀁𠀂𠀃", 22, 18);
        test_single!("xxx𠀀𠀁𠀂𠀃𠀄", "xxx𠀀𠀁𠀂𠀃", 23, 19);
        test_single!("𠀀𠀁𠀂𠀃𠀄𠀅", "𠀀𠀁𠀂𠀃𠀄", 24, 20);
        test_single!("𠀀𠀁𠀂𠀃𠀄𠀀𠀁𠀂𠀃𠀄", "𠀀𠀁𠀂𠀃𠀄", 40, 20);
        test_single!("x𠀀𠀁𠀂𠀃𠀄𠀀𠀁𠀂𠀃𠀄", "x𠀀𠀁𠀂𠀃", 41, 17);
        test_single!("xx𠀀𠀁𠀂𠀃𠀄𠀀𠀁𠀂𠀃𠀄", "xx𠀀𠀁𠀂𠀃", 42, 18);
        test_single!("xxx𠀀𠀁𠀂𠀃𠀄𠀀𠀁𠀂𠀃𠀄", "xxx𠀀𠀁𠀂𠀃", 43, 19);
    }

    #[test]
    fn test_get_var_name_empty_returns_ref() {
        test_single!("", "ref", 0, 3);
        test_double!("", "", "ref", 0, 0, 3);
    }

    #[test]
    fn test_get_var_name_strips_leading_underscores() {
        test_single!("___foo", "foo", 6, 3);
        test_single!("___abcdefghijklmnopqrs", "abcdefghijklmnopqrs", 22, 19);
        test_single!("___abcdefghijklmnopqrst", "abcdefghijklmnopqrst", 23, 20);
        test_single!("___abcdefghijklmnopqrstu", "abcdefghijklmnopqrst", 24, 20);
    }

    #[test]
    fn test_get_var_name_double_joins_parts_with_dollar() {
        test_double!("foo", "bar", "foo$bar", 3, 3, 7);
        test_double!("a", "b", "a$b", 1, 1, 3);
        // Parts join to exactly `MAX_LEN` bytes (9 + `$` + 10) - no truncation.
        test_double!("foobarbaz", "abcdefghij", "foobarbaz$abcdefghij", 9, 10, 20);
    }

    #[test]
    fn test_get_var_name_double_empty_and_underscores() {
        // Empty second part still inserts the `$` separator.
        test_double!("foo", "", "foo$", 3, 0, 4);
        // Empty first part leaves `name` empty, so the second part is treated as the
        // first: no `$` separator, and leading underscores are trimmed.
        test_double!("", "bar", "bar", 0, 3, 3);
        // First part is all underscores -> trims to empty -> same as above.
        test_double!("___", "bar", "bar", 3, 3, 3);
        // Both parts empty (after trimming) -> fallback to `ref`.
        test_double!("___", "___", "ref", 3, 3, 3);
        // Leading underscores are only trimmed from the first non-empty part.
        // The second part keeps its underscores (name is non-empty by then).
        test_double!("_foo", "_bar", "foo$_bar", 4, 4, 8);
        test_double!("_foo", "___", "foo$___", 4, 3, 7);
    }

    #[test]
    fn test_get_var_name_double_first_part_fills_name() {
        // First part already fills `name` to `MAX_LEN`, so the second part is ignored
        // (early return at top of closure).
        test_double!("abcdefghijklmnopqrst", "xyz", "abcdefghijklmnopqrst", 20, 3, 20);
        // First part exceeds `MAX_LEN`, gets truncated to 20, second part ignored.
        test_double!("abcdefghijklmnopqrstu", "xyz", "abcdefghijklmnopqrst", 21, 3, 20);
    }

    #[test]
    fn test_get_var_name_double_dollar_reaches_max_len() {
        // Regression test for `cut_at == 0`:
        // First part is 19 bytes, pushing `$` brings `name` to exactly `MAX_LEN` (20).
        // The second part is then truncated to zero bytes (`cut_at == 0`), leaving a
        // trailing `$`. This previously tripped a `debug_assert!(cut_at > 0)`.
        test_double!("abcdefghijklmnopqrs", "x", "abcdefghijklmnopqrs$", 19, 1, 20);
        // Same, but second part starts with a multi-byte char - exercises the cold
        // `cut` path with `cut_at == 0` (must not underflow when indexing back).
        test_double!("abcdefghijklmnopqrs", "β", "abcdefghijklmnopqrs$", 19, 2, 20);
        // First part is 19 bytes of mixed ASCII + multi-byte UTF-8.
        test_double!("xαβγδεζηθι", "y", "xαβγδεζηθι$", 19, 1, 20);
    }

    #[test]
    fn test_get_var_name_double_truncates_second_part_ascii() {
        test_double!("foobar", "abcdefghijklmnopqrstuvwxyz", "foobar$abcdefghijklm", 6, 26, 20);
        test_double!("ab", "cdefghijklmnopqrstuvwxyz", "ab$cdefghijklmnopqrs", 2, 24, 20);
        // Second part fits exactly to `MAX_LEN` after the `$` - no truncation.
        test_double!("abcdefghijklmnopqr", "z", "abcdefghijklmnopqr$z", 18, 1, 20);
        // `cut_at == 1`: only the first byte of the second part survives.
        test_double!("abcdefghijklmnopqr", "yz", "abcdefghijklmnopqr$y", 18, 2, 20);
    }

    #[test]
    fn test_get_var_name_double_truncates_second_part_utf8() {
        // The second part is composed entirely of multi-byte UTF-8 chars. The cut point
        // lands inside it, and `cut` walks back to the start of the char it lands on -
        // so the walk-back distance is determined by which byte of the char is hit.
        // Cover every (char width, byte position) combination.

        // 2-byte chars (Greek): cut can land on byte 0 or 1.
        test_double!("abc", "αβγδεζηθικ", "abc$αβγδεζηθ", 3, 20, 20); // byte 0 (lead): walk back 0
        test_double!("ab", "αβγδεζηθικ", "ab$αβγδεζηθ", 2, 20, 19); // byte 1: walk back 1

        // 3-byte chars (Korean): cut can land on byte 0, 1 or 2.
        test_double!("abcd", "가나다라마바사", "abcd$가나다라마", 4, 21, 20); // byte 0 (lead): walk back 0
        test_double!("abc", "가나다라마바사", "abc$가나다라마", 3, 21, 19); // byte 1: walk back 1
        test_double!("ab", "가나다라마바사", "ab$가나다라마", 2, 21, 18); // byte 2: walk back 2

        // 4-byte chars (CJK Ext B): cut can land on byte 0, 1, 2 or 3.
        test_double!("abc", "𠀀𠀁𠀂𠀃𠀄𠀅", "abc$𠀀𠀁𠀂𠀃", 3, 24, 20); // byte 0 (lead): walk back 0
        test_double!("ab", "𠀀𠀁𠀂𠀃𠀄𠀅", "ab$𠀀𠀁𠀂𠀃", 2, 24, 19); // byte 1: walk back 1
        test_double!("a", "𠀀𠀁𠀂𠀃𠀄𠀅", "a$𠀀𠀁𠀂𠀃", 1, 24, 18); // byte 2: walk back 2
        test_double!("abcd", "𠀀𠀁𠀂𠀃𠀄𠀅", "abcd$𠀀𠀁𠀂", 4, 24, 17); // byte 3: walk back 3

        // Mixed ASCII + 2-byte: cut point lands mid-char after some ASCII bytes.
        test_double!("ab", "xyαβγδεζηθικ", "ab$xyαβγδεζη", 2, 22, 19);
    }

    #[test]
    fn test_get_var_name_double_passes_through_to_identifier() {
        // Digit mid-string is a valid identifier part - kept as-is.
        test_double!("foo", "1bar", "foo$1bar", 3, 4, 8);
        // Leading digit is not a valid identifier start - next char is capitalized.
        test_double!("1foo", "bar", "Foo$bar", 4, 3, 7);
        // Invalid identifier chars trigger camel-casing of the following char.
        test_double!("foo-bar.qux", "x", "fooBarQux$x", 11, 1, 11);
        test_double!("foo", "bar qux grep", "foo$barQuxGrep", 3, 12, 14);
    }
