use serde::{Deserialize, Serialize};

pub const LOGIN_PATH: &str = "/api/v1/auth/login";
pub const SESSION_PATH: &str = "/api/v1/auth/session";
pub const LOGOUT_PATH: &str = "/api/v1/auth/logout";
pub const ACCOUNT_PATH: &str = "/api/v1/platform/administrators/self";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionFields {
    authenticated: bool,
    user_id: String,
    username: String,
    role: String,
    csrf_token: String,
}

/// A response parsed as an authenticated administrator with a valid CSRF token.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SessionFields", into = "SessionFields")]
pub struct AdministratorSession {
    user_id: String,
    username: String,
    csrf_token: String,
}

impl std::fmt::Debug for AdministratorSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdministratorSession")
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
enum SessionError {
    #[error("expected an authenticated administrator")]
    Authentication,
    #[error("invalid administrator identifier")]
    UserId,
    #[error("invalid administrator username")]
    Username,
    #[error("invalid administrator CSRF token")]
    CsrfToken,
}

impl TryFrom<SessionFields> for AdministratorSession {
    type Error = SessionError;

    fn try_from(fields: SessionFields) -> Result<Self, Self::Error> {
        let SessionFields {
            authenticated,
            user_id,
            username,
            role,
            csrf_token,
        } = fields;
        if !authenticated || role != "admin" {
            return Err(SessionError::Authentication);
        }
        if !safe_request_id(&user_id) {
            return Err(SessionError::UserId);
        }
        if !(3..=64).contains(&username.len())
            || !username
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
            || !username.as_bytes()[0].is_ascii_alphanumeric()
            || !username.as_bytes()[username.len() - 1].is_ascii_alphanumeric()
        {
            return Err(SessionError::Username);
        }
        if csrf_token.len() != 43
            || !csrf_token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            || !b"AEIMQUYcgkosw048".contains(&csrf_token.as_bytes()[42])
        {
            return Err(SessionError::CsrfToken);
        }
        Ok(Self {
            user_id,
            username,
            csrf_token,
        })
    }
}

impl From<AdministratorSession> for SessionFields {
    fn from(session: AdministratorSession) -> Self {
        let AdministratorSession {
            user_id,
            username,
            csrf_token,
        } = session;
        Self {
            authenticated: true,
            user_id,
            username,
            role: "admin".into(),
            csrf_token,
        }
    }
}

impl AdministratorSession {
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn csrf_token(&self) -> &str {
        &self.csrf_token
    }
}

pub fn valid_username_candidate(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.bytes().all(|b| (0x20..=0x7e).contains(&b))
}

pub fn valid_password(value: &str) -> bool {
    (12..=1024).contains(&value.len()) && !value.chars().any(|c| c <= '\u{1f}' || c == '\u{7f}')
}

pub fn valid_instance_name(value: &str, maximum: usize) -> bool {
    (1..=32).contains(&maximum)
        && !value.trim().is_empty()
        && value.trim().chars().count() <= maximum
        && !value
            .chars()
            .any(|c| c <= '\u{1f}' || ('\u{7f}'..='\u{9f}').contains(&c))
}

pub fn safe_request_id(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
}

#[path = "../../../test/interface/unit/contracts/tests.rs"]
#[cfg(test)]
mod tests;
