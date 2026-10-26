use super::*;

#[test]
fn credentials_keep_utf8_byte_policy_and_unicode_names() {
    assert!(valid_password("密码密码密码"));
    assert!(!valid_password("short"));
    assert!(!valid_password("abcdefghijkl\n"));
    assert!(valid_instance_name(&"😀".repeat(32), 32));
    assert!(!valid_instance_name(&"😀".repeat(33), 32));
    assert!(!valid_instance_name("name\u{85}", 32));
}

fn session_payload() -> serde_json::Value {
    serde_json::json!({
        "authenticated": true,
        "user_id": "123e4567-e89b-12d3-a456-426614174000",
        "username": "admin",
        "role": "admin",
        "csrf_token": "A".repeat(43),
    })
}

#[test]
fn session_is_validated_during_deserialization_and_keeps_its_wire_format() {
    let value = session_payload();
    let session: AdministratorSession = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(session.user_id(), value["user_id"]);
    assert_eq!(session.username(), "admin");
    assert_eq!(serde_json::to_value(&session).unwrap(), value);
    assert!(!format!("{session:?}").contains(&"A".repeat(43)));
}

#[test]
fn session_rejects_invalid_authentication_identifiers_names_tokens_and_unknown_fields() {
    let baseline = session_payload();
    for (field, invalid) in [
        ("authenticated", serde_json::json!(false)),
        ("role", serde_json::json!("viewer")),
        ("user_id", serde_json::json!("")),
        ("user_id", serde_json::json!("id\n")),
        ("username", serde_json::json!("")),
        ("username", serde_json::json!("ab")),
        ("username", serde_json::json!("A".repeat(3))),
        ("username", serde_json::json!("a".repeat(65))),
        ("username", serde_json::json!("-admin")),
        ("username", serde_json::json!("admin-")),
        ("csrf_token", serde_json::json!("A".repeat(42))),
        ("csrf_token", serde_json::json!("A".repeat(44))),
        (
            "csrf_token",
            serde_json::json!(format!("{}B", "A".repeat(42))),
        ),
        (
            "csrf_token",
            serde_json::json!(format!("{}=", "A".repeat(42))),
        ),
        ("extra", serde_json::json!(true)),
    ] {
        let mut value = baseline.clone();
        value[field] = invalid;
        assert!(
            serde_json::from_value::<AdministratorSession>(value).is_err(),
            "{field}"
        );
    }
    for name in ["a-b", "a_b", "a.b", &"a".repeat(64)] {
        let mut value = baseline.clone();
        value["username"] = serde_json::json!(name);
        assert!(serde_json::from_value::<AdministratorSession>(value).is_ok());
    }
}
