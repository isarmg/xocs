use serde::Deserialize;
use serde_json::{Value, json};

use super::*;

#[test]
fn administrator_debug_does_not_log_password_or_csrf_credentials() {
    let login = AdministratorLoginRequest {
        username: "admin".into(),
        password: "DO_NOT_LOG_PASSWORD".into(),
    };
    assert!(!format!("{login:?}").contains("DO_NOT_LOG_PASSWORD"));
    let token = "A".repeat(43);
    let session = AdministratorSession::new("admin-1", "admin", &token).unwrap();
    assert!(!format!("{session:?}").contains(&token));
    assert!(format!("{session:?}").contains("[REDACTED]"));
}

const ERROR_FIXTURES: &str = include_str!("../../fixtures/contracts/error-envelope.fixtures.json");
const ADMIN_AUTH_FIXTURES: &str =
    include_str!("../../fixtures/contracts/administrator-auth.fixtures.json");
const STATE_FIXTURES: &str = include_str!("../../fixtures/contracts/state-contract.fixtures.json");
const RELEASE_FIXTURES: &str = include_str!("../../fixtures/contracts/release.fixtures.json");
const BACKUP_FIXTURES: &str =
    include_str!("../../fixtures/contracts/backup-manifest.fixtures.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSet {
    valid: Vec<Value>,
    invalid: Vec<Value>,
}

#[test]
fn error_envelope_matches_every_shared_fixture() {
    assert_fixtures(ERROR_FIXTURES, error_envelope_from_slice);
}

#[test]
fn administrator_auth_matches_every_shared_fixture() {
    let fixtures: FixtureSet =
        serde_json::from_str(ADMIN_AUTH_FIXTURES).expect("fixture JSON is valid");
    for (index, value) in fixtures.valid.into_iter().enumerate() {
        let accepted = serde_json::from_value::<AdministratorLoginRequest>(value.clone()).is_ok()
            || serde_json::from_value::<AdministratorSession>(value).is_ok();
        assert!(accepted, "rejected valid administrator fixture {index}");
    }
    for (index, value) in fixtures.invalid.into_iter().enumerate() {
        let accepted = serde_json::from_value::<AdministratorLoginRequest>(value.clone()).is_ok()
            || serde_json::from_value::<AdministratorSession>(value).is_ok();
        assert!(!accepted, "accepted invalid administrator fixture {index}");
    }
}

#[test]
fn administrator_session_is_validated_at_construction_and_preserves_its_wire_format() {
    let session = AdministratorSession::new(
        "018f1f4b-7a5d-7b5f-8d31-123456789abc",
        "admin",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&session).unwrap()["role"],
        json!(ADMINISTRATOR_ROLE)
    );

    assert_eq!(session.username(), "admin");
    assert_eq!(
        serde_json::from_value::<AdministratorSession>(serde_json::to_value(&session).unwrap())
            .unwrap(),
        session,
    );
    for (field, invalid) in [
        ("authenticated", json!(false)),
        ("user_id", json!("invalid id")),
        ("username", json!("Admin")),
        ("role", json!("viewer")),
        ("role", json!({"admin":null})),
        ("csrf_token", json!("noncanonical-token-shape")),
        ("unknown", json!(true)),
    ] {
        let mut wire = serde_json::to_value(&session).unwrap();
        wire[field] = invalid;
        assert!(serde_json::from_value::<AdministratorSession>(wire).is_err());
    }
    assert!(AdministratorSession::new("invalid id", "admin", session.csrf_token()).is_err());
    assert!(AdministratorSession::new(session.user_id(), "Admin", session.csrf_token()).is_err());
    assert!(AdministratorSession::new(session.user_id(), "admin", "invalid-token").is_err());
}

#[test]
fn administrator_login_cannot_serialize_an_invalid_public_value() {
    let mut login = AdministratorLoginRequest {
        username: " Admin ".to_owned(),
        password: "bounded wire candidate".to_owned(),
    };
    assert_eq!(
        serde_json::to_value(&login).unwrap(),
        json!({
            "username": " Admin ",
            "password": "bounded wire candidate"
        })
    );

    login.username = "管理员".to_owned();
    assert!(serde_json::to_value(&login).is_err());

    login.username = "admin".to_owned();
    login.password.clear();
    assert!(serde_json::to_value(&login).is_err());
}

#[test]
fn state_contract_matches_every_shared_fixture() {
    assert_fixtures(STATE_FIXTURES, StateContract::from_slice);
}

#[test]
fn release_identity_matches_every_shared_fixture() {
    assert_fixtures(RELEASE_FIXTURES, ReleaseIdentity::from_slice);
}

#[test]
fn backup_manifest_matches_every_shared_fixture() {
    assert_fixtures(BACKUP_FIXTURES, BackupManifest::from_slice);
}

fn assert_fixtures<T, E>(fixtures: &str, parse: impl Fn(&[u8]) -> Result<T, E>)
where
    E: std::fmt::Display,
{
    let fixtures: FixtureSet = serde_json::from_str(fixtures).expect("fixture JSON is valid");
    for (index, value) in fixtures.valid.into_iter().enumerate() {
        let bytes = serde_json::to_vec(&value).expect("fixture value serializes");
        if let Err(error) = parse(&bytes) {
            panic!("rejected valid shared fixture {index}: {error}");
        }
    }
    for (index, value) in fixtures.invalid.into_iter().enumerate() {
        let bytes = serde_json::to_vec(&value).expect("fixture value serializes");
        assert!(
            parse(&bytes).is_err(),
            "accepted invalid shared fixture {index}"
        );
    }
}

#[test]
fn required_nullable_fields_cannot_be_omitted() {
    let state = json!({
        "contract_version": 1,
        "application": "sample-product",
        "application_version": "0.3.0",
        "source_revision": "a".repeat(40),
        "maintenance_locks": [],
        "resources": [],
        "external_requirements": [],
        "companion_contracts": []
    });
    assert!(StateContract::from_slice(&serde_json::to_vec(&state).unwrap()).is_err());

    let backup = json!({
        "manifest_version": 1,
        "tool_version": "0.3.0",
        "product": "sample-product",
        "application_version": "0.3.0",
        "created_at_epoch_seconds": 0,
        "external_requirements": [],
        "resources": [{
            "name": "database",
            "kind": "sqlite",
            "path": "database.sqlite3",
            "bytes": 0,
            "files": 1,
            "sha256": "a".repeat(64)
        }]
    });
    assert!(BackupManifest::from_slice(&serde_json::to_vec(&backup).unwrap()).is_err());
}

#[test]
fn semantic_boundaries_match_the_json_schemas() {
    let release = ReleaseIdentity {
        product: "sample-product".to_owned(),
        version: "0.3.0".to_owned(),
        source_revision: "a".repeat(40),
        target: "x86_64-unknown-linux-gnu".to_owned(),
        state_contract_sha256: "b".repeat(64),
    };
    release.validate().unwrap();

    let mut invalid_revision = release;
    invalid_revision.source_revision = "A".repeat(40);
    assert!(matches!(
        invalid_revision.validate(),
        Err(ValidationError::InvalidSourceRevision { .. })
    ));

    let schema = StateSchemaIdentity {
        revision: MAX_SAFE_JSON_INTEGER,
        sha256: "c".repeat(64),
    };
    schema.validate().unwrap();
    let too_large = StateSchemaIdentity {
        revision: MAX_SAFE_JSON_INTEGER + 1,
        sha256: "c".repeat(64),
    };
    assert!(matches!(
        too_large.validate(),
        Err(ValidationError::UnsafeInteger { .. })
    ));
}

#[test]
fn mathematically_integral_json_numbers_match_javascript_guards() {
    let state = format!(
        r#"{{
                "contract_version": 1.0,
                "application": "sample-product",
                "application_version": "0.3.0",
                "source_revision": "{}",
                "schema": {{"revision": 3e0, "sha256": "{}"}},
                "maintenance_locks": [],
                "resources": [],
                "external_requirements": [],
                "companion_contracts": []
            }}"#,
        "a".repeat(40),
        "b".repeat(64)
    );
    StateContract::from_slice(state.as_bytes()).unwrap();

    let backup = format!(
        r#"{{
                "manifest_version": 1.0,
                "tool_version": "0.3.0",
                "product": "sample-product",
                "application_version": "0.3.0",
                "schema_identity": {{
                    "application": "sample-product",
                    "application_version": "0.3.0",
                    "schema_revision": 4.0,
                    "schema_sha256": "{}"
                }},
                "created_at_epoch_seconds": 0e0,
                "external_requirements": [],
                "resources": [{{
                    "name": "database",
                    "kind": "sqlite",
                    "path": "database.sqlite3",
                    "bytes": 0.0,
                    "files": 1e0,
                    "sha256": "{}"
                }}]
            }}"#,
        "a".repeat(64),
        "b".repeat(64)
    );
    BackupManifest::from_slice(backup.as_bytes()).unwrap();

    let fractional = state.replace("\"revision\": 3e0", "\"revision\": 3.5");
    assert!(StateContract::from_slice(fractional.as_bytes()).is_err());
}

#[test]
fn current_contract_does_not_invent_product_owned_array_or_path_rules() {
    let manifest = json!({
        "manifest_version": 1,
        "tool_version": "0.3.0",
        "product": "sample-product",
        "application_version": "0.3.0",
        "schema_identity": null,
        "created_at_epoch_seconds": 0,
        "external_requirements": [],
        "resources": [
            {
                "name": "z-resource",
                "kind": "data-tree",
                "path": "../product-owned-path",
                "bytes": 0,
                "files": 1,
                "sha256": "a".repeat(64)
            },
            {
                "name": "z-resource",
                "kind": "data-tree",
                "path": "../product-owned-path",
                "bytes": 0,
                "files": 1,
                "sha256": "a".repeat(64)
            }
        ]
    });
    BackupManifest::from_slice(&serde_json::to_vec(&manifest).unwrap()).unwrap();
}

#[test]
fn only_the_current_versions_are_accepted() {
    let state = serde_json::from_str::<FixtureSet>(STATE_FIXTURES)
        .unwrap()
        .valid
        .remove(0);
    for version in [0, 2] {
        let mut value = state.clone();
        value["contract_version"] = json!(version);
        assert!(StateContract::from_slice(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    let release = serde_json::from_str::<FixtureSet>(RELEASE_FIXTURES)
        .unwrap()
        .valid
        .remove(0);
    let mut unknown = release;
    unknown["upgrade_tool"] = json!("unexpected");
    assert!(ReleaseIdentity::from_slice(&serde_json::to_vec(&unknown).unwrap()).is_err());
}
