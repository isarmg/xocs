
    use super::*;
    use crate::{GraphemeClusterSegmenter, LineSegmenter, WordSegmenter};
    use icu_provider::prelude::*;

    #[test]
    fn burmese_dictionary_test() {
        let segmenter = LineSegmenter::new_dictionary(Default::default());
        // From css/css-text/word-break/word-break-normal-my-000.html
        let s = "မြန်မာစာမြန်မာစာမြန်မာစာ";
        let result: Vec<usize> = segmenter.segment_str(s).collect();
        assert_eq!(result, vec![0, 18, 24, 42, 48, 66, 72]);

        let s_utf16: Vec<u16> = s.encode_utf16().collect();
        let result: Vec<usize> = segmenter.segment_utf16(&s_utf16).collect();
        assert_eq!(result, vec![0, 6, 8, 14, 16, 22, 24]);
    }

    #[test]
    fn cj_dictionary_test() {
        let response: DataResponse<SegmenterDictionaryAutoV1> = Baked
            .load(DataRequest {
                id: DataIdentifierBorrowed::for_marker_attributes(
                    DataMarkerAttributes::from_str_or_panic("cjdict"),
                ),
                ..Default::default()
            })
            .unwrap();
        let word_segmenter = WordSegmenter::new_dictionary(Default::default());
        let dict_segmenter =
            DictionarySegmenter::new(response.payload.get(), GraphemeClusterSegmenter::new());

        // Match case
        let s = "龟山岛龟山岛";
        let result: Vec<usize> = dict_segmenter.segment_str(s).collect();
        assert_eq!(result, vec![9, 18]);

        let result: Vec<usize> = word_segmenter.segment_str(s).collect();
        assert_eq!(result, vec![0, 9, 18]);

        let s_utf16: Vec<u16> = s.encode_utf16().collect();
        let result: Vec<usize> = dict_segmenter.segment_utf16(&s_utf16).collect();
        assert_eq!(result, vec![3, 6]);

        let result: Vec<usize> = word_segmenter.segment_utf16(&s_utf16).collect();
        assert_eq!(result, vec![0, 3, 6]);

        // Match case, then no match case
        let s = "エディターエディ";
        let result: Vec<usize> = dict_segmenter.segment_str(s).collect();
        assert_eq!(result, vec![15, 24]);

        // TODO(#3236): Why is WordSegmenter not returning the middle segment?
        let result: Vec<usize> = word_segmenter.segment_str(s).collect();
        assert_eq!(result, vec![0, 24]);

        let s_utf16: Vec<u16> = s.encode_utf16().collect();
        let result: Vec<usize> = dict_segmenter.segment_utf16(&s_utf16).collect();
        assert_eq!(result, vec![5, 8]);

        // TODO(#3236): Why is WordSegmenter not returning the middle segment?
        let result: Vec<usize> = word_segmenter.segment_utf16(&s_utf16).collect();
        assert_eq!(result, vec![0, 8]);
    }

    #[test]
    fn khmer_dictionary_test() {
        let segmenter = LineSegmenter::new_dictionary(Default::default());
        let s = "ភាសាខ្មែរភាសាខ្មែរភាសាខ្មែរ";
        let result: Vec<usize> = segmenter.segment_str(s).collect();
        assert_eq!(result, vec![0, 27, 54, 81]);

        let s_utf16: Vec<u16> = s.encode_utf16().collect();
        let result: Vec<usize> = segmenter.segment_utf16(&s_utf16).collect();
        assert_eq!(result, vec![0, 9, 18, 27]);
    }

    #[test]
    fn lao_dictionary_test() {
        let segmenter = LineSegmenter::new_dictionary(Default::default());
        let s = "ພາສາລາວພາສາລາວພາສາລາວ";
        let r: Vec<usize> = segmenter.segment_str(s).collect();
        assert_eq!(r, vec![0, 12, 21, 33, 42, 54, 63]);

        let s_utf16: Vec<u16> = s.encode_utf16().collect();
        let r: Vec<usize> = segmenter.segment_utf16(&s_utf16).collect();
        assert_eq!(r, vec![0, 4, 7, 11, 14, 18, 21]);
    }

    #[test]
    fn test_dictionary_grapheme_rewind() {
        let response: DataResponse<SegmenterDictionaryAutoV1> = Baked
            .load(DataRequest {
                id: DataIdentifierBorrowed::for_marker_attributes(
                    DataMarkerAttributes::from_str_or_panic("cjdict"),
                ),
                ..Default::default()
            })
            .unwrap();
        let dict_segmenter =
            DictionarySegmenter::new(response.payload.get(), GraphemeClusterSegmenter::new());

        // Test that grapheme_iter is correctly rewound when trie traversal backtracks.
        // In "エディターエディター":
        // 1. "エディター" (15 bytes) is matched as an Intermediate dictionary word at byte 15.
        // 2. Trie traversal continues exploring whether the compound prefix "エディターエディ..."
        //    forms a longer dictionary word, advancing grapheme_iter past byte 15 in the process.
        // 3. When trie traversal eventually hits NoMatch, self.iter rewinds to the last match
        //    at byte 15.
        // 4. Without rewinding grapheme_iter alongside self.iter, on the next call to next()
        //    (starting from byte 15), grapheme_iter is already positioned ahead of byte 15.
        //    When the second word's prefix "エディ" hits an Intermediate match at byte 24,
        //    the grapheme boundary check fails because grapheme_iter skips past 24, causing
        //    incorrect segmentation.
        let s = "エディターエディター";
        let result: Vec<usize> = dict_segmenter.segment_str(s).collect();
        assert_eq!(result, vec![15, 30]);
    }
