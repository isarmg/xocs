
    use super::{is_ascii_punctuation, is_punctuation};

    #[test]
    fn test_ascii() {
        assert!(is_ascii_punctuation(b'!'));
        assert!(is_ascii_punctuation(b'@'));
        assert!(is_ascii_punctuation(b'~'));
        assert!(!is_ascii_punctuation(b' '));
        assert!(!is_ascii_punctuation(b'0'));
        assert!(!is_ascii_punctuation(b'A'));
        assert!(!is_ascii_punctuation(0xA1));
    }

    #[test]
    fn test_unicode() {
        assert!(is_punctuation('~'));
        assert!(!is_punctuation(' '));

        assert!(is_punctuation('\u{00A1}'));
        assert!(is_punctuation('\u{060C}'));
        assert!(is_punctuation('\u{FF65}'));
        assert!(is_punctuation('\u{1BC9F}'));
        assert!(!is_punctuation('\u{1BCA0}'));
    }
