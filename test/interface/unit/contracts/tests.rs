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

#[test]
fn unknown_session_fields_and_nonadmin_roles_are_rejected() {
    let mut value = serde_json::json!({"authenticated":true,"user_id":"123e4567-e89b-12d3-a456-426614174000","username":"admin","role":"admin","csrf_token":"A".repeat(43)});
    assert!(
        serde_json::from_value::<AdministratorSession>(value.clone())
            .unwrap()
            .validate()
    );
    value["role"] = serde_json::json!("viewer");
    assert!(
        !serde_json::from_value::<AdministratorSession>(value.clone())
            .unwrap()
            .validate()
    );
    value["extra"] = serde_json::json!(true);
    assert!(serde_json::from_value::<AdministratorSession>(value).is_err());
}
