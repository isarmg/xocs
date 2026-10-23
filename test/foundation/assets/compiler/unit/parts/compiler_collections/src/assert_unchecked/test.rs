
    mod pass {
        #[test]
        fn plain() {
            unsafe { assert_unchecked!(0 == 0) };
        }

        #[test]
        fn string_literal() {
            unsafe { assert_unchecked!(0 == 0, "String literal") };
        }

        #[test]
        fn fmt_string() {
            unsafe { assert_unchecked!(0 == 0, "Format str {} {:?}", 123, 456) };
        }
    }

    // Cannot test failing assertions in release mode as it'd be undefined behavior!
    #[cfg(debug_assertions)]
    mod fail {
        #[test]
        #[should_panic(expected = "assertion failed: 0 == 1")]
        fn plain() {
            unsafe { assert_unchecked!(0 == 1) };
        }

        #[test]
        #[should_panic(expected = "String literal")]
        fn string_literal() {
            unsafe { assert_unchecked!(0 == 1, "String literal") };
        }

        #[test]
        #[should_panic(expected = "Format str: 123 456")]
        fn fmt_string() {
            unsafe { assert_unchecked!(0 == 1, "Format str: {} {:?}", 123, 456) };
        }
    }
