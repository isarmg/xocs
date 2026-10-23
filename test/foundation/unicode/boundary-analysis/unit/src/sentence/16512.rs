#[test]
fn empty_string_neo() {
    let segmenter = SentenceSegmenter::new_neo(Default::default());
    let breaks: Vec<usize> = segmenter.segment_str("").collect();
    assert_eq!(breaks, [0]);
}
