
    use std::error::Error as _;

    use super::*;

    fn bad_json_error() -> serde_json::Error {
        serde_json::from_str::<u32>("not a number").unwrap_err()
    }

    #[test]
    fn display_messages() {
        assert_eq!(
            Error::VlqLeftover.to_string(),
            "VLQ string was malformed and data was left over"
        );
        assert_eq!(
            Error::VlqNoValues.to_string(),
            "VLQ string was empty and no values could be decoded"
        );
        assert_eq!(
            Error::VlqOverflow.to_string(),
            "The input encoded a number that didn't fit into i64"
        );
        assert!(Error::BadJson(bad_json_error()).to_string().starts_with("JSON parsing error:"));
        assert_eq!(
            Error::BadSegmentSize(7).to_string(),
            "Mapping segment had an unsupported size of 7"
        );
        assert_eq!(
            Error::BadSourceReference(3).to_string(),
            "Reference to non-existing source at position 3"
        );
        assert_eq!(
            Error::BadNameReference(9).to_string(),
            "Reference to non-existing name at position 9"
        );
    }

    #[test]
    fn error_source() {
        // Only `BadJson` wraps an underlying error.
        assert!(Error::BadJson(bad_json_error()).source().is_some());
        assert!(Error::VlqLeftover.source().is_none());
        assert!(Error::BadSegmentSize(1).source().is_none());
    }

    #[test]
    fn from_serde_json_error() {
        let err: Error = bad_json_error().into();
        assert!(matches!(err, Error::BadJson(_)));
    }
