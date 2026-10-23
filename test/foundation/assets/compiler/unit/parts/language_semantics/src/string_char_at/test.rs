
    use crate::language_semantics::string_char_at::StringCharAtResult;

    use super::StringCharAt;

    #[test]
    fn test_evaluate_string_char_at() {
        let s = "test";
        assert_eq!(s.char_at(Some(0.0)), StringCharAtResult::Value('t'));
        assert_eq!(s.char_at(Some(1.0)), StringCharAtResult::Value('e'));
        assert_eq!(s.char_at(Some(2.0)), StringCharAtResult::Value('s'));
        assert_eq!(s.char_at(Some(4.0)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(0.5)), StringCharAtResult::Value('t'));
        assert_eq!(s.char_at(None), StringCharAtResult::Value('t'));
        assert_eq!(s.char_at(Some(f64::NAN)), StringCharAtResult::Value('t'));
        assert_eq!(s.char_at(Some(f64::INFINITY)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(f64::NEG_INFINITY)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(-1.0)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(-1.1)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(-1_073_741_825.0)), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(f64::from(u32::MAX))), StringCharAtResult::OutOfRange);
        assert_eq!(s.char_at(Some(f64::from(u32::MAX) + 1.0)), StringCharAtResult::OutOfRange);

        let astral = "𝄞";
        assert_eq!(astral.char_at(Some(0.0)), StringCharAtResult::InvalidChar(0xd834));
        assert_eq!(astral.char_at(Some(1.0)), StringCharAtResult::InvalidChar(0xdd1e));
        assert_eq!(astral.char_at(Some(2.0)), StringCharAtResult::OutOfRange);
    }
