//! Local failures and validated, untrusted identifiers from HTTP responses.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalErrorCode {
    AuthLogoutUnconfirmed,
    AuthOperationSuperseded,
    BodySerializationFailed,
    BrowserRequired,
    CsrfHeaderOwnedBySession,
    DisplayError,
    InvalidAccount,
    InvalidApiPath,
    InvalidContentType,
    InvalidCredentials,
    InvalidErrorResponse,
    InvalidHeaders,
    InvalidJsonResponse,
    InvalidRequestBudget,
    InvalidResponse,
    InvalidResponseContract,
    InvalidResponseShape,
    InvalidTypedResponseShape,
    NetworkError,
    RequestAborted,
    RequestTimeout,
    ResponseTooLarge,
}

impl LocalErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AuthLogoutUnconfirmed => "auth_logout_unconfirmed",
            Self::AuthOperationSuperseded => "auth_operation_superseded",
            Self::BodySerializationFailed => "body_serialization_failed",
            Self::BrowserRequired => "browser_required",
            Self::CsrfHeaderOwnedBySession => "csrf_header_owned_by_session",
            Self::DisplayError => "display_error",
            Self::InvalidAccount => "invalid_account",
            Self::InvalidApiPath => "invalid_api_path",
            Self::InvalidContentType => "invalid_content_type",
            Self::InvalidCredentials => "invalid_credentials",
            Self::InvalidErrorResponse => "invalid_error_response",
            Self::InvalidHeaders => "invalid_headers",
            Self::InvalidJsonResponse => "invalid_json_response",
            Self::InvalidRequestBudget => "invalid_request_budget",
            Self::InvalidResponse => "invalid_response",
            Self::InvalidResponseContract => "invalid_response_contract",
            Self::InvalidResponseShape => "invalid_response_shape",
            Self::InvalidTypedResponseShape => "invalid_typed_response_shape",
            Self::NetworkError => "network_error",
            Self::RequestAborted => "request_aborted",
            Self::RequestTimeout => "request_timeout",
            Self::ResponseTooLarge => "response_too_large",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemoteErrorCode {
    SessionRequired,
    CurrentPasswordInvalid,
    AdministratorConflict,
    Unknown(RemoteCode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteCode(String);

impl RemoteErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SessionRequired => "auth.session_required",
            Self::CurrentPasswordInvalid => "admin.current_password_invalid",
            Self::AdministratorConflict => "admin.conflict",
            Self::Unknown(value) => &value.0,
        }
    }
}

impl TryFrom<String> for RemoteErrorCode {
    type Error = InvalidRemoteCode;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !(1..=128).contains(&value.len())
            || !value.as_bytes()[0].is_ascii_lowercase()
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
            })
        {
            return Err(InvalidRemoteCode);
        }
        Ok(match value.as_str() {
            "auth.session_required" => Self::SessionRequired,
            "admin.current_password_invalid" => Self::CurrentPasswordInvalid,
            "admin.conflict" => Self::AdministratorConflict,
            _ => Self::Unknown(RemoteCode(value)),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidRemoteCode;

impl fmt::Display for InvalidRemoteCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid remote error code")
    }
}
impl std::error::Error for InvalidRemoteCode {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiErrorCode {
    Local(LocalErrorCode),
    Remote(RemoteErrorCode),
}

impl ApiErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Local(code) => code.as_str(),
            Self::Remote(code) => code.as_str(),
        }
    }
}

impl From<LocalErrorCode> for ApiErrorCode {
    fn from(code: LocalErrorCode) -> Self {
        Self::Local(code)
    }
}

impl Serialize for ApiErrorCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ApiErrorCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // A wire value cannot claim the local UI's trusted display-message policy.
        RemoteErrorCode::try_from(String::deserialize(deserializer)?)
            .map(Self::Remote)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RequestId(String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for RequestId {
    type Error = InvalidRequestId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !crate::contracts::safe_request_id(&value) {
            return Err(InvalidRequestId);
        }
        Ok(Self(value))
    }
}

impl From<RequestId> for String {
    fn from(value: RequestId) -> Self {
        value.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidRequestId;

impl fmt::Display for InvalidRequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid request identifier")
    }
}
impl std::error::Error for InvalidRequestId {}
