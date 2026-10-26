use super::*;

#[test]
fn debug_never_exposes_password_hashes_or_session_secrets() {
    let record = AdministratorRecord {
        administrator_id: Identifier::new("admin-1").unwrap(),
        username: "admin".into(),
        password_hash: "DO_NOT_LOG_PASSWORD_HASH".into(),
        active: true,
        session_version: 1,
        created_at_micros: 1,
        updated_at_micros: 1,
        last_login_at_micros: None,
    };
    assert!(!format!("{record:?}").contains("DO_NOT_LOG_PASSWORD_HASH"));
    let csrf = "A".repeat(43);
    let token = format!("{}E", "B".repeat(42));
    let authenticated = AuthenticatedSession {
        administrator: crate::contracts::AdministratorSession::new("admin-1", "admin", &csrf)
            .unwrap(),
        session_token: token.clone(),
    };
    let debug = format!("{authenticated:?}");
    assert!(!debug.contains(&csrf));
    assert!(!debug.contains(&token));
    assert!(debug.contains("[REDACTED]"));
}

#[test]
fn policy_and_cookie_contract_are_fixed() {
    let policy = AdministratorPolicy;
    assert_eq!(policy.session_idle_micros(), 1_800_000_000);
    assert_eq!(policy.sessions_global(), 1_024);
    let token = "A".repeat(43);
    assert_eq!(
        session_set_cookie(
            "example-product",
            AdministratorOriginMode::ProductionHttps,
            &token
        )
        .unwrap(),
        format!(
            "__Host-admin-example-product-session={token}; Path=/; HttpOnly; Secure; SameSite=Strict"
        )
    );
    assert!(
        !session_clear_cookie(
            "example-product",
            AdministratorOriginMode::LoopbackDevelopmentHttp
        )
        .unwrap()
        .contains("Secure")
    );
}

#[tokio::test]
async fn independent_installations_share_cpu_budget_but_not_attempt_counts() {
    let first = AdministratorService::new(
        crate::testkit::administrator_fixture("admin", "correct horse battery").await,
    );
    let second = AdministratorService::new(
        crate::testkit::administrator_fixture("admin", "correct horse battery").await,
    );
    assert!(Arc::ptr_eq(&first.argon2_slots, &second.argon2_slots));
    let first_state = first.store().authentication_state().await.unwrap();
    let second_state = second.store().authentication_state().await.unwrap();
    assert!(!Arc::ptr_eq(&first_state, &second_state));
    for _ in 0..LOGIN_ATTEMPTS_MAX {
        first_state.admit_login_attempt().unwrap();
    }
    assert!(matches!(
        first_state.admit_login_attempt(),
        Err(AuthenticationStateError::LoginAttemptsExceeded { .. })
    ));
    assert_eq!(second_state.admit_login_attempt(), Ok(()));
}

#[tokio::test]
async fn fixture_budgets_are_isolated_but_keep_the_production_limit_and_timeout() {
    let first = AdministratorService::with_isolated_test_budget(
        crate::testkit::administrator_fixture("admin", "correct horse battery").await,
    );
    let second = AdministratorService::with_isolated_test_budget(
        crate::testkit::administrator_fixture("admin", "correct horse battery").await,
    );
    assert!(!Arc::ptr_eq(&first.argon2_slots, &second.argon2_slots));
    assert_eq!(first.argon2_slots.available_permits(), ARGON2_CONCURRENCY);
    assert_eq!(second.argon2_slots.available_permits(), ARGON2_CONCURRENCY);
    let permits = first
        .argon2_slots
        .clone()
        .acquire_many_owned(ARGON2_CONCURRENCY.try_into().unwrap())
        .await
        .unwrap();
    assert_eq!(first.argon2_slots.available_permits(), 0);
    assert_eq!(second.argon2_slots.available_permits(), ARGON2_CONCURRENCY);
    // Advance only after all fixture I/O. Exercise the real timed admission
    // path without making this policy assertion depend on host CPU load.
    tokio::time::pause();
    let started = tokio::time::Instant::now();
    assert!(matches!(
        first.hash_password("correct horse battery").await,
        Err(PasswordWorkError::Capacity(_))
    ));
    assert!(started.elapsed() >= Duration::from_micros(ARGON2_ACQUIRE_TIMEOUT_MICROS));
    drop(permits);
    assert_eq!(first.argon2_slots.available_permits(), ARGON2_CONCURRENCY);
}

#[test]
fn audit_input_is_bounded_before_json_parsing_and_uses_canonical_request_ids() {
    use std::error::Error as _;

    let mut event = SecurityAuditEvent {
        event_id: Identifier::new("event-1").unwrap(),
        action: SecurityAction::LoginSucceeded,
        outcome: AuditOutcome::Success,
        actor_administrator_id: None,
        subject_digest: None,
        request_id: Some(
            crate::error::RequestId::new("valid-request").expect("fixture request ID is valid"),
        ),
        detail_json: "{}".into(),
        occurred_at_micros: 1,
    };
    assert!(event.validate().is_ok());
    event.detail_json = format!(
        "{{\"padding\":\"{}\"}}",
        "x".repeat(AUDIT_DETAILS_MAX_BYTES)
    );
    assert!(event.validate().is_err());
    event.detail_json = "{}".into();
    assert!(RequestId::new("invalid\nrequest").is_err());

    event.request_id = None;
    event.detail_json = "{".into();
    let failure = event.validate().unwrap_err();
    assert!(failure.source().unwrap().is::<serde_json::Error>());

    let context = LoginContext {
        request_id: Some(RequestId::new("valid-context").unwrap()),
        now_micros: 1,
    };
    context.validate().unwrap();
}

// Separate fixtures model separate processes without sharing the process-wide budget.
impl<Store: AdministratorStore> AdministratorService<Store> {
    pub(crate) fn with_isolated_test_budget(store: Store) -> Self {
        Self {
            store,
            argon2_slots: Arc::new(Semaphore::new(ARGON2_CONCURRENCY)),
        }
    }
}
