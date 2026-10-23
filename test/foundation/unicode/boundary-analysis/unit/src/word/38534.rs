#[test]
fn empty_string_neo() {
    let segmenter =
        WordSegmenter::new_neo_for_non_complex_scripts(WordBreakInvariantOptions::default());
    let breaks: Vec<usize> = segmenter.segment_str("").collect();
    assert_eq!(breaks, [0]);
}
