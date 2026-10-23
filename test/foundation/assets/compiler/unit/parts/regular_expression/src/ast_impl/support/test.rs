
    use crate::syntax_arena::Allocator;

    use super::*;
    use crate::regular_expression::{LiteralParser, Options};

    #[test]
    fn unsupported_flags() {
        let supported = RegexUnsupportedFlags::default();
        assert!(!has_unsupported_regular_expression_flags("gim", &supported));
        assert!(has_unsupported_regular_expression_flags("gg", &supported));
        assert!(has_unsupported_regular_expression_flags("uv", &supported));
        assert!(has_unsupported_regular_expression_flags("x", &supported));

        let unsupported = RegexUnsupportedFlags {
            sticky: true,
            unicode: true,
            dot_all: true,
            match_indices: true,
            unicode_sets: true,
        };
        for flags in ["y", "u", "s", "d", "v"] {
            assert!(has_unsupported_regular_expression_flags(flags, &unsupported));
        }
    }

    #[test]
    fn duplicate_named_capture_groups() {
        let allocator = Allocator::default();
        let pattern =
            LiteralParser::new(&allocator, "(?<name>a)|(?<name>b)", None, Options::default())
                .parse()
                .unwrap();

        let supported = RegexUnsupportedPatterns {
            named_capture_groups: false,
            duplicate_named_capture_groups: false,
            unicode_property_escapes: false,
            look_behind_assertions: false,
            pattern_modifiers: false,
        };
        assert!(!has_unsupported_regular_expression_pattern(&pattern, &supported));

        let unsupported =
            RegexUnsupportedPatterns { duplicate_named_capture_groups: true, ..supported };
        assert!(has_unsupported_regular_expression_pattern(&pattern, &unsupported));
    }
