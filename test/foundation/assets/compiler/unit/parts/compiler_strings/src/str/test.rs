
    use crate::syntax_arena::Allocator;

    use super::*;

    #[test]
    #[expect(clippy::items_after_statements)]
    fn str_from_str_in() {
        let allocator = Allocator::new();
        let allocator = &allocator;

        // Pass an actual `Allocator`
        let s = Str::from_str_in("world", &allocator);
        assert_eq!(s.as_str(), "world");
        assert_eq!(s, Str::from("world"));

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let s = Str::from_str_in("hello", &wrapper);
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s, Str::from("hello"));
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn str_from_strs_array_in() {
        let allocator = Allocator::new();
        let allocator = &allocator;

        // Pass an actual `Allocator`
        let s = Str::from_strs_array_in(["hello", " ", "world", "!"], &allocator);
        assert_eq!(s.as_str(), "hello world!");
        assert_eq!(s, Str::from("hello world!"));

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let s = Str::from_strs_array_in(["foo", "_", "bar"], &wrapper);
        assert_eq!(s.as_str(), "foo_bar");
        assert_eq!(s, Str::from("foo_bar"));
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn str_from_cow_in() {
        let allocator = Allocator::new();
        let allocator = &allocator;

        // `Cow::Borrowed` references the same string, without allocating in arena
        let borrowed = "world";
        let used_before = allocator.used_bytes();
        let s = Str::from_cow_in(&Cow::Borrowed(borrowed), &allocator);
        assert_eq!(s.as_str(), "world");
        assert_eq!(s, Str::from("world"));
        assert_eq!(s.as_str().as_ptr(), borrowed.as_ptr());
        assert_eq!(allocator.used_bytes(), used_before);

        // `Cow::Owned` allocates a new string in arena
        let owned = "owned".to_string();
        let owned_ptr = owned.as_ptr();
        let s = Str::from_cow_in(&Cow::Owned(owned), &allocator);
        assert_eq!(s.as_str(), "owned");
        assert_eq!(s, Str::from("owned"));
        assert_ne!(s.as_str().as_ptr(), owned_ptr);
        assert!(allocator.used_bytes() > used_before);

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let s = Str::from_cow_in(&Cow::Borrowed("hello"), &wrapper);
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s, Str::from("hello"));
    }
