#[cfg(test)]
macro_rules! assert_contains {
    ($s:expr, $needle:expr) => {{
        let (s, needle) = ($s, $needle);
        assert!(
            s.contains(needle),
            "{:?} should've contained {:?}",
            s,
            needle
        );
    }};
}
