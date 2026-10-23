
    use super::{KNOWN_GLOBAL_IDENTIFIERS, is_known_global_identifier};

    #[test]
    fn known_global_identifiers_table_is_sorted_and_unique() {
        // `binary_search` relies on byte-order sorting and duplicates are pointless.
        for pair in KNOWN_GLOBAL_IDENTIFIERS.windows(2) {
            assert!(pair[0] < pair[1], "`{}` must sort before `{}`", pair[0], pair[1]);
        }
    }

    #[test]
    fn known_global_identifiers_lookup() {
        assert!(is_known_global_identifier("Array"));
        assert!(is_known_global_identifier("window"));
        assert!(is_known_global_identifier("AbortController"));
        assert!(is_known_global_identifier("webkitURL"));
        assert!(!is_known_global_identifier("NaN"));
        assert!(!is_known_global_identifier("foo"));
        assert!(!is_known_global_identifier(""));
    }
