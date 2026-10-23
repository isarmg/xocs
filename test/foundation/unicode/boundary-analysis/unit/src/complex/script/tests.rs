
    use super::*;

    #[test]
    fn test_thai_only() {
        let s = "ภาษาไทยภาษาไทย";
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let mut iter = ComplexScriptIteratorUtf16::new(&utf16);
        assert_eq!(
            iter.next(),
            Some((utf16.as_slice(), ComplexScript::Thai)),
            "Thai script only with UTF-16"
        );
        let mut iter = ComplexScriptIterator::new(s);
        assert_eq!(
            iter.next(),
            Some((s, ComplexScript::Thai)),
            "Thai script only with UTF-8"
        );
        assert_eq!(iter.next(), None, "Iterator for UTF-8 is finished");
    }

    #[test]
    fn test_myanmar_only() {
        let s = "မြန်မာစာမြန်မာစာမြန်မာစာ";
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let mut iter = ComplexScriptIteratorUtf16::new(&utf16);
        assert_eq!(
            iter.next(),
            Some((utf16.as_slice(), ComplexScript::Myanmar)),
            "Myanmar script only with UTF-16"
        );
        let mut iter = ComplexScriptIterator::new(s);
        assert_eq!(
            iter.next(),
            Some((s, ComplexScript::Myanmar)),
            "Myanmar script only with UTF-8"
        );
        assert_eq!(iter.next(), None, "Iterator for UTF-8 is finished");
    }

    #[test]
    fn test_combine() {
        const TEST_STR_THAI: &str = "ภาษาไทยภาษาไทย";
        const TEST_STR_MYANMAR: &str = "ဗမာနွယ်ဘာသာစကားမျာ";
        let s = format!("{TEST_STR_THAI}{TEST_STR_MYANMAR}");
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let thai_utf16: Vec<u16> = TEST_STR_THAI.encode_utf16().collect();
        let myanmar_utf16: Vec<u16> = TEST_STR_MYANMAR.encode_utf16().collect();

        let mut iter = ComplexScriptIteratorUtf16::new(&utf16);
        assert_eq!(
            iter.next(),
            Some((thai_utf16.as_slice(), ComplexScript::Thai)),
            "Thai script with UTF-16 at first"
        );
        assert_eq!(
            iter.next(),
            Some((myanmar_utf16.as_slice(), ComplexScript::Myanmar)),
            "Myanmar script with UTF-16 at second"
        );
        assert_eq!(iter.next(), None, "Iterator for UTF-16 is finished");

        let mut iter = ComplexScriptIterator::new(&s);
        assert_eq!(
            iter.next(),
            Some((TEST_STR_THAI, ComplexScript::Thai)),
            "Thai script with UTF-8 at first"
        );
        assert_eq!(
            iter.next(),
            Some((TEST_STR_MYANMAR, ComplexScript::Myanmar)),
            "Myanmar script with UTF-8 at second"
        );
        assert_eq!(iter.next(), None, "Iterator for UTF-8 is finished");
    }
