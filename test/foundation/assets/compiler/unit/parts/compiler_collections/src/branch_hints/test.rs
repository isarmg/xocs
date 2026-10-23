
    use super::{likely, unlikely};

    #[test]
    fn likely_returns_input_unchanged() {
        assert!(likely(true));
        assert!(!likely(false));
    }

    #[test]
    fn unlikely_returns_input_unchanged() {
        assert!(unlikely(true));
        assert!(!unlikely(false));
    }

    #[test]
    fn usable_in_const_context() {
        const { assert!(likely(true)) };
        const { assert!(!unlikely(false)) };
    }
