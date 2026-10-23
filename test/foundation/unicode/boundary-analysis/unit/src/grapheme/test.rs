
    use crate::*;

    include!("../../../integration/helpers.rs.raw");

    #[test]
    fn empty_string() {
        let segmenter = GraphemeClusterSegmenter::new();
        let breaks: Vec<usize> = segmenter.segment_str("").collect();
        assert_eq!(breaks, [0]);
    }

    #[test]
    fn emoji_flags() {
        // https://github.com/unicode-org/icu4x/issues/4780
        check_grapheme("🇺🇸🏴󠁧󠁢󠁥󠁮󠁧󠁿", &["🇺🇸", "🏴󠁧󠁢󠁥󠁮󠁧󠁿"], GraphemeClusterSegmenter::new());
    }
    #[test]
    fn empty_string_neo() {
        let segmenter = GraphemeClusterSegmenter::new_neo();
        let breaks: Vec<usize> = segmenter.segment_str("").collect();
        assert_eq!(breaks, [0]);
    }

    #[test]
    fn emoji_flags_neo() {
        // https://github.com/unicode-org/icu4x/issues/4780
        check_grapheme("🇺🇸🏴", &["🇺🇸", "🏴"], GraphemeClusterSegmenter::new_neo());
    }
