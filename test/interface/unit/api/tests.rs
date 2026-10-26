use super::*;
#[test]
fn restore_does_not_touch_the_browser_during_revocation() {
    for phase in [AuthPhase::Anonymous, AuthPhase::LogoutUnconfirmed] {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.revoking = Some(state.generation);
            state.phase = phase;
        });
        let error = futures::executor::block_on(restore()).unwrap_err();
        assert_eq!(
            error.code,
            ApiErrorCode::Local(LocalErrorCode::AuthLogoutUnconfirmed)
        );
        STATE.with(|state| {
            let state = state.borrow();
            assert!(state.revoking.is_some());
            assert!(state.phase == phase);
            assert!(state.restore.is_none());
        });
    }
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.revoking = None;
        state.phase = AuthPhase::Loading;
    });
}

fn revocation_state(latest_intent: u64, latest_logout: u64) -> State {
    State {
        session: None,
        transport: Some(
            serde_json::from_value(json!({
                "authenticated": true,
                "user_id": "admin-id",
                "username": "admin",
                "role": "admin",
                "csrf_token": "A".repeat(43),
            }))
            .expect("invariant: the revocation fixture is a valid administrator session"),
        ),
        generation: latest_intent,
        phase: AuthPhase::Anonymous,
        error: None,
        account_updated: false,
        revoking: Some(latest_logout),
        restore: None,
    }
}

#[test]
fn queued_login_does_not_discard_logout_failure() {
    // Logout owns generation 1; login intent 2 is waiting for its lock.
    let mut state = revocation_state(2, 1);
    let error = ApiError::local(LocalErrorCode::NetworkError);
    assert!(settle_revocation(&mut state, 1, Some(&error)));
    // A failed login returns without publishing any state.
    assert_eq!(state.generation, 2);
    assert_eq!(state.phase, AuthPhase::LogoutUnconfirmed);
    assert_eq!(state.revoking, Some(1));
    assert_eq!(state.error, Some(error));
    assert!(state.transport.is_some());
}

#[test]
fn queued_login_does_not_discard_confirmed_logout() {
    let mut state = revocation_state(2, 1);
    assert!(settle_revocation(&mut state, 1, None));
    assert_eq!(state.generation, 2);
    assert_eq!(state.phase, AuthPhase::Anonymous);
    assert_eq!(state.revoking, None);
    assert!(state.transport.is_none());
    assert!(state.error.is_none());
}

#[test]
fn older_logout_cannot_settle_a_newer_logout_barrier() {
    // Logout 1, login 2, and logout 3 have all recorded their intents.
    for old_error in [None, Some(ApiError::local(LocalErrorCode::NetworkError))] {
        let mut state = revocation_state(3, 3);
        assert!(!settle_revocation(&mut state, 1, old_error.as_ref()));
        assert_eq!(state.generation, 3);
        assert_eq!(state.revoking, Some(3));
        assert_eq!(state.phase, AuthPhase::Anonymous);
        let error = ApiError::local(LocalErrorCode::RequestTimeout);
        assert!(settle_revocation(&mut state, 3, Some(&error)));
        assert_eq!(state.phase, AuthPhase::LogoutUnconfirmed);
        assert_eq!(state.revoking, Some(3));
        assert!(settle_revocation(&mut state, 3, None));
        assert_eq!(state.revoking, None);
    }
}

#[test]
fn typed_decode_preserves_large_integers_and_null_contracts() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Payload {
        sequence: i64,
        optional: Option<String>,
    }
    let value = json!({"sequence": 9_007_199_254_740_993i64, "optional": null});
    let decoded: Payload = decode_response(value).unwrap();
    assert_eq!(decoded.sequence, 9_007_199_254_740_993);
    assert_eq!(decoded.optional, None);
    assert_eq!(
        decode_response::<Payload>(Value::Null)
            .unwrap_err()
            .code
            .as_str(),
        "invalid_typed_response_shape"
    );
    assert_eq!(decode_response::<()>(Value::Null).unwrap(), ());
    assert!(decode_validated_response::<Payload>(json!({"sequence": 1}), |_| false).is_err());
    assert!(decode_response::<Payload>(json!({"sequence":"1"})).is_err());
}
#[test]
fn typed_body_serialization_errors_are_explicit() {
    struct Invalid;
    impl Serialize for Invalid {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("intentional failure"))
        }
    }
    assert_eq!(
        encode_json_body(&Invalid).unwrap_err().code.as_str(),
        "body_serialization_failed"
    );
    assert_eq!(
        encode_json_body(&json!({"revision":"a"})).unwrap(),
        json!({"revision":"a"})
    );
}
#[test]
fn api_boundary_rejects_cross_origin_and_fragment_paths() {
    for path in [
        "https://evil.example/api/v1/x",
        "//evil.example/api/v1/x",
        "/api/v1/x#secret",
        "/api/v1/x\n",
        "/api/v1/\\evil",
    ] {
        assert!(!validate_api_path(path), "{path:?}");
    }
    assert!(validate_api_path("/api/v1/items?cursor=a%2Fb"));
}

#[test]
fn error_envelope_parses_identifiers_once_and_keeps_unknown_remote_codes() {
    let envelope = decode_error_envelope(json!({
        "code": "content.remote_failure",
        "message": "untrusted diagnostic",
        "retryable": false,
        "request_id": "req-1:abc",
        "details": {"revision": 4}
    }))
    .unwrap();
    assert!(matches!(
        envelope.code,
        ApiErrorCode::Remote(RemoteErrorCode::Unknown(_))
    ));
    assert_eq!(envelope.code.as_str(), "content.remote_failure");
    assert_eq!(envelope.request_id.unwrap().as_str(), "req-1:abc");

    let envelope = decode_error_envelope(json!({
        "code": "admin.conflict", "message": "conflict", "retryable": false
    }))
    .unwrap();
    assert_eq!(
        envelope.code,
        ApiErrorCode::Remote(RemoteErrorCode::AdministratorConflict)
    );
    assert!(envelope.request_id.is_none());
}

#[test]
fn remote_error_codes_cannot_claim_local_message_policy() {
    let envelope = decode_error_envelope(json!({
        "code": "display_error", "message": "private server diagnostic", "retryable": false
    }))
    .unwrap();
    assert_ne!(
        envelope.code,
        ApiErrorCode::Local(LocalErrorCode::DisplayError)
    );
    assert_eq!(
        serde_json::to_value(&envelope.code).unwrap(),
        "display_error"
    );
}

#[test]
fn malformed_error_envelopes_preserve_the_parse_source() {
    let valid = json!({"code": "content.failure", "message": "failed", "retryable": false});
    for (field, value) in [
        ("code", json!("")),
        ("code", json!("not a code")),
        ("code", json!("A".repeat(129))),
        ("message", Value::Null),
        ("retryable", json!(0)),
        ("request_id", json!("unsafe\nidentifier")),
        ("request_id", Value::Null),
        ("details", json!([])),
        ("details", Value::Null),
        ("unexpected", json!(true)),
    ] {
        let mut body = valid.clone();
        body[field] = value;
        let error = decode_error_envelope(body)
            .err()
            .expect("invalid wire envelope");
        assert_eq!(
            error.code,
            ApiErrorCode::Local(LocalErrorCode::InvalidErrorResponse)
        );
        let source = std::error::Error::source(&error).expect("JSON source retained");
        assert!(source.source().is_some());
        assert!(!error.to_string().contains("unsafe"));
        assert!(serde_json::to_value(error).unwrap().get("source").is_none());
    }
}
