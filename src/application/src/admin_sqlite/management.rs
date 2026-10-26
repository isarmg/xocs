use super::*;
use crate::admin_core::{
    AdministratorManagementContext, AdministratorMutation, AuditOutcome, ManagementError,
};
use std::time::{SystemTime, UNIX_EPOCH};

type Failure = ManagementError<Error>;

fn storage_error(error: sqlx::Error) -> Failure {
    if error
        .as_database_error()
        .is_some_and(|error| error.is_unique_violation())
    {
        ManagementError::Conflict
    } else {
        ManagementError::Store(Error::Sqlx(error))
    }
}

pub(super) async fn execute(
    store: &SqliteAdministratorStore,
    context: &AdministratorManagementContext,
    mutation: AdministratorMutation,
) -> Result<(), Failure> {
    context.validate()?;
    // Serialize authorization, account checks, data and audit writes.
    // Read the clock AFTER waiting for the writer lock and password hashing.
    let mut transaction = store
        .pool
        .begin_with("BEGIN IMMEDIATE")
        .await
        .map_err(storage_error)?;
    let wall_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| ManagementError::Store(Error::Clock(source)))?
        .as_micros();
    let now = u64::try_from(wall_now)
        .map_err(|_| ManagementError::Store(Error::IntegerRange))?
        .max(context.now_micros);
    let timestamp = to_i64(now).map_err(ManagementError::Store)?;
    let authorized = session_authorized_on_connection(&mut transaction, &context.identity, now)
        .await
        .map_err(ManagementError::Store)?;
    if !authorized {
        return Err(ManagementError::Unauthorized);
    }

    let (subject, action, revokes_sessions) = match mutation {
        AdministratorMutation::UpdateOwnAccount {
            username,
            expected_password_hash,
            password_hash,
        } => {
            crate::admin_auth::require_canonical_administrator_username(&username)?;
            if let Some(hash) = &password_hash {
                crate::admin_auth::require_current_password_hash(hash)?;
            }
            let id = context.identity.administrator_id();
            let changed = sqlx::query("UPDATE _common_administrators SET username=?, password_hash=COALESCE(?, password_hash), session_version=session_version+1, updated_at_micros=? WHERE administrator_id=? AND active=1 AND password_hash=?")
                .bind(&username).bind(password_hash).bind(timestamp).bind(id.as_str()).bind(expected_password_hash)
                .execute(&mut *transaction).await.map_err(storage_error)?.rows_affected();
            if changed != 1 {
                return Err(ManagementError::Unauthorized);
            }
            revoke_administrator_sessions(&mut transaction, id, now)
                .await
                .map_err(ManagementError::Store)?;
            (username, SecurityAction::AdministratorAccountUpdated, true)
        }
    };
    let primary_event = event(context, &subject, action, now)?;
    insert_audit(&mut transaction, &primary_event)
        .await
        .map_err(ManagementError::Store)?;
    if revokes_sessions {
        let event = event(
            context,
            &subject,
            SecurityAction::AdministratorSessionsRevoked,
            now,
        )?;
        insert_audit(&mut transaction, &event)
            .await
            .map_err(ManagementError::Store)?;
    }
    transaction.commit().await.map_err(storage_error)
}

fn event(
    context: &AdministratorManagementContext,
    subject: &str,
    action: SecurityAction,
    now: u64,
) -> Result<SecurityAuditEvent, Failure> {
    Ok(SecurityAuditEvent {
        event_id: Identifier::generate()
            .map_err(|error| ManagementError::Store(Error::Authentication(error)))?,
        action,
        outcome: AuditOutcome::Success,
        actor_administrator_id: Some(context.identity.administrator_id().clone()),
        subject_digest: Some(crate::admin_auth::token_hash(subject)),
        request_id: context.request_id.clone(),
        detail_json: "{}".into(),
        occurred_at_micros: now,
    })
}

#[path = "../../../../test/application/unit/admin_sqlite/management/tests.rs"]
#[cfg(test)]
mod tests;
