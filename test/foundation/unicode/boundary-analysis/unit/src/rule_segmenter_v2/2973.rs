#[test]
fn test_lookahead_positions_stays_on_stack() {
    use crate::provider::Baked;

    for &SegmenterStateMachine { num_lookaheads, .. } in [
        Baked::SINGLETON_SEGMENTER_BREAK_LINE_V2,
        Baked::SINGLETON_SEGMENTER_BREAK_GRAPHEME_CLUSTER_V2,
        Baked::SINGLETON_SEGMENTER_BREAK_SENTENCE_V2,
        Baked::SINGLETON_SEGMENTER_BREAK_WORD_V2,
    ] {
        assert!(num_lookaheads <= 1, "{num_lookaheads}");
    }
}
