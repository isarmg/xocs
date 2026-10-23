
    use super::StringCharCodeAt;

    #[test]
    fn test_evaluate_char_code_at() {
        let s = "abcde";
        assert_eq!(s.char_code_at(Some(0.0)), Some(97));
        assert_eq!(s.char_code_at(Some(1.0)), Some(98));
        assert_eq!(s.char_code_at(Some(2.0)), Some(99));
        assert_eq!(s.char_code_at(Some(3.0)), Some(100));
        assert_eq!(s.char_code_at(Some(4.0)), Some(101));
        assert_eq!(s.char_code_at(Some(5.0)), None);
        assert_eq!(s.char_code_at(Some(-1.0)), None);
        assert_eq!(s.char_code_at(None), Some(97));
        assert_eq!(s.char_code_at(Some(0.0)), Some(97));
        assert_eq!(s.char_code_at(Some(f64::NAN)), Some(97));
        assert_eq!(s.char_code_at(Some(f64::INFINITY)), None);
    }
