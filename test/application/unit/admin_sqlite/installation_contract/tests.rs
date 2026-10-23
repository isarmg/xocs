use super::*;
#[test]
fn credential_export_is_deterministic_and_matches_live_validators() {
    let first = installation_contract().unwrap();
    assert_eq!(first, installation_contract().unwrap());
    let vectors = &first["credential_validation_vectors"];
    for vector in vectors["username"].as_array().unwrap() {
        let input = vector["input"].as_str().unwrap();
        assert_eq!(
            admin_auth::normalize_administrator_username(input)
                .ok()
                .as_deref(),
            vector["canonical"].as_str()
        );
        assert_eq!(
            admin_auth::require_canonical_administrator_username(input).is_ok(),
            vector["stored_valid"].as_bool().unwrap()
        );
    }
    for vector in vectors["password"].as_array().unwrap() {
        assert_eq!(
            admin_auth::validate_password(vector["input"].as_str().unwrap()).is_ok(),
            vector["valid"].as_bool().unwrap()
        );
    }
    let vector = &vectors["password_hash"];
    let encoded = vector["encoded"].as_str().unwrap();
    assert!(admin_auth::verify_password(
        vector["password"].as_str().unwrap(),
        encoded
    ));
    assert!(!admin_auth::verify_password(
        vector["incorrect_password"].as_str().unwrap(),
        encoded
    ));
    assert_eq!(
        first["operations"]["reset_password"]["sql"],
        ADMINISTRATOR_PASSWORD_RESET_SQL
    );
    assert_eq!(first["ddl"], ADMIN_PERSISTENT_DDL);
    assert_eq!(
        first["credential_policy"]["password_hash"]["version"],
        u32::from(argon2::Version::V0x13)
    );
    assert_eq!(
        first["credential_policy"]["password_hash"]["algorithm"],
        argon2::Algorithm::Argon2id.to_string()
    );
}
