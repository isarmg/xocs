use super::*;
use crate::admin_core::{AdministratorStore, AuditOutcome};

fn event(id: &str, action: SecurityAction, administrator_id: &Identifier) -> SecurityAuditEvent {
    SecurityAuditEvent {
        event_id: Identifier::new(id).unwrap(),
        action,
        outcome: AuditOutcome::Success,
        actor_administrator_id: Some(administrator_id.clone()),
        subject_digest: None,
        request_id: Some(
            crate::error::RequestId::new("request-1").expect("fixture request ID is valid"),
        ),
        detail_json: "{}".into(),
        occurred_at_micros: 10,
    }
}

fn session(administrator_id: &Identifier, id: &str, byte: u8) -> SessionRecord {
    SessionRecord {
        session_id: Identifier::new(id).unwrap(),
        administrator_id: administrator_id.clone(),
        token_hash: [byte; 32],
        csrf_hash: [byte.wrapping_add(1); 32],
        administrator_session_version: 1,
        created_at_micros: 10,
        last_seen_at_micros: 10,
        idle_expires_at_micros: 100,
        absolute_expires_at_micros: 200,
        revoked_at_micros: None,
    }
}

#[tokio::test]
async fn transaction_authorization_uses_current_revocation_expiry_and_account_state()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::admin_core::{AdministratorService, LoginContext, SESSION_IDLE_MICROS};
    let store = crate::testkit::administrator_fixture("admin", "correct horse battery").await;
    let service = AdministratorService::with_isolated_test_budget(store);
    let login = service
        .login(
            "admin",
            "correct horse battery",
            &LoginContext {
                request_id: None,
                now_micros: 10,
            },
        )
        .await?;
    let identity = service
        .authenticate_session(&login.session_token, 10)
        .await?;
    let store = service.store();
    let mut connection = store.pool().acquire().await?;
    assert!(session_authorized_on_connection(&mut connection, &identity, 10).await?);
    let idle_expiry = 10 + SESSION_IDLE_MICROS;
    assert!(session_authorized_on_connection(&mut connection, &identity, idle_expiry - 1).await?);
    assert!(!session_authorized_on_connection(&mut connection, &identity, idle_expiry).await?);
    assert!(matches!(
        session_authorized_on_connection(&mut connection, &identity, u64::MAX).await,
        Err(Error::IntegerRange)
    ));
    drop(connection);

    for (mutation, now) in [
        ("UPDATE _common_admin_sessions SET revoked_at_micros=11", 11),
        (
            "UPDATE _common_admin_sessions SET csrf_hash=zeroblob(32)",
            11,
        ),
        (
            "UPDATE _common_admin_sessions SET absolute_expires_at_micros=100, idle_expires_at_micros=100",
            100,
        ),
        ("UPDATE _common_administrators SET active=0", 11),
        (
            "UPDATE _common_administrators SET session_version=session_version+1",
            11,
        ),
    ] {
        let mut transaction = store.pool().begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query(mutation).execute(&mut *transaction).await?;
        assert!(!session_authorized_on_connection(&mut transaction, &identity, now).await?);
        transaction.rollback().await?;
    }
    let mut transaction = store.pool().begin_with("BEGIN IMMEDIATE").await?;
    assert!(session_authorized_on_connection(&mut transaction, &identity, 11).await?);
    transaction.rollback().await?;
    store.pool().close().await;
    Ok(())
}

#[tokio::test]
async fn mutations_and_audit_are_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let pool = crate::sqlite::create_if_missing(
        directory.path().join("admin.sqlite3"),
        crate::sqlite::PoolOptions::new(1),
    )
    .await?;
    sqlx::raw_sql(ADMIN_PERSISTENT_DDL).execute(&pool).await?;
    let store = SqliteAdministratorStore::new(pool.clone());
    let administrator_id = Identifier::new("administrator-1")?;
    store
        .create_administrator(
            AdministratorRecord {
                administrator_id: administrator_id.clone(),
                username: "admin".into(),
                password_hash: crate::admin_auth::hash_password("correct horse battery")?,
                active: true,
                session_version: 1,
                created_at_micros: 1,
                updated_at_micros: 1,
                last_login_at_micros: None,
            },
            event(
                "event-created",
                SecurityAction::AdministratorCreated,
                &administrator_id,
            ),
        )
        .await?;

    let rolled_back = session(&administrator_id, "session-rollback", 3);
    assert!(
        store
            .commit_login_success(LoginSuccess {
                session: rolled_back.clone(),
                session_created_event: event(
                    "event-created",
                    SecurityAction::SessionCreated,
                    &administrator_id
                ),
                login_succeeded_event: event(
                    "event-never",
                    SecurityAction::LoginSucceeded,
                    &administrator_id
                ),
            })
            .await
            .is_err()
    );
    assert!(
        store
            .session_by_token_hash(rolled_back.token_hash)
            .await?
            .is_none()
    );

    let active = session(&administrator_id, "session-active", 7);
    store
        .commit_login_success(LoginSuccess {
            session: active.clone(),
            session_created_event: event(
                "event-session",
                SecurityAction::SessionCreated,
                &administrator_id,
            ),
            login_succeeded_event: event(
                "event-login",
                SecurityAction::LoginSucceeded,
                &administrator_id,
            ),
        })
        .await?;
    assert_eq!(
        store
            .session_by_token_hash(active.token_hash)
            .await?
            .unwrap()
            .session,
        active
    );
    let audit_count: i64 = sqlx::query_scalar("SELECT count(*) FROM _common_security_audit_events")
        .fetch_one(&pool)
        .await?;
    assert_eq!(audit_count, 3);
    Ok(())
}

#[tokio::test]
async fn out_of_order_login_commits_preserve_latest_activity_and_both_sessions_and_audits()
-> Result<(), Box<dyn std::error::Error>> {
    let store = crate::testkit::administrator_fixture("admin", "correct horse battery").await;
    let administrator = store.administrator_by_username("admin").await?.unwrap();
    assert_eq!(administrator.last_login_at_micros, None);
    let mut committed = Vec::new();
    for (time, name, byte) in [(20, "newer", 21), (10, "older", 11)] {
        let mut record = session(&administrator.administrator_id, name, byte);
        record.created_at_micros = time;
        record.last_seen_at_micros = time;
        let mut created = event(
            &format!("{name}-created"),
            SecurityAction::SessionCreated,
            &administrator.administrator_id,
        );
        created.occurred_at_micros = time;
        let mut succeeded = event(
            &format!("{name}-succeeded"),
            SecurityAction::LoginSucceeded,
            &administrator.administrator_id,
        );
        succeeded.occurred_at_micros = time;
        store
            .commit_login_success(LoginSuccess {
                session: record.clone(),
                session_created_event: created,
                login_succeeded_event: succeeded,
            })
            .await?;
        committed.push(record);
        let current = store.administrator_by_username("admin").await?.unwrap();
        assert_eq!(current.last_login_at_micros, Some(20));
        assert_eq!(current.updated_at_micros, 20);
    }
    for record in committed {
        assert_eq!(
            store
                .session_by_token_hash(record.token_hash)
                .await?
                .unwrap()
                .session,
            record
        );
    }
    let audit: Vec<(String, i64)> = sqlx::query_as(
        "SELECT action,occurred_at_micros FROM _common_security_audit_events ORDER BY occurred_at_micros,action",
    ).fetch_all(store.pool()).await?;
    assert_eq!(
        audit,
        [
            ("login.succeeded".into(), 10),
            ("session.created".into(), 10),
            ("login.succeeded".into(), 20),
            ("session.created".into(), 20),
        ]
    );
    store.pool().close().await;
    Ok(())
}
