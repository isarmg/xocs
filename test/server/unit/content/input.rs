use super::*;
use axum::http::HeaderValue;
use std::error::Error as _;

#[test]
fn idempotency_keys_preserve_valid_headers_and_reject_malformed_values() {
    let mut headers = HeaderMap::new();
    assert!(IdempotencyKey::from_headers(&headers).unwrap().is_none());
    for key in ["operation-1".to_owned(), "x".repeat(128)] {
        headers.insert("Idempotency-Key", HeaderValue::from_str(&key).unwrap());
        let parsed = IdempotencyKey::from_headers(&headers).unwrap().unwrap();
        assert_eq!(parsed.as_str(), key);
    }
    for key in ["".to_owned(), "has space".to_owned(), "x".repeat(129)] {
        headers.insert("Idempotency-Key", HeaderValue::from_str(&key).unwrap());
        assert!(matches!(
            IdempotencyKey::from_headers(&headers),
            Err(IdempotencyKeyError::Invalid)
        ));
    }
    headers.insert("Idempotency-Key", HeaderValue::from_bytes(&[0xff]).unwrap());
    let error = IdempotencyKey::from_headers(&headers).err().unwrap();
    assert!(matches!(error, IdempotencyKeyError::Header(_)));
    assert!(error.source().is_some());
}

#[test]
fn short_content_is_normalized_and_bounded_when_deserialized() {
    for text in [" hello ".to_owned(), "文".repeat(1024)] {
        let parsed: ShortContent = serde_json::from_value(serde_json::json!(text)).unwrap();
        assert_eq!(parsed.as_str(), text.trim());
        assert_eq!(serde_json::to_value(parsed).unwrap(), text.trim());
    }
    for text in [" \n ".to_owned(), "文".repeat(1025)] {
        assert!(serde_json::from_value::<ShortContent>(serde_json::json!(text)).is_err());
    }
}
