use super::*;

pub const ADMINISTRATORS_PATH: &str = "/api/v1/platform/administrators";

pub const ADMIN_ACCOUNT_PATH: &str = "/api/v1/platform/administrators/self";

/// Self-service always verifies the current password; omit new_password to keep it.
/// No account identifier is accepted from the caller.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdministratorAccountRequest {
    #[serde(deserialize_with = "deserialize_administrator_username_candidate")]
    pub username: String,
    #[serde(deserialize_with = "deserialize_secret_text")]
    pub current_password: String,
    #[serde(default, deserialize_with = "deserialize_new_password")]
    pub new_password: Option<String>,
}

fn deserialize_new_password<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    deserialize_secret_text(deserializer).map(Some)
}

#[path = "../../../../test/application/unit/contracts/administrators/tests.rs"]
#[cfg(test)]
mod tests;
