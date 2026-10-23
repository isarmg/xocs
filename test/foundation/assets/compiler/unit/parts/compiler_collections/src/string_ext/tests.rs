
    use super::StringExt;

    #[test]
    fn push_unchecked_ascii() {
        let mut s = String::with_capacity(8);
        // SAFETY: capacity 8, pushing single bytes well within capacity
        unsafe {
            s.push_unchecked('a');
            s.push_unchecked('$');
        }
        assert_eq!(s, "a$");
    }

    #[test]
    fn push_unchecked_multibyte() {
        let mut s = String::with_capacity(8);
        // SAFETY: capacity 8; 'é' is 2 bytes, '가' is 3 bytes -> 5 bytes total <= 8
        unsafe {
            s.push_unchecked('é');
            s.push_unchecked('가');
        }
        assert_eq!(s, "é가");
    }

    #[test]
    fn push_str_unchecked_works() {
        let mut s = String::with_capacity(16);
        // SAFETY: capacity 16, "foo" + "bar" = 6 bytes <= 16
        unsafe {
            s.push_str_unchecked("foo");
            s.push_str_unchecked("bar");
        }
        assert_eq!(s, "foobar");
    }

    #[test]
    fn push_up_to_exact_capacity() {
        let mut s = String::with_capacity(3);
        // SAFETY: capacity 3; filling it exactly (2 + 1 bytes)
        unsafe {
            s.push_str_unchecked("ab");
            s.push_unchecked('c');
        }
        assert_eq!(s, "abc");
    }
