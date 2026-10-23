
    use super::*;
    #[cfg(feature = "std")]
    use std::string::ToString;

    #[test]
    fn numeric_application_error_preserves_code() {
        let code = NonZeroU32::new(13).unwrap();
        let error = Error::from(code);
        assert_eq!(error.code(), Some(code));
        assert_eq!(error.raw_os_error(), Some(13));
    }

    #[cfg(feature = "getrandom")]
    #[test]
    fn entropy_failure_retains_its_error_instead_of_a_legacy_numeric_code() {
        let source = getrandom::Error::UNEXPECTED;
        let error = Error::from(source);
        assert_eq!(error.code(), None);
        assert_eq!(error.raw_os_error(), source.raw_os_error());
        #[cfg(feature = "std")]
        {
            assert_eq!(error.to_string(), source.to_string());
            assert!(std::error::Error::source(&error).is_some());
        }
    }
