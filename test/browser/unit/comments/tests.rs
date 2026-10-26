use super::*;
use serde_json::json;

fn submission() -> Value {
    json!({
        "request_id": "12345678-1234-4123-8123-123456789abc",
        "content": "hello",
        "parent_comment_id": null,
    })
}

#[test]
fn pending_identity_and_unicode_trim() {
    let value = submission();
    assert!(PendingSubmission::parse(&value.to_string(), None).is_ok());
    let mut invalid = value;
    invalid["request_id"] = json!("12345678-1234-7123-8123-123456789abc");
    assert!(PendingSubmission::parse(&invalid.to_string(), None).is_err());
    assert_eq!("\u{85} text \u{85}".trim(), "text");
}

#[test]
fn pending_submission_is_bound_to_its_original_page() {
    let shared = submission();
    assert!(PendingSubmission::parse(&shared.to_string(), Some(1)).is_err());
    let mut article = shared.clone();
    article["article_id"] = json!(1);
    assert!(PendingSubmission::parse(&article.to_string(), None).is_err());
    assert!(PendingSubmission::parse(&article.to_string(), Some(2)).is_err());
    assert_eq!(
        PendingSubmission::parse(&article.to_string(), Some(1))
            .unwrap()
            .json(),
        article
    );
    assert_eq!(
        PendingSubmission::parse(&shared.to_string(), None)
            .unwrap()
            .json(),
        shared
    );
}

#[test]
fn persisted_submission_requires_current_object_fields() {
    let mut missing_parent = submission();
    missing_parent
        .as_object_mut()
        .unwrap()
        .remove("parent_comment_id");
    let mut unknown = submission();
    unknown["unexpected"] = json!(true);
    let mut null_article = submission();
    null_article["article_id"] = Value::Null;
    for invalid in [
        missing_parent,
        unknown,
        null_article,
        json!(["12345678-1234-4123-8123-123456789abc", "hello", null]),
        Value::Null,
    ] {
        let error = PendingSubmission::parse(&invalid.to_string(), None)
            .err()
            .unwrap();
        assert!(matches!(error, PendingError::Json(_)), "{invalid}");
        assert!(std::error::Error::source(&error).is_some());
    }
    for raw in [
        r#"{"request_id":"12345678-1234-4123-8123-123456789abc","content":"hello","content":"changed","parent_comment_id":null}"#,
        r#"{"request_id":"12345678-1234-4123-8123-123456789abc","content":"hello","parent_comment_id":null} {}"#,
    ] {
        assert!(matches!(
            PendingSubmission::parse(raw, None),
            Err(PendingError::Json(_))
        ));
    }
}

#[test]
fn persisted_submission_preserves_receipt_and_unicode_boundaries() {
    let mut valid = submission();
    valid["content"] = json!("界".repeat(1024));
    valid["parent_comment_id"] = json!(9_007_199_254_740_991i64);
    let parsed = PendingSubmission::parse(&valid.to_string(), None).unwrap();
    assert_eq!(parsed.json(), valid);
    assert_eq!(
        serde_json::from_str::<Value>(&parsed.stored_json()).unwrap(),
        valid
    );
    for (field, value) in [
        ("content", json!("界".repeat(1025))),
        ("content", json!("\u{85} \n")),
        ("parent_comment_id", json!(0)),
        ("parent_comment_id", json!(-1)),
        ("parent_comment_id", json!(9_007_199_254_740_992i64)),
        ("request_id", json!("12345678-1234-4123-7123-123456789abc")),
    ] {
        let mut invalid = valid.clone();
        invalid[field] = value;
        assert!(
            PendingSubmission::parse(&invalid.to_string(), None).is_err(),
            "{invalid}"
        );
    }
}
