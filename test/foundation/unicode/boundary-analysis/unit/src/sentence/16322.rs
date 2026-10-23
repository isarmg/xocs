#[test]
fn empty_string() {
    let segmenter = SentenceSegmenter::new(Default::default());
    let breaks: Vec<usize> = segmenter.segment_str("").collect();
    assert_eq!(breaks, [0]);
}
