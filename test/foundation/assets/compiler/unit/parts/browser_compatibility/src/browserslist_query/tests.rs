
    use super::BrowserslistQuery;

    #[test]
    fn test_empty() {
        let res = BrowserslistQuery::Single(String::new()).exec().unwrap();
        assert!(!res.is_any_target(), "empty query should return non-empty result");
    }
