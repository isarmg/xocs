
    use std::hash::BuildHasher;

    use crate::syntax_arena::Allocator;
    use oxc_data_structures::types::implements;

    use super::*;

    #[test]
    fn ident_size() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(std::mem::size_of::<Ident<'_>>(), 16);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(std::mem::size_of::<Ident<'_>>(), 12);
    }

    #[test]
    fn ident_send_sync() {
        assert!(implements!(Ident: Send));
        assert!(implements!(Ident: Sync));
    }

    #[test]
    fn ident_copy() {
        assert!(implements!(Ident: Copy));
    }

    #[test]
    fn ident_from_str() {
        let s = "hello";
        let ident = Ident::from(s);
        assert_eq!(ident.as_str(), "hello");
    }

    #[test]
    fn ident_empty() {
        let ident = Ident::empty();
        assert_eq!(ident.as_str(), "");
        assert_eq!(ident.ident_len(), 0);
    }

    #[test]
    fn ident_new_const() {
        let ident = new_const_ident("world");
        assert_eq!(ident.as_str(), "world");
    }

    #[test]
    fn ident_eq() {
        let a = Ident::from("foo");
        let b = Ident::from("foo");
        let c = Ident::from("bar");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn ident_eq_str() {
        let ident = Ident::from("hello");
        assert_eq!(ident, "hello");
        assert_ne!(ident, "world");
    }

    #[test]
    fn ident_hash_consistency() {
        let build_hasher = IdentBuildHasher;

        // Hash an Ident via BuildHasher::hash_one
        let ident = Ident::from("fooBar");
        let ident_hash_val = build_hasher.hash_one(ident);

        // Hash a &str through the same hasher
        let str_hash = build_hasher.hash_one("fooBar");

        assert_eq!(ident_hash_val, str_hash);
    }

    #[test]
    fn ident_hashmap_str_lookup() {
        let mut map = IdentHashMap::default();
        map.insert(Ident::from("key1"), 1);
        map.insert(Ident::from("key2"), 2);

        // Lookup with &str via Equivalent
        assert_eq!(map.get("key1"), Some(&1));
        assert_eq!(map.get("key2"), Some(&2));
        assert_eq!(map.get("key3"), None);
    }

    #[test]
    fn ident_hashmap_ident_lookup() {
        let mut map = IdentHashMap::default();
        map.insert(Ident::from("key1"), 1);

        let key = Ident::from("key1");
        assert_eq!(map.get(&key), Some(&1));
    }

    #[test]
    fn ident_clone_in() {
        let allocator = Allocator::new();
        let original = Ident::from("test");
        let cloned = original.clone_in(&allocator);
        assert_eq!(original.as_str(), cloned.as_str());
        assert_eq!(original, cloned);
        // Verify hash is preserved (not recomputed from different pointer)
        assert_eq!(original.ident_hash_value(), cloned.ident_hash_value());
    }

    #[test]
    fn ident_deref() {
        let ident = Ident::from("hello");
        // Should be able to call str methods
        assert!(ident.starts_with("hel")); // spellchecker:disable-line
        assert_eq!(ident.len(), 5);
    }

    #[test]
    fn ident_display() {
        let ident = Ident::from("test");
        assert_eq!(format!("{ident}"), "test");
    }

    #[test]
    fn ident_debug() {
        let ident = Ident::from("test");
        assert_eq!(format!("{ident:?}"), "\"test\"");
    }

    #[test]
    fn static_ident_correct() {
        let ident = static_ident!("require");
        assert_eq!(ident.as_str(), "require");
        assert_eq!(ident, Ident::from("require"));
    }

    #[test]
    fn static_ident_const_context() {
        const IDENT: Ident<'static> = static_ident!("hello");
        assert_eq!(IDENT.as_str(), "hello");
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn ident_from_str_in() {
        let allocator = Allocator::new();
        let allocator: &Allocator = &allocator;

        // Pass an actual `Allocator`
        let ident = Ident::from_str_in("world", &allocator);
        assert_eq!(ident.as_str(), "world");
        assert_eq!(ident, Ident::from("world"));

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let ident = Ident::from_str_in("hello", &wrapper);
        assert_eq!(ident.as_str(), "hello");
        assert_eq!(ident, Ident::from("hello"));
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn ident_from_strs_array_in() {
        let allocator = Allocator::new();
        let allocator = &allocator;

        // Pass an actual `Allocator`
        let ident = Ident::from_strs_array_in(["hello", " ", "world", "!"], &allocator);
        assert_eq!(ident.as_str(), "hello world!");
        assert_eq!(ident, Ident::from("hello world!"));

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let ident = Ident::from_strs_array_in(["foo", "_", "bar"], &wrapper);
        assert_eq!(ident.as_str(), "foo_bar");
        assert_eq!(ident, Ident::from("foo_bar"));
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn ident_from_cow_in() {
        let allocator = Allocator::new();
        let allocator = &allocator;

        // `Cow::Borrowed` references the same string, without allocating in arena
        let borrowed = "world";
        let used_before = allocator.used_bytes();
        let ident = Ident::from_cow_in(&Cow::Borrowed(borrowed), &allocator);
        assert_eq!(ident.as_str(), "world");
        assert_eq!(ident, Ident::from("world"));
        assert_eq!(ident.as_str().as_ptr(), borrowed.as_ptr());
        assert_eq!(allocator.used_bytes(), used_before);

        // `Cow::Owned` allocates a new string in arena
        let owned = "owned".to_string();
        let owned_ptr = owned.as_ptr();
        let ident = Ident::from_cow_in(&Cow::Owned(owned), &allocator);
        assert_eq!(ident.as_str(), "owned");
        assert_eq!(ident, Ident::from("owned"));
        assert_ne!(ident.as_str().as_ptr(), owned_ptr);
        assert!(allocator.used_bytes() > used_before);

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let ident = Ident::from_cow_in(&Cow::Borrowed("hello"), &wrapper);
        assert_eq!(ident.as_str(), "hello");
        assert_eq!(ident, Ident::from("hello"));
    }

    #[test]
    fn arena_ident_hashmap() {
        let allocator = Allocator::new();
        let mut map = ArenaIdentHashMap::new_in(&allocator);
        let key = Ident::from_in("hello", &allocator);
        map.insert(key, 42);
        assert_eq!(map.get("hello"), Some(&42));
        assert_eq!(map.get(&Ident::from("hello")), Some(&42));
    }

    #[test]
    fn arena_ident_hashset() {
        let allocator = Allocator::new();
        let mut set = ArenaIdentHashSet::new_in(&allocator);
        set.insert(Ident::from_in("hello", &allocator));
        assert!(set.contains("hello"));
        assert!(set.contains(&Ident::from("hello")));
        assert!(!set.contains("world"));

        let set =
            ArenaIdentHashSet::from_iter_in([Ident::from("foo"), Ident::from("bar")], &allocator);
        assert_eq!(set.len(), 2);
        assert!(set.contains("foo"));
        assert!(set.contains("bar"));
    }
