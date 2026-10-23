
    use super::*;

    #[test]
    #[ignore]
    fn standard_alsw() {
        StandardAlsw::test_check();
        StandardAlsw::test_decode();
    }

    #[test]
    #[ignore]
    fn url_safe_alsw() {
        UrlSafeAlsw::test_check();
        UrlSafeAlsw::test_decode();
    }
