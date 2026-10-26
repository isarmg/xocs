//! Administrator initialization shared by explicit offline provisioning and tests.
use blog_application::admin_core::{
    AdministratorRecord, AdministratorService, AdministratorStore, AuditOutcome, Identifier,
    SecurityAction, SecurityAuditEvent,
};

pub async fn seed_administrator(
    service: &AdministratorService<blog_application::admin_sqlite::SqliteAdministratorStore>,
    username: &str,
    password: &blog_application::admin_auth::Password,
    now: u64,
) -> Result<(), super::Error> {
    let account = AdministratorRecord {
        administrator_id: Identifier::generate()?,
        username: username.to_owned(),
        password_hash: password.hash()?,
        active: true,
        session_version: 1,
        created_at_micros: now,
        updated_at_micros: now,
        last_login_at_micros: None,
    };
    service
        .store()
        .create_administrator(
            account,
            SecurityAuditEvent {
                event_id: Identifier::generate()?,
                action: SecurityAction::AdministratorCreated,
                outcome: AuditOutcome::Success,
                actor_administrator_id: None,
                subject_digest: Some(blog_application::admin_auth::token_hash(username)),
                request_id: None,
                detail_json: "{}".into(),
                occurred_at_micros: now,
            },
        )
        .await?;
    Ok(())
}
