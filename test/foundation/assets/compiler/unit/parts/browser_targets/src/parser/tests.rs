
    use super::*;

    #[test]
    fn parse_browserslist_query_empty() {
        let result = parse_browserslist_query("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_browserslist_query_whitespace_only() {
        let result = parse_browserslist_query("   ");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_float_leading_dot() {
        let mut parser = Parser::new(".5");
        let result = parser.parse_float();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 0.5);
    }

    #[test]
    fn parse_double_leading_dot() {
        let mut parser = Parser::new(".25");
        let result = parser.parse_double();
        assert!(result.is_some());
        assert!((result.unwrap() - 0.25).abs() < 0.001);
    }

    #[test]
    fn parse_version_basic() {
        let mut parser = Parser::new("1.2.3");
        let result = parser.parse_version();
        assert_eq!(result, Some("1.2.3"));
    }

    #[test]
    fn parse_version_empty() {
        let mut parser = Parser::new("abc");
        let result = parser.parse_version();
        assert_eq!(result, None);
    }

    #[test]
    fn parse_version_trailing_dot() {
        let mut parser = Parser::new("2.");
        let result = parser.parse_version();
        assert_eq!(result, Some("2"));
    }

    #[test]
    fn parse_version_leading_dot() {
        let mut parser = Parser::new(".7");
        let result = parser.parse_version();
        assert_eq!(result, None);
    }

    #[test]
    fn parse_identifier_basic() {
        let mut parser = Parser::new("chrome");
        let result = parser.parse_identifier();
        assert_eq!(result, Some("chrome"));
    }

    #[test]
    fn parse_identifier_with_underscore() {
        let mut parser = Parser::new("op_mini");
        let result = parser.parse_identifier();
        assert_eq!(result, Some("op_mini"));
    }

    #[test]
    fn parse_comparator_all_types() {
        let mut p1 = Parser::new("<");
        assert!(matches!(p1.parse_comparator(), Some(Comparator::Less)));

        let mut p2 = Parser::new("<=");
        assert!(matches!(p2.parse_comparator(), Some(Comparator::LessOrEqual)));

        let mut p3 = Parser::new(">");
        assert!(matches!(p3.parse_comparator(), Some(Comparator::Greater)));

        let mut p4 = Parser::new(">=");
        assert!(matches!(p4.parse_comparator(), Some(Comparator::GreaterOrEqual)));

        let mut p5 = Parser::new("x");
        assert!(p5.parse_comparator().is_none());
    }

    #[test]
    fn parse_region_basic() {
        let mut parser = Parser::new("US");
        let result = parser.parse_region();
        assert_eq!(result, Some("US"));
    }

    #[test]
    fn parse_region_alt_prefix() {
        let mut parser = Parser::new("alt-AS");
        let result = parser.parse_region();
        assert_eq!(result, Some("alt-AS"));
    }

    #[test]
    fn parse_region_invalid() {
        let mut parser = Parser::new("1X");
        let result = parser.parse_region();
        assert_eq!(result, None);
    }

    #[test]
    fn parse_i32_negative() {
        let mut parser = Parser::new("-2020");
        let result = parser.parse_i32();
        assert_eq!(result, Some(-2020));
    }

    #[test]
    fn parse_i32_positive_with_sign() {
        let mut parser = Parser::new("+2020");
        let result = parser.parse_i32();
        assert_eq!(result, Some(2020));
    }

    #[test]
    fn match_keyword_word_boundary() {
        let mut parser = Parser::new("lastX");
        assert!(!parser.match_keyword(b"last"));
    }

    #[test]
    fn match_keyword_too_short() {
        let mut parser = Parser::new("la");
        assert!(!parser.match_keyword(b"last"));
    }

    #[test]
    fn match_version_keyword_singular() {
        let mut parser = Parser::new("version");
        assert!(parser.match_version_keyword());
    }

    #[test]
    fn match_version_keyword_plural() {
        let mut parser = Parser::new("versions");
        assert!(parser.match_version_keyword());
    }

    #[test]
    fn match_year_keyword_singular() {
        let mut parser = Parser::new("year");
        assert!(parser.match_year_keyword());
    }

    #[test]
    fn match_year_keyword_plural() {
        let mut parser = Parser::new("years");
        assert!(parser.match_year_keyword());
    }

    #[test]
    fn match_year_keyword_invalid() {
        let mut parser = Parser::new("yearly");
        assert!(!parser.match_year_keyword());
    }

    #[test]
    fn parse_version_range_bounded() {
        let mut parser = Parser::new(" 1.0 - 2.0");
        let result = parser.parse_version_range();
        assert!(matches!(result, Some(VersionRange::Bounded("1.0", "2.0"))));
    }

    #[test]
    fn parse_version_range_unbounded() {
        let mut parser = Parser::new(" >= 1.0");
        let result = parser.parse_version_range();
        assert!(matches!(result, Some(VersionRange::Unbounded(Comparator::GreaterOrEqual, "1.0"))));
    }

    #[test]
    fn parse_version_range_accurate() {
        let mut parser = Parser::new(" 1.0");
        let result = parser.parse_version_range();
        assert!(matches!(result, Some(VersionRange::Accurate("1.0"))));
    }

    #[test]
    fn parse_version_range_no_space() {
        let mut parser = Parser::new("1.0");
        let result = parser.parse_version_range();
        assert!(result.is_none());
    }

    #[test]
    fn parse_query_atom_defaults() {
        let mut parser = Parser::new("defaults");
        let result = parser.parse_query_atom();
        assert!(matches!(result, Some(QueryAtom::Defaults)));
    }

    #[test]
    fn parse_query_atom_dead() {
        let mut parser = Parser::new("dead");
        let result = parser.parse_query_atom();
        assert!(matches!(result, Some(QueryAtom::Dead)));
    }

    #[test]
    fn parse_query_atom_percentage() {
        let mut parser = Parser::new("> 1%");
        let result = parser.parse_query_atom();
        assert!(matches!(result, Some(QueryAtom::Percentage { .. })));
    }

    #[test]
    fn parse_query_atom_browser() {
        let mut parser = Parser::new("chrome 90");
        let result = parser.parse_query_atom();
        assert!(matches!(result, Some(QueryAtom::Browser(_, _))));
    }

    #[test]
    fn parse_unknown_query() {
        let mut parser = Parser::new("###invalid###");
        let result = parser.parse_unknown();
        assert!(matches!(result, Some(QueryAtom::Unknown("###invalid###"))));
    }

    #[test]
    fn parse_browserslist_with_or() {
        let result = parse_browserslist_query("ie 10 or ie 11");
        assert!(result.is_ok());
        let queries = result.unwrap();
        assert_eq!(queries.len(), 2);
        assert!(!queries[1].is_and);
    }

    #[test]
    fn parse_browserslist_with_and() {
        let result = parse_browserslist_query("ie >= 10 and ie <= 11");
        assert!(result.is_ok());
        let queries = result.unwrap();
        assert_eq!(queries.len(), 2);
        assert!(queries[1].is_and);
    }

    #[test]
    fn parse_browserslist_with_not() {
        let result = parse_browserslist_query("ie >= 10, not ie 11");
        assert!(result.is_ok());
        let queries = result.unwrap();
        assert_eq!(queries.len(), 2);
        assert!(queries[1].negated);
    }

    // ================================================================
    // Tests for uncovered error paths
    // ================================================================

    #[test]
    fn match_bytes_no_match() {
        let mut parser = Parser::new("abc");
        assert!(!parser.match_bytes(b"xyz"));
    }

    #[test]
    fn match_bytes_too_short() {
        let mut parser = Parser::new("a");
        assert!(!parser.match_bytes(b"abc"));
    }

    #[test]
    fn match_version_keyword_too_short() {
        let mut parser = Parser::new("ver");
        assert!(!parser.match_version_keyword());
    }

    #[test]
    fn match_version_keyword_wrong_word() {
        let mut parser = Parser::new("verbose");
        assert!(!parser.match_version_keyword());
    }

    #[test]
    fn match_year_keyword_too_short() {
        let mut parser = Parser::new("ye");
        assert!(!parser.match_year_keyword());
    }

    #[test]
    fn parse_u16_non_digit() {
        let mut parser = Parser::new("abc");
        assert!(parser.parse_u16().is_none());
    }

    #[test]
    fn parse_u16_empty() {
        let mut parser = Parser::new("");
        assert!(parser.parse_u16().is_none());
    }

    #[test]
    fn parse_u32_non_digit() {
        let mut parser = Parser::new("xyz");
        assert!(parser.parse_u32().is_none());
    }

    #[test]
    fn parse_u32_empty() {
        let mut parser = Parser::new("");
        assert!(parser.parse_u32().is_none());
    }

    #[test]
    fn parse_region_single_char() {
        let mut parser = Parser::new("U");
        assert!(parser.parse_region().is_none());
    }

    #[test]
    fn parse_region_non_alpha() {
        let mut parser = Parser::new("12");
        assert!(parser.parse_region().is_none());
    }

    #[test]
    fn parse_version_range_bounded_incomplete() {
        // Has "from" version but missing "to" version after dash
        let mut parser = Parser::new(" 1.0 - ");
        let result = parser.parse_version_range();
        // Falls back to accurate version
        assert!(matches!(result, Some(VersionRange::Accurate("1.0"))));
    }

    #[test]
    fn parse_version_range_unbounded_no_version() {
        let mut parser = Parser::new(" >= ");
        let result = parser.parse_version_range();
        assert!(result.is_none());
    }

    // Test parse_last_or_years failure paths
    #[test]
    fn parse_last_no_whitespace_after() {
        let mut parser = Parser::new("last");
        let result = parser.parse_last_or_years();
        assert!(result.is_none());
    }

    #[test]
    fn parse_last_no_number() {
        let mut parser = Parser::new("last abc");
        let result = parser.parse_last_or_years();
        assert!(result.is_none());
    }

    #[test]
    fn parse_last_number_no_whitespace() {
        let mut parser = Parser::new("last 2");
        let result = parser.parse_last_or_years();
        assert!(result.is_none());
    }

    #[test]
    fn parse_last_browser_no_whitespace() {
        let mut parser = Parser::new("last 2 chrome");
        let result = parser.parse_last_or_years();
        assert!(result.is_none());
    }

    #[test]
    fn parse_last_major_no_whitespace() {
        let mut parser = Parser::new("last 2 major");
        let result = parser.parse_last_or_years();
        assert!(result.is_none());
    }

    #[test]
    fn parse_last_no_version_keyword() {
        let mut parser = Parser::new("last 2 xyz");
        let result = parser.parse_last_or_years();
        assert!(result.is_none());
    }

    #[test]
    fn parse_last_major_versions_special() {
        // Test "last 2 major versions" where name is "major"
        let mut parser = Parser::new("last 2 major versions");
        let result = parser.parse_last_or_years();
        assert!(matches!(result, Some(QueryAtom::Last { count: 2, major: true, name: None })));
    }

    // Test parse_unreleased failure paths
    #[test]
    fn parse_unreleased_no_whitespace() {
        let mut parser = Parser::new("unreleased");
        let result = parser.parse_unreleased();
        assert!(result.is_none());
    }

    #[test]
    fn parse_unreleased_browser_no_whitespace() {
        let mut parser = Parser::new("unreleased chrome");
        let result = parser.parse_unreleased();
        assert!(result.is_none());
    }

    #[test]
    fn parse_unreleased_no_version_keyword() {
        let mut parser = Parser::new("unreleased xyz abc");
        let result = parser.parse_unreleased();
        assert!(result.is_none());
    }

    // Test parse_since failure paths
    #[test]
    fn parse_since_no_whitespace() {
        let mut parser = Parser::new("since");
        let result = parser.parse_since();
        assert!(result.is_none());
    }

    #[test]
    fn parse_since_no_year() {
        let mut parser = Parser::new("since abc");
        let result = parser.parse_since();
        assert!(result.is_none());
    }

    #[test]
    fn parse_since_or_supports_empty() {
        let mut parser = Parser::new("");
        let result = parser.parse_since_or_supports();
        assert!(result.is_none());
    }

    #[test]
    fn parse_since_or_supports_fallback() {
        // Starts with 's' but second char is neither 'i' nor 'u'
        let mut parser = Parser::new("safari 10");
        let result = parser.parse_since_or_supports();
        assert!(result.is_none()); // Falls through to browser parsing later
    }

    // Test parse_supports failure paths
    #[test]
    fn parse_supports_no_whitespace() {
        let mut parser = Parser::new("supports");
        let result = parser.parse_supports_only();
        assert!(result.is_none());
    }

    #[test]
    fn parse_supports_no_feature() {
        let mut parser = Parser::new("supports ");
        let result = parser.parse_supports_only();
        assert!(result.is_none());
    }

    // Test parse_cover failure paths
    #[test]
    fn parse_cover_no_whitespace() {
        let mut parser = Parser::new("cover");
        let result = parser.parse_cover();
        assert!(result.is_none());
    }

    #[test]
    fn parse_cover_no_number() {
        let mut parser = Parser::new("cover abc");
        let result = parser.parse_cover();
        assert!(result.is_none());
    }

    #[test]
    fn parse_cover_no_percent() {
        let mut parser = Parser::new("cover 50");
        let result = parser.parse_cover();
        assert!(result.is_none());
    }

    #[test]
    fn parse_cover_in_no_whitespace() {
        let mut parser = Parser::new("cover 50% in");
        let result = parser.parse_cover();
        assert!(result.is_none());
    }

    #[test]
    fn parse_cover_in_no_region() {
        let mut parser = Parser::new("cover 50% in 12");
        let result = parser.parse_cover();
        assert!(result.is_none());
    }

    #[test]
    fn parse_cover_or_current_short() {
        let mut parser = Parser::new("c");
        let result = parser.parse_cover_or_current();
        assert!(result.is_none());
    }

    #[test]
    fn parse_cover_or_current_fallback() {
        // Second char is neither 'o' nor 'u'
        let mut parser = Parser::new("chrome 90");
        let result = parser.parse_cover_or_current();
        assert!(result.is_none()); // Falls to browser parsing
    }

    // Test parse_percentage failure paths
    #[test]
    fn parse_percentage_no_number() {
        let mut parser = Parser::new("> abc");
        let result = parser.parse_percentage();
        assert!(result.is_none());
    }

    #[test]
    fn parse_percentage_no_percent() {
        let mut parser = Parser::new("> 50");
        let result = parser.parse_percentage();
        assert!(result.is_none());
    }

    #[test]
    fn parse_percentage_in_no_whitespace() {
        let mut parser = Parser::new("> 50% in");
        let result = parser.parse_percentage();
        assert!(result.is_none());
    }

    #[test]
    fn parse_percentage_in_no_region() {
        let mut parser = Parser::new("> 50% in 12");
        let result = parser.parse_percentage();
        assert!(result.is_none());
    }

    #[test]
    fn parse_electron_no_keyword() {
        let mut parser = Parser::new("electronics");
        let result = parser.parse_electron();
        assert!(result.is_none());
    }

    // Test parse_node failure paths
    #[test]
    fn parse_node_no_keyword() {
        let mut parser = Parser::new("nodejs");
        let result = parser.parse_node();
        assert!(result.is_none());
    }

    #[test]
    fn parse_node_no_version() {
        let mut parser = Parser::new("node");
        let result = parser.parse_node();
        assert!(result.is_none());
    }

    // Test parse_firefox_or_fully failure paths
    #[test]
    fn parse_firefox_or_fully_short() {
        let mut parser = Parser::new("f");
        let result = parser.parse_firefox_or_fully();
        assert!(result.is_none());
    }

    #[test]
    fn parse_firefox_or_fully_fallback() {
        // Second char is not 'i', 'x', 'f', or 'u'
        let mut parser = Parser::new("fa");
        let result = parser.parse_firefox_or_fully();
        assert!(result.is_none());
    }

    #[test]
    fn parse_firefox_esr_no_keyword() {
        let mut parser = Parser::new("fire");
        let result = parser.parse_firefox_esr();
        assert!(result.is_none());
    }

    #[test]
    fn parse_firefox_esr_no_whitespace() {
        let mut parser = Parser::new("firefox");
        let result = parser.parse_firefox_esr();
        assert!(result.is_none());
    }

    #[test]
    fn parse_firefox_esr_no_esr() {
        let mut parser = Parser::new("firefox xyz");
        let result = parser.parse_firefox_esr();
        assert!(result.is_none());
    }

    #[test]
    fn parse_fully_supports_no_whitespace() {
        let mut parser = Parser::new("fully");
        let result = parser.parse_fully_supports();
        assert!(result.is_none());
    }

    #[test]
    fn parse_fully_supports_no_supports() {
        let mut parser = Parser::new("fully abc");
        let result = parser.parse_fully_supports();
        assert!(result.is_none());
    }

    #[test]
    fn parse_fully_supports_supports_no_whitespace() {
        let mut parser = Parser::new("fully supports");
        let result = parser.parse_fully_supports();
        assert!(result.is_none());
    }

    #[test]
    fn parse_fully_supports_no_feature() {
        let mut parser = Parser::new("fully supports ");
        let result = parser.parse_fully_supports();
        assert!(result.is_none());
    }

    // Test parse_operamini failure paths
    #[test]
    fn parse_operamini_no_whitespace() {
        let mut parser = Parser::new("operamini");
        let result = parser.parse_operamini();
        assert!(result.is_none());
    }

    #[test]
    fn parse_operamini_no_all() {
        let mut parser = Parser::new("operamini xyz");
        let result = parser.parse_operamini();
        assert!(result.is_none());
    }

    // Test parse_current_node failure paths
    #[test]
    fn parse_current_node_partial() {
        let mut parser = Parser::new("current");
        let result = parser.parse_current_node();
        assert!(result.is_none());
    }

    #[test]
    fn parse_current_node_no_node() {
        let mut parser = Parser::new("current xyz");
        let result = parser.parse_current_node();
        assert!(result.is_none());
    }

    // Test parse_maintained_node failure paths
    #[test]
    fn parse_maintained_node_partial() {
        let mut parser = Parser::new("maintained");
        let result = parser.parse_maintained_node();
        assert!(result.is_none());
    }

    // Test parse_phantom_or_partially failure paths
    #[test]
    fn parse_phantom_or_partially_short() {
        let mut parser = Parser::new("p");
        let result = parser.parse_phantom_or_partially();
        assert!(result.is_none());
    }

    #[test]
    fn parse_phantom_or_partially_fallback() {
        // Second char is neither 'h' nor 'a'
        let mut parser = Parser::new("px");
        let result = parser.parse_phantom_or_partially();
        assert!(result.is_none());
    }

    #[test]
    fn parse_phantom_no_whitespace() {
        let mut parser = Parser::new("phantomjs");
        let result = parser.parse_phantom();
        assert!(result.is_none());
    }

    #[test]
    fn parse_phantom_wrong_version() {
        let mut parser = Parser::new("phantomjs 3.0");
        let result = parser.parse_phantom();
        assert!(result.is_none());
    }

    #[test]
    fn parse_partially_supports_no_whitespace() {
        let mut parser = Parser::new("partially");
        let result = parser.parse_partially_supports();
        assert!(result.is_none());
    }

    #[test]
    fn parse_partially_supports_no_supports() {
        let mut parser = Parser::new("partially abc");
        let result = parser.parse_partially_supports();
        assert!(result.is_none());
    }

    #[test]
    fn parse_partially_supports_supports_no_whitespace() {
        let mut parser = Parser::new("partially supports");
        let result = parser.parse_partially_supports();
        assert!(result.is_none());
    }

    #[test]
    fn parse_partially_supports_no_feature() {
        let mut parser = Parser::new("partially supports ");
        let result = parser.parse_partially_supports();
        assert!(result.is_none());
    }

    // Test parse_defaults_or_dead failure paths
    #[test]
    fn parse_defaults_or_dead_short() {
        let mut parser = Parser::new("de");
        let result = parser.parse_defaults_or_dead();
        assert!(result.is_none());
    }

    #[test]
    fn parse_defaults_or_dead_fallback() {
        // Third char is neither 'f' nor 'a' - test the fallback branch
        let mut parser = Parser::new("dex");
        let result = parser.parse_defaults_or_dead();
        assert!(result.is_none());
    }

    #[test]
    fn parse_defaults_or_dead_def_but_not_defaults() {
        let mut parser = Parser::new("default"); // Missing 's'
        let result = parser.parse_defaults_or_dead();
        assert!(result.is_none());
    }

    #[test]
    fn parse_defaults_or_dead_dea_but_not_dead() {
        let mut parser = Parser::new("dealer");
        let result = parser.parse_defaults_or_dead();
        assert!(result.is_none());
    }

    // Test parse_browser failure paths
    #[test]
    fn parse_browser_no_version() {
        let mut parser = Parser::new("chrome");
        let result = parser.parse_browser();
        assert!(result.is_none());
    }

    // Test parse_query_atom edge cases
    #[test]
    fn parse_query_atom_empty() {
        let mut parser = Parser::new("");
        let result = parser.parse_query_atom();
        assert!(result.is_none());
    }

    #[test]
    fn parse_query_atom_non_letter() {
        let mut parser = Parser::new("123");
        let result = parser.parse_query_atom();
        assert!(result.is_none());
    }

    #[test]
    fn parse_query_atom_special_char() {
        let mut parser = Parser::new("@#$");
        let result = parser.parse_query_atom();
        assert!(result.is_none());
    }

    // Test at_composition_operator paths
    #[test]
    fn at_composition_operator_empty() {
        let parser = Parser::new("");
        assert!(!parser.at_composition_operator());
    }

    #[test]
    fn at_composition_operator_comma() {
        let parser = Parser::new(",");
        assert!(parser.at_composition_operator());
    }

    #[test]
    fn at_composition_operator_no_space() {
        let parser = Parser::new("x");
        assert!(!parser.at_composition_operator());
    }

    #[test]
    fn at_composition_operator_space_then_and() {
        let parser = Parser::new(" and ");
        assert!(parser.at_composition_operator());
    }

    #[test]
    fn at_composition_operator_space_then_or() {
        let parser = Parser::new(" or ");
        assert!(parser.at_composition_operator());
    }

    #[test]
    fn at_composition_operator_space_then_other() {
        let parser = Parser::new(" xyz");
        assert!(!parser.at_composition_operator());
    }

    // Test parse_unknown with composition operator
    #[test]
    fn parse_unknown_stops_at_comma() {
        let mut parser = Parser::new("abc,def");
        let result = parser.parse_unknown();
        assert!(matches!(result, Some(QueryAtom::Unknown("abc"))));
    }

    #[test]
    fn parse_unknown_stops_at_and() {
        let mut parser = Parser::new("abc and def");
        let result = parser.parse_unknown();
        assert!(matches!(result, Some(QueryAtom::Unknown("abc"))));
    }

    #[test]
    fn parse_unknown_stops_at_or() {
        let mut parser = Parser::new("abc or def");
        let result = parser.parse_unknown();
        assert!(matches!(result, Some(QueryAtom::Unknown("abc"))));
    }

    // ================================================================
    // Tests for remaining edge cases
    // ================================================================

    // Test feature name parsing with terminating special characters
    #[test]
    fn parse_supports_feature_with_comma() {
        // Feature name should stop at comma
        let mut parser = Parser::new("supports flexbox,");
        let result = parser.parse_query_atom();
        assert!(matches!(result, Some(QueryAtom::Supports("flexbox", None))));
    }

    #[test]
    fn parse_supports_feature_with_space() {
        // Feature name stops at space before "and"
        let mut parser = Parser::new("supports flexbox and");
        let result = parser.parse_query_atom();
        assert!(matches!(result, Some(QueryAtom::Supports("flexbox", None))));
    }

    #[test]
    fn parse_fully_supports_feature_terminated() {
        let mut parser = Parser::new("fully supports flexbox,");
        let result = parser.parse_query_atom();
        assert!(matches!(result, Some(QueryAtom::Supports("flexbox", Some(SupportKind::Fully)))));
    }

    #[test]
    fn parse_partially_supports_feature_terminated() {
        let mut parser = Parser::new("partially supports flexbox,");
        let result = parser.parse_query_atom();
        assert!(matches!(
            result,
            Some(QueryAtom::Supports("flexbox", Some(SupportKind::Partially)))
        ));
    }

    // Test since_or_supports fallback when second char doesn't match
    #[test]
    fn parse_since_or_supports_with_s_third_char() {
        // Starts with 's', second char is 'a' (not 'i' or 'u'), falls to fallback
        let mut parser = Parser::new("sa");
        let result = parser.parse_since_or_supports();
        assert!(result.is_none());
    }

    // Test that "last N chromex" fails when chromex is followed by invalid keyword
    #[test]
    fn parse_last_browser_invalid_suffix() {
        let mut parser = Parser::new("last 2 chromex abc");
        let result = parser.parse_last_or_years();
        // Should fail because "abc" is not "version(s)" or "major version(s)"
        assert!(result.is_none());
    }

    #[test]
    fn parse_since_or_supports_single_char() {
        // Input is just "s" - only 1 char, so we skip the if block
        let mut parser = Parser::new("s");
        let result = parser.parse_since_or_supports();
        assert!(result.is_none());
    }

    #[test]
    fn parse_baseline_shapes() {
        assert!(matches!(
            Parser::new("baseline widely available").parse_baseline(),
            Some(QueryAtom::Baseline {
                kind: BaselineKind::WidelyAvailable,
                downstream: false,
                kaios: false
            })
        ));

        assert!(matches!(
            Parser::new("baseline newly available").parse_baseline(),
            Some(QueryAtom::Baseline { kind: BaselineKind::NewlyAvailable, .. })
        ));

        assert!(matches!(
            Parser::new("baseline 2020 with downstream including kaios").parse_baseline(),
            Some(QueryAtom::Baseline {
                kind: BaselineKind::Year(2020),
                downstream: true,
                kaios: true
            })
        ));

        assert!(matches!(
            Parser::new("baseline widely available on 2022-07-01").parse_baseline(),
            Some(QueryAtom::Baseline {
                kind: BaselineKind::WidelyAvailableOnDate { year: 2022, month: 7, day: 1 },
                ..
            })
        ));
        assert!(matches!(
            Parser::new("baseline newly available on 2022-07-01").parse_baseline(),
            Some(QueryAtom::Baseline { kind: BaselineKind::NewlyAvailableOnDate, .. })
        ));
    }

    #[test]
    fn reject_invalid_baseline_grammar() {
        for query in ["baseline", "baseline widely available on 2022-7-01"] {
            assert!(Parser::new(query).parse_baseline().is_none());
        }
        assert!(parse_browserslist_query("baseline 2020 including kaios with downstream").is_err());
    }
