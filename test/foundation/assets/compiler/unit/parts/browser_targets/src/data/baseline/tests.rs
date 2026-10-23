
    use super::*;

    #[test]
    fn selects_events() {
        assert!(min_versions_on(20_150_728).is_none());
        let mut versions = min_versions_on(20_150_729).unwrap();
        assert_eq!(versions.len(), 14);
        assert!(versions.any(|(browser, version)| browser == "edge" && version == "12"));
        assert_eq!(min_versions_on(u64::MAX).unwrap().len(), 14);
    }
