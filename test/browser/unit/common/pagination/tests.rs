use super::last_page;

#[test]
fn page_navigation_counts_empty_exact_partial_and_capped_results() {
    for (total, size, expected) in [
        (0, 20, 1),
        (20, 20, 1),
        (21, 20, 2),
        (40, 20, 2),
        (1_000_001, 10, 100_000),
        (9_007_199_254_740_991, 100, 100_000),
    ] {
        assert_eq!(last_page(total, size), expected);
    }
}
