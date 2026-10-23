
    use super::*;

    #[test]
    fn inlinestr_ascii() {
        let s: InlineStr = 'a'.into();
        assert_eq!("a", s.deref());
    }

    #[test]
    fn inlinestr_unicode() {
        let s: InlineStr = '🍔'.into();
        assert_eq!("🍔", s.deref());
    }

    #[test]
    fn cowstr_size() {
        let size = std::mem::size_of::<CowStr>();
        let word_size = std::mem::size_of::<isize>();
        assert_eq!(3 * word_size, size);
    }

    #[test]
    fn cowstr_char_to_string() {
        let c = '藏';
        let smort: CowStr = c.into();
        let owned: String = smort.to_string();
        let expected = "藏".to_owned();
        assert_eq!(expected, owned);
    }

    #[test]
    fn max_inline_str_len_atleast_four() {
        // we need 4 bytes to store a char
        assert!(MAX_INLINE_STR_LEN >= 4);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn inlinestr_fits_twentytwo() {
        let s = "0123456789abcdefghijkl";
        let stack_str = InlineStr::try_from(s).unwrap();
        assert_eq!(stack_str.deref(), s);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn inlinestr_not_fits_twentythree() {
        let s = "0123456789abcdefghijklm";
        let _stack_str = InlineStr::try_from(s).unwrap_err();
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn small_boxed_str_clones_to_stack() {
        let s = "0123456789abcde".to_owned();
        let smort: CowStr = s.into();
        let smort_clone = smort.clone();

        if let CowStr::Inlined(..) = smort_clone {
        } else {
            panic!("Expected a Inlined variant!");
        }
    }

    #[test]
    fn cow_to_cow_str() {
        let s = "some text";
        let cow = Cow::Borrowed(s);
        let actual = CowStr::from(cow);
        let expected = CowStr::Borrowed(s);
        assert_eq!(actual, expected);
        assert!(variant_eq(&actual, &expected));

        let s = "some text".to_string();
        let cow: Cow<str> = Cow::Owned(s.clone());
        let actual = CowStr::from(cow);
        let expected = CowStr::Boxed(s.into_boxed_str());
        assert_eq!(actual, expected);
        assert!(variant_eq(&actual, &expected));
    }

    #[test]
    fn cow_str_to_cow() {
        let s = "some text";
        let cow_str = CowStr::Borrowed(s);
        let actual = Cow::from(cow_str);
        let expected = Cow::Borrowed(s);
        assert_eq!(actual, expected);
        assert!(variant_eq(&actual, &expected));

        let s = "s";
        let inline_str: InlineStr = InlineStr::try_from(s).unwrap();
        let cow_str = CowStr::Inlined(inline_str);
        let actual = Cow::from(cow_str);
        let expected: Cow<str> = Cow::Owned(s.to_string());
        assert_eq!(actual, expected);
        assert!(variant_eq(&actual, &expected));

        let s = "s";
        let cow_str = CowStr::Boxed(s.to_string().into_boxed_str());
        let actual = Cow::from(cow_str);
        let expected: Cow<str> = Cow::Owned(s.to_string());
        assert_eq!(actual, expected);
        assert!(variant_eq(&actual, &expected));
    }

    #[test]
    fn cow_str_to_string() {
        let s = "some text";
        let cow_str = CowStr::Borrowed(s);
        let actual = String::from(cow_str);
        let expected = String::from("some text");
        assert_eq!(actual, expected);

        let s = "s";
        let inline_str: InlineStr = InlineStr::try_from(s).unwrap();
        let cow_str = CowStr::Inlined(inline_str);
        let actual = String::from(cow_str);
        let expected = String::from("s");
        assert_eq!(actual, expected);

        let s = "s";
        let cow_str = CowStr::Boxed(s.to_string().into_boxed_str());
        let actual = String::from(cow_str);
        let expected = String::from("s");
        assert_eq!(actual, expected);
    }

    #[test]
    fn cow_char_to_cow_str() {
        let c = 'c';
        let cow: Cow<char> = Cow::Owned(c);
        let actual = CowStr::from(cow);
        let expected = CowStr::Inlined(InlineStr::from(c));
        assert_eq!(actual, expected);
        assert!(variant_eq(&actual, &expected));

        let c = 'c';
        let cow: Cow<char> = Cow::Borrowed(&c);
        let actual = CowStr::from(cow);
        let expected = CowStr::Inlined(InlineStr::from(c));
        assert_eq!(actual, expected);
        assert!(variant_eq(&actual, &expected));
    }

    fn variant_eq<T>(a: &T, b: &T) -> bool {
        std::mem::discriminant(a) == std::mem::discriminant(b)
    }
