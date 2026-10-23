use super::*;
use crate::admin_core::{AdministratorStore, AuditOutcome};

fn event(id: &str, action: SecurityAction, administrator_id: &Identifier) -> SecurityAuditEvent {
    SecurityAuditEvent {
        event_id: Identifier::new(id).unwrap(),
        action,
        outcome: AuditOutcome::Success,
        actor_administrator_id: Some(administrator_id.clone()),
        subject_digest: None,
        request_id: Some("request-1".into()),
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
