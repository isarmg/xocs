//! Administrator initialization shared by explicit offline provisioning and tests.
use blog_application::admin_core::{
    AdministratorRecord, AdministratorService, AdministratorStore, AuditOutcome, Identifier,
    SecurityAction, SecurityAuditEvent,
};

pub async fn seed_administrator<S: AdministratorStore>(
    service: &AdministratorService<S>,
    username: &str,
    password: &str,
    now: u64,
) -> anyhow::Result<bool> {
    let username = blog_application::admin_auth::normalize_administrator_username(username)?;
    let account = AdministratorRecord {
        administrator_id: Identifier::new(blog_application::admin_auth::random_token()?)?,
        username: username.clone(),
        password_hash: blog_application::admin_auth::hash_password(password)?,
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
                event_id: Identifier::new(blog_application::admin_auth::random_token()?)?,
                action: SecurityAction::AdministratorCreated,
                outcome: AuditOutcome::Success,
                actor_administrator_id: None,
                subject_digest: Some(blog_application::admin_auth::token_hash(&username)),
                request_id: None,
                detail_json: "{}".into(),
                occurred_at_micros: now,
            },
        )
        .await?;
    Ok(true)
}
