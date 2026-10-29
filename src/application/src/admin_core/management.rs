use super::*;

/// The authorization snapshot must be revalidated by the store in the same
/// transaction as the mutation, after any password hashing work has finished.
#[derive(Clone, Debug)]
pub struct AdministratorManagementContext {
    pub identity: AuthenticatedIdentity,
    pub request_id: Option<RequestId>,
    pub now_micros: u64,
}

impl AdministratorManagementContext {
    pub fn validate(&self) -> Result<(), Error> {
        require_persistable_time(self.now_micros)?;
        Ok(())
    }
}

/// Hashes and internal records never implement Serialize for a public response.
pub enum AdministratorMutation {
    UpdateOwnAccount {
        username: String,
        expected_password_hash: String,
        password_hash: Option<String>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ManagementError<StoreError: std::error::Error + Send + Sync + 'static> {
    #[error("administrator authorization is no longer valid")]
    Unauthorized,
    #[error("administrator already exists or cannot be changed in this state")]
    Conflict,
    #[error("administrator input is invalid")]
    InvalidInput,
    #[error("current administrator password is incorrect")]
    InvalidCurrentPassword,
    #[error(transparent)]
    PasswordWork(#[from] PasswordWorkError),
    #[error(transparent)]
    Policy(#[from] Error),
    #[error(transparent)]
    Authentication(#[from] crate::admin_auth::Error),
    #[error("administrator storage failed: {0}")]
    Store(#[source] StoreError),
}

impl<Store: AdministratorStore + 'static> AdministratorService<Store> {
    /// Verify the current credential before atomically updating the session's own
    /// account. The store repeats session/CSRF checks under its mutation lock.
    pub async fn update_own_account(
        &self,
        context: &AdministratorManagementContext,
        username: &str,
        current_password: &str,
        new_password: Option<&str>,
    ) -> Result<(), ManagementError<Store::StoreError>> {
        context.validate()?;
        let username = crate::admin_auth::normalize_administrator_username(username)?;
        if current_password.len() > 1024 {
            return Err(ManagementError::InvalidInput);
        }
        let record = self
            .store
            .administrator_by_username(&context.identity.username)
            .await
            .map_err(ManagementError::Store)?
            .ok_or(ManagementError::Unauthorized)?;
        if !record.active || record.administrator_id != context.identity.administrator_id {
            return Err(ManagementError::Unauthorized);
        }
        let permit = tokio::time::timeout(
            Duration::from_micros(ARGON2_ACQUIRE_TIMEOUT_MICROS),
            self.argon2_slots.clone().acquire_owned(),
        )
        .await
        .map_err(PasswordWorkError::Capacity)?
        .expect("invariant: password verification semaphore remains open");
        let encoded = record.password_hash.clone();
        let candidate = current_password.to_owned();
        let matches = tokio::task::spawn_blocking(move || {
            let matches = crate::admin_auth::verify_password(&candidate, &encoded);
            drop(permit);
            matches
        })
        .await
        .map_err(PasswordWorkError::Worker)?;
        if !matches {
            return Err(ManagementError::InvalidCurrentPassword);
        }
        let password_hash = match new_password {
            Some(value) => Some(self.hash_password(value).await?),
            None => None,
        };
        self.store
            .manage_administrator(
                context,
                AdministratorMutation::UpdateOwnAccount {
                    username,
                    expected_password_hash: record.password_hash,
                    password_hash,
                },
            )
            .await
    }
}
