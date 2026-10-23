#[test]
fn test_contains() {
    macro_rules! test_contains {
        (true, $haystack: expr, $needle: expr) => {
            assert!(Contains($haystack, $needle).const_eval());
        };
        (false, $haystack: expr, $needle: expr) => {
            assert!(!Contains($haystack, $needle).const_eval());
        };
    }

    test_contains!(true, "asd", "");
    test_contains!(true, "asd", "a");
    test_contains!(true, "asdf", "sd");
    test_contains!(false, "", "a");
    test_contains!(false, "asd", "abcd");

    test_contains!(true, "唐可可", '可');
    test_contains!(true, "Liyuu", 'i');
    test_contains!(false, "Liyuu", '我');
}
