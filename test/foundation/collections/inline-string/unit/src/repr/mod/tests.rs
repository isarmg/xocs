
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    use quickcheck_macros::quickcheck;
    use test_case::test_case;

    use super::{Repr, MAX_SIZE};
    use crate::ReserveError;

    const EIGHTEEN_MB: usize = 18 * 1024 * 1024;
    const EIGHTEEN_MB_STR: &str = unsafe { core::str::from_utf8_unchecked(&[42; EIGHTEEN_MB]) };

    #[test_case("hello world!"; "inline")]
    #[test_case("this is a long string that should be stored on the heap"; "heap")]
    fn test_create(s: &'static str) {
        let repr = Repr::new(s).unwrap();
        assert_eq!(repr.as_str(), s);
        assert_eq!(repr.len(), s.len());

        // test StaticStr variant
        let repr = Repr::const_new(s);
        assert_eq!(repr.as_str(), s);
        assert_eq!(repr.len(), s.len());
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn quickcheck_create(s: String) {
        let repr = Repr::new(&s).unwrap();
        assert_eq!(repr.as_str(), s);
        assert_eq!(repr.len(), s.len());
    }

    #[test_case(0; "empty")]
    #[test_case(10; "short")]
    #[test_case(64; "long")]
    #[test_case(EIGHTEEN_MB; "huge")]
    fn test_with_capacity(cap: usize) {
        let r = Repr::with_capacity(cap).unwrap();
        assert!(r.capacity() >= MAX_SIZE);
        assert_eq!(r.len(), 0);
    }

    #[test_case(""; "empty")]
    #[test_case("abc"; "short")]
    #[test_case("hello world! I am a longer string 🦀"; "long")]
    fn test_from_utf8_valid(s: &'static str) {
        let bytes = s.as_bytes();
        let r = Repr::from_utf8(bytes).expect("valid UTF-8");

        assert_eq!(r.as_str(), s);
        assert_eq!(r.len(), s.len());
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn quickcheck_from_utf8(buf: Vec<u8>) {
        match (core::str::from_utf8(&buf), Repr::from_utf8(&buf)) {
            (Ok(s), Ok(r)) => {
                assert_eq!(r.as_str(), s);
                assert_eq!(r.len(), s.len());
            }
            (Err(e), Err(r)) => assert_eq!(e, r),
            _ => panic!("core::str and Repr differ on what is valid UTF-8!"),
        }
    }

    #[test_case(String::new(), true; "empty should inline")]
    #[test_case(String::new(), false; "empty not inline")]
    #[test_case(String::with_capacity(10), true ; "empty with small capacity inline")]
    #[test_case(String::with_capacity(10), false ; "empty with small capacity not inline")]
    #[test_case(String::with_capacity(128), true ; "empty with large capacity inline")]
    #[test_case(String::with_capacity(128), false ; "empty with large capacity not inline")]
    #[test_case(String::from("nyc 🗽"), true; "short should inline")]
    #[test_case(String::from("nyc 🗽"), false ; "short not inline")]
    #[test_case(String::from("this is a really long string, which is intended"), true; "long")]
    #[test_case(String::from("this is a really long string, which is intended"), false; "long not inline")]
    #[test_case(EIGHTEEN_MB_STR.to_string(), true ; "huge should inline")]
    #[test_case(EIGHTEEN_MB_STR.to_string(), false ; "huge not inline")]
    fn test_from_string(s: String, try_to_inline: bool) {
        // note: when cloning a String it truncates capacity, which is why we measure these values
        // before cloning the string
        let s_len = s.len();
        let s_cap = s.capacity();
        let s_str = s.clone();

        let r = Repr::from_string(s, try_to_inline).unwrap();

        assert_eq!(r.len(), s_len);
        assert_eq!(r.as_str(), s_str.as_str());

        if s_cap == 0 || (try_to_inline && s_len <= MAX_SIZE) {
            // we should inline the string, if we were asked to, and the length of the string would
            // fit inline, meaning we would truncate capacity
            assert!(!r.is_heap_allocated());
        } else {
            assert!(r.is_heap_allocated());
        }
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn quickcheck_from_string(s: String, try_to_inline: bool) {
        let r = Repr::from_string(s.clone(), try_to_inline).unwrap();

        assert_eq!(r.len(), s.len());
        assert_eq!(r.as_str(), s.as_str());

        if s.capacity() == 0 {
            // we should always inline the string, if the length of the source string is 0
            assert!(!r.is_heap_allocated());
        } else if s.capacity() <= MAX_SIZE {
            // we should inline the string, if we were asked to
            assert_eq!(!r.is_heap_allocated(), try_to_inline);
        } else {
            assert!(r.is_heap_allocated());
        }
    }

    #[test_case(""; "empty")]
    #[test_case("nyc 🗽"; "short")]
    #[test_case("this is a really long string, which is intended"; "long")]
    fn test_into_string(control: &'static str) {
        let r = Repr::new(control).unwrap();
        let s = r.into_string();

        assert_eq!(control.len(), s.len());
        assert_eq!(control, s.as_str());

        // test StaticStr variant
        let r = Repr::const_new(control);
        let s = r.into_string();

        assert_eq!(control.len(), s.len());
        assert_eq!(control, s.as_str());
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn quickcheck_into_string(control: String) {
        let r = Repr::new(&control).unwrap();
        let s = r.into_string();

        assert_eq!(control.len(), s.len());
        assert_eq!(control, s.as_str());
    }

    #[test_case("", "a", false; "empty")]
    #[test_case("", "🗽", false; "empty_emoji")]
    #[test_case("abc", "🗽🙂🦀🌈👏🐶", true; "inline_to_heap")]
    #[test_case("i am a long string that will be on the heap", "extra", true; "heap_to_heap")]
    fn test_push_str(control: &'static str, append: &'static str, is_heap: bool) {
        let mut r = Repr::new(control).unwrap();
        let mut c = String::from(control);

        r.push_str(append);
        c.push_str(append);

        assert_eq!(r.as_str(), c.as_str());
        assert_eq!(r.len(), c.len());

        assert_eq!(r.is_heap_allocated(), is_heap);

        // test StaticStr variant
        let mut r = Repr::const_new(control);
        let mut c = String::from(control);

        r.push_str(append);
        c.push_str(append);

        assert_eq!(r.as_str(), c.as_str());
        assert_eq!(r.len(), c.len());

        assert_eq!(r.is_heap_allocated(), is_heap);
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn quickcheck_push_str(control: String, append: String) {
        let mut r = Repr::new(&control).unwrap();
        let mut c = control;

        r.push_str(&append);
        c.push_str(&append);

        assert_eq!(r.as_str(), c.as_str());
        assert_eq!(r.len(), c.len());
    }

    #[test_case(&[42; 0], &[42; EIGHTEEN_MB]; "empty_to_heap_capacity")]
    #[test_case(&[42; 8], &[42; EIGHTEEN_MB]; "inline_to_heap_capacity")]
    #[test_case(&[42; 128], &[42; EIGHTEEN_MB]; "heap_inline_to_heap_capacity")]
    #[test_case(&[42; EIGHTEEN_MB], &[42; 64]; "heap_capacity_to_heap_capacity")]
    fn test_push_str_from_buf(buf: &[u8], append: &[u8]) {
        // The goal of this test is to exercise the scenario when our capacity is stored on the heap

        let control = unsafe { core::str::from_utf8_unchecked(buf) };
        let append = unsafe { core::str::from_utf8_unchecked(append) };

        let mut r = Repr::new(control).unwrap();
        let mut c = String::from(control);

        r.push_str(append);
        c.push_str(append);

        assert_eq!(r.as_str(), c.as_str());
        assert_eq!(r.len(), c.len());

        assert!(r.is_heap_allocated());
    }

    #[test_case("", 0, false; "empty_zero")]
    #[test_case("", 10, false; "empty_small")]
    #[test_case("", 64, true; "empty_large")]
    #[test_case("abc", 0, false; "short_zero")]
    #[test_case("abc", 8, false; "short_small")]
    #[test_case("abc", 64, true; "short_large")]
    #[test_case("I am a long string that will be on the heap", 0, true; "large_zero")]
    #[test_case("I am a long string that will be on the heap", 10, true; "large_small")]
    #[test_case("I am a long string that will be on the heap", EIGHTEEN_MB, true; "large_huge")]
    fn test_reserve(initial: &'static str, additional: usize, is_heap: bool) {
        let mut r = Repr::new(initial).unwrap();
        r.reserve(additional).unwrap();

        assert!(r.capacity() >= initial.len() + additional);
        assert_eq!(r.is_heap_allocated(), is_heap);

        // Test static_str variant
        let mut r = Repr::const_new(initial);
        r.reserve(additional).unwrap();

        assert!(r.capacity() >= initial.len() + additional);
        assert_eq!(r.is_heap_allocated(), is_heap);
    }

    #[test]
    fn test_reserve_overflow() {
        let mut r = Repr::new("abc").unwrap();
        let err = r.reserve(usize::MAX).unwrap_err();
        assert_eq!(err, ReserveError(()));
    }

    #[test_case(""; "empty")]
    #[test_case("abc"; "short")]
    #[test_case("i am a longer string that will be on the heap"; "long")]
    #[test_case(EIGHTEEN_MB_STR; "huge")]
    fn test_clone(initial: &'static str) {
        let r_a = Repr::new(initial).unwrap();
        let r_b = r_a.clone();

        assert_eq!(r_a.as_str(), initial);
        assert_eq!(r_a.len(), initial.len());

        assert_eq!(r_a.as_str(), r_b.as_str());
        assert_eq!(r_a.len(), r_b.len());
        assert_eq!(r_a.capacity(), r_b.capacity());
        assert_eq!(r_a.is_heap_allocated(), r_b.is_heap_allocated());

        // test StaticStr variant
        let r_a = Repr::const_new(initial);
        let r_b = r_a.clone();

        assert_eq!(r_a.as_str(), initial);
        assert_eq!(r_a.len(), initial.len());

        assert_eq!(r_a.as_str(), r_b.as_str());
        assert_eq!(r_a.len(), r_b.len());
        assert_eq!(r_a.capacity(), r_b.capacity());
        assert_eq!(r_a.is_heap_allocated(), r_b.is_heap_allocated());
    }

    #[test_case(Repr::const_new(""), Repr::const_new(""); "empty clone from static")]
    #[test_case(Repr::const_new("abc"), Repr::const_new("efg"); "short clone from static")]
    #[test_case(Repr::new("i am a longer string that will be on the heap").unwrap(), Repr::const_new(EIGHTEEN_MB_STR); "long clone from static")]
    #[test_case(Repr::const_new(""), Repr::const_new(""); "empty clone from inline")]
    #[test_case(Repr::const_new("abc"), Repr::const_new("efg"); "short clone from inline")]
    #[test_case(Repr::new("i am a longer string that will be on the heap").unwrap(), Repr::const_new("small"); "long clone from inline")]
    #[test_case(Repr::const_new(""), Repr::new(EIGHTEEN_MB_STR).unwrap(); "empty clone from heap")]
    #[test_case(Repr::const_new("abc"), Repr::new(EIGHTEEN_MB_STR).unwrap(); "short clone from heap")]
    #[test_case(Repr::new("i am a longer string that will be on the heap").unwrap(), Repr::new(EIGHTEEN_MB_STR).unwrap(); "long clone from heap")]
    fn test_clone_from(mut initial: Repr, source: Repr) {
        initial.clone_from(&source);
        assert_eq!(initial.as_str(), source.as_str());
        assert_eq!(initial.is_heap_allocated(), source.is_heap_allocated());
    }

    #[quickcheck]
    #[cfg_attr(miri, ignore)]
    fn quickcheck_clone(initial: String) {
        let r_a = Repr::new(&initial).unwrap();
        let r_b = r_a.clone();

        assert_eq!(r_a.as_str(), initial);
        assert_eq!(r_a.len(), initial.len());

        assert_eq!(r_a.as_str(), r_b.as_str());
        assert_eq!(r_a.len(), r_b.len());
        assert_eq!(r_a.capacity(), r_b.capacity());
        assert_eq!(r_a.is_heap_allocated(), r_b.is_heap_allocated());
    }
