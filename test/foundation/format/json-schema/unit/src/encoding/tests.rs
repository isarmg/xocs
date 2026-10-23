
    use super::*;

    #[test]
    fn test_encode_ref_name() {
        assert_eq!(encode_ref_name("Simple!"), "Simple!");
        assert_eq!(
            encode_ref_name("Needs %-encoding 🚀"),
            "Needs%20%25-encoding%20%F0%9F%9A%80"
        );
        assert_eq!(
            encode_ref_name("aA0-._!$&'()*+,;=:@?"),
            "aA0-._!$&'()*+,;=:@?",
        );
        assert_eq!(encode_ref_name("\"£%^\\~/"), "%22%C2%A3%25%5E%5C~0~1",);
    }

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("Simple!"), Some("Simple!".into()));
        assert_eq!(
            percent_decode("Needs %-encoding 🚀"),
            Some("Needs %-encoding 🚀".into())
        );
        assert_eq!(
            percent_decode("Needs%20%25-encoding%20%F0%9F%9A%80"),
            Some("Needs %-encoding 🚀".into())
        );
        assert_eq!(
            percent_decode("aA0-._!$&'()*+,;=:@?"),
            Some("aA0-._!$&'()*+,;=:@?".into())
        );
        assert_eq!(percent_decode("\"£%^\\~/"), Some("\"£%^\\~/".into()));
        assert_eq!(
            percent_decode("%22%C2%A3%25%5E%5C~0~1"),
            Some("\"£%^\\~0~1".into())
        );
        assert_eq!(percent_decode("%%%2020%%%"), Some("%% 20%%%".into()));
        assert_eq!(percent_decode("%f0%9F%9a%80"), Some("🚀".into()));
        assert_eq!(percent_decode("%F0%9F%9A"), None);
    }
