
    use super::*;
    use crate::compiler_strings::static_ident;

    fn options(pattern: &str) -> ManglePropertiesOptions {
        ManglePropertiesOptions::new(lazy_regex::Regex::new(pattern).unwrap())
    }

    #[test]
    fn collection_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<PropertyMangleCollection<'static>>();
    }

    #[test]
    fn assignment_prefers_frequency_then_name() {
        let allocator = Allocator::default();
        let options = options("^_");
        let mut state = PropertyCollectState::new_in(&allocator);
        state.frequencies.extend([
            (static_ident!("_rare"), 1),
            (static_ident!("_often"), 3),
            (static_ident!("_also_often"), 3),
        ]);
        let (mapping, _) = assign(&options, &state, &allocator);
        assert_eq!(mapping["_also_often"].as_str(), "e");
        assert_eq!(mapping["_often"].as_str(), "t");
        assert_eq!(mapping["_rare"].as_str(), "n");
    }

    #[test]
    fn assignment_honors_cache_and_allows_duplicate_targets() {
        let allocator = Allocator::default();
        let mut options = options("^_");
        options.cache.insert("_a".into(), Some("A".into())).unwrap();
        options.cache.insert("_b".into(), Some("A".into())).unwrap();
        options.cache.insert("_keep".into(), None).unwrap();
        let mut state = PropertyCollectState::new_in(&allocator);
        state.frequencies.extend([
            (static_ident!("_a"), 1),
            (static_ident!("_b"), 1),
            (static_ident!("_auto"), 1),
        ]);
        state.occupied.insert(static_ident!("_keep"));
        let (mapping, cache) = assign(&options, &state, &allocator);
        assert_eq!(mapping["_a"].as_str(), "A");
        assert_eq!(mapping["_b"].as_str(), "A");
        assert_ne!(mapping["_auto"].as_str(), "A");
        assert_eq!(cache.get("_keep"), Some(&None));
    }

    #[test]
    fn assignment_filters_ineligible_collected_names() {
        let allocator = Allocator::default();
        let mut options = options("^(?:_|1$|constructor$)");
        options.exclude = Some(lazy_regex::Regex::new("^_excluded(?:One|Two)$").unwrap());
        options.reserved.insert("_reserved".into());
        options.cache.insert("_cached".into(), None).unwrap();
        let mut state = PropertyCollectState::new_in(&allocator);
        state.frequencies.extend([
            (static_ident!("_mangle"), 1),
            (static_ident!("_excludedOne"), 10),
            (static_ident!("_reserved"), 10),
            (static_ident!("_cached"), 10),
            (static_ident!("public"), 10),
            (static_ident!("1"), 10),
            (static_ident!("constructor"), 10),
        ]);
        let (mapping, cache) = assign(&options, &state, &allocator);
        assert_eq!(mapping.len(), 1);
        assert!(mapping.contains_key("_mangle"));
        assert_eq!(cache.get("_cached"), Some(&None));
    }

    #[test]
    fn hard_reservations_are_exact() {
        assert!(is_hard_reserved("__proto__"));
        assert!(is_hard_reserved("constructor"));
        assert!(is_hard_reserved("prototype"));
        assert!(!is_hard_reserved("toString"));
        assert!(!is_hard_reserved("then"));
    }

    #[test]
    fn cache_targets_are_identifier_names() {
        assert!(is_valid_property_mangle_cache_target("valid$name"));
        assert!(!is_valid_property_mangle_cache_target("not-valid"));
        assert!(!is_valid_property_mangle_cache_target("__proto__"));
        assert!(!is_valid_property_mangle_cache_target("constructor"));
        assert!(!is_valid_property_mangle_cache_target("prototype"));
    }

    #[test]
    fn invalid_cache_targets_cannot_enter_the_rust_engine() {
        let mut cache = ManglePropertyCache::default();
        for target in ["a-b", "__proto__", "constructor", "prototype", ""] {
            let error = cache.insert("_field".into(), Some(target.into())).unwrap_err();
            assert_eq!(error.original.as_str(), "_field");
            assert_eq!(error.target.as_str(), target);
            assert!(cache.is_empty());
        }
        cache.insert("_field".into(), Some("valid$name".into())).unwrap();
        assert_eq!(cache.get("_field").and_then(Option::as_deref), Some("valid$name"));
    }
