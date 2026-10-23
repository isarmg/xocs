
    use super::*;

    #[test]
    fn count_capturing_groups() {
        for (source_text, expected) in [
            ("()", (1, 0)),
            (r"\1()", (1, 0)),
            ("(foo)", (1, 0)),
            ("(foo)(bar)", (2, 0)),
            ("(foo(bar))", (2, 0)),
            ("(foo)[(bar)]", (1, 0)),
            (r"(foo)\(bar\)", (1, 0)),
            ("(foo)(?<n>bar)", (2, 1)),
            ("(foo)(?=...)(?!...)(?<=...)(?<!...)(?:...)", (1, 0)),
            ("(foo)(?<n>bar)(?<nn>baz)", (3, 2)),
            ("(?<n>.)(?<m>.)|(?<n>..)|(?<m>.)", (4, 2)),
            ("(?<n>.)(?<m>.)|(?:..)|(?<m>.)", (3, 2)),
            // Test exit_group reset behavior: consecutive groups at same depth
            ("((?<a>x))((?<b>y))|(?<c>z)", (5, 3)), // 2 outer groups + 2 inner named + 1 named = 5 total
            ("((?<a>x))|((?<a>y))", (4, 1)), // 2 outer + 2 inner named = 4 total, 1 unique name
            // Nested groups with alternatives
            ("((?<a>x)|((?<a>y)))", (4, 1)), // 1 outer + 1 named + 1 inner + 1 named = 4 total
            ("(((?<a>x))|((?<b>y)))|(((?<a>z))|((?<b>w)))", (10, 2)), // Complex nesting
        ] {
            let mut reader = Reader::initialize(source_text, true, false).unwrap();

            let allocator = Allocator::default();
            let (num_of_left_capturing_parens, capturing_group_names) =
                parse_capturing_groups(&mut reader, true, &allocator).unwrap();

            let actual = (num_of_left_capturing_parens, capturing_group_names.len());
            assert_eq!(expected, actual, "{source_text}");
        }
    }

    #[test]
    fn duplicated_named_capturing_groups() {
        for source_text in [
            "(?<n>.)(?<n>..)",
            "(?<n>.(?<n>..))",
            "|(?<n>.(?<n>..))",
            "(?<m>.)|(?<n>.(?<n>..))",
            // Test consecutive groups with same name in same alternative (should be error)
            "((?<a>x))((?<a>y))((?<a>z))",
            "(?<n>a)((?<n>b))",
        ] {
            let mut reader = Reader::initialize(source_text, true, false).unwrap();

            let allocator = Allocator::default();
            assert!(
                parse_capturing_groups(&mut reader, true, &allocator).is_err(),
                "{source_text}"
            );
        }
    }
