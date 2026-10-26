//! Axum extraction failures at the shared HTTP error boundary. Product Serde
//! definitions retain ownership of their current field and semantic contract.

use crate::error::{ErrorEnvelope, RequestId};
use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Query, Request, rejection::JsonRejection},
    http::{StatusCode, header, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;
use std::time::Duration;

/// Total time allowed to receive one product JSON body, independent of progress.
pub const JSON_BODY_TIMEOUT: Duration = Duration::from_secs(10);

/// Apply the same bounded correlation and error policy as xcss's native
/// transport. Install this outside product input/authentication middleware.
pub async fn request_context_middleware(request: Request, next: Next) -> Response {
    use tower::ServiceExt;
    match crate::server_runtime::request_service(next)
        .oneshot(request)
        .await
    {
        Ok(response) => response,
        Err(never) => match never {},
    }
}

fn json_data_reason(error: &dyn std::error::Error) -> &'static str {
    let mut source = error.source();
    while let Some(error) = source {
        let json_error = error.downcast_ref::<serde_json::Error>().or_else(|| {
            error
                .downcast_ref::<serde::error_path::Error<serde_json::Error>>()
                .map(|error| error.inner())
        });
        if let Some(error) = json_error {
            // Inspect only the serializer's stable category prefixes. Neither
            // the error text nor submitted field names/values leave this scope.
            let message = error.to_string();
            return if message.starts_with("missing field ") {
                "MISSING_FIELD"
            } else if message.starts_with("unknown field ") {
                "UNKNOWN_FIELD"
            } else if message.starts_with("invalid type: ")
                && !message.contains(", expected struct ")
            {
                "TYPE_MISMATCH"
            } else {
                "JSON_STRUCTURE"
            };
        }
        source = error.source();
    }
    "JSON_STRUCTURE"
}

#[derive(Debug, Clone, Copy)]
pub struct ContractJson<T>(pub T);
#[derive(Debug, Clone, Copy)]
pub struct ContractQuery<T>(pub T);
#[derive(Debug, Clone, Copy)]
pub struct ContractPath<T>(pub T);

fn rejected(status: StatusCode, reason: &'static str, request_id: Option<&RequestId>) -> Response {
    let (code, message) = if status == StatusCode::REQUEST_TIMEOUT {
        (
            "request_timeout",
            "The request body was not received in time.",
        )
    } else if status == StatusCode::PAYLOAD_TOO_LARGE {
        (
            "payload_too_large",
            "The request body exceeds its size limit.",
        )
    } else if status.is_server_error() {
        (
            "internal_error",
            "The request endpoint could not process its input.",
        )
    } else {
        (
            "contract_violation",
            "The request does not satisfy the current input contract.",
        )
    };
    let mut envelope = super::failure(code, message);
    envelope.retryable = status == StatusCode::REQUEST_TIMEOUT;
    envelope.details.insert("reason".into(), reason.into());
    envelope.request_id = request_id.cloned();
    let mut response = error_response(status, envelope);
    if status == StatusCode::REQUEST_TIMEOUT {
        // The unfinished HTTP/1 request body must not keep the connection open.
        response
            .headers_mut()
            .insert(header::CONNECTION, "close".parse().expect("static header"));
    }
    response
}

fn error_response(status: StatusCode, error: ErrorEnvelope) -> Response {
    let mut response = (status, Json(error)).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        "no-store".parse().expect("static header"),
    );
    response
}

impl<T, S> FromRequest<S> for ContractJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let request_id = request.extensions().get::<RequestId>().cloned();
        // Bound only extraction. Once a handler starts, its transaction is not
        // interrupted by this deadline, avoiding uncertain write outcomes.
        tokio::time::timeout(JSON_BODY_TIMEOUT, Json::<T>::from_request(request, state))
            .await
            .map_err(|_| {
                rejected(
                    StatusCode::REQUEST_TIMEOUT,
                    "BODY_TIMEOUT",
                    request_id.as_ref(),
                )
            })?
            .map(|Json(value)| Self(value))
            .map_err(|error| {
                #[expect(
                    clippy::wildcard_enum_match_arm,
                    reason = "JsonRejection is an external non-exhaustive rejection type"
                )]
                let (status, reason) = match &error {
                    JsonRejection::JsonDataError(error) => {
                        (StatusCode::BAD_REQUEST, json_data_reason(error))
                    }
                    JsonRejection::JsonSyntaxError(_) => (StatusCode::BAD_REQUEST, "INVALID_JSON"),
                    JsonRejection::MissingJsonContentType(_) => {
                        (StatusCode::UNSUPPORTED_MEDIA_TYPE, "CONTENT_TYPE_REQUIRED")
                    }
                    _ if error.status() == StatusCode::PAYLOAD_TOO_LARGE => {
                        (StatusCode::PAYLOAD_TOO_LARGE, "BODY_LIMIT")
                    }
                    _ => (StatusCode::BAD_REQUEST, "INVALID_JSON"),
                };
                rejected(status, reason, request_id.as_ref())
            })
    }
}

impl<T, S> FromRequestParts<S> for ContractQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(value)| Self(value))
            .map_err(|_| {
                rejected(
                    StatusCode::BAD_REQUEST,
                    "INVALID_QUERY",
                    parts.extensions.get::<RequestId>(),
                )
            })
    }
}

impl<T, S> FromRequestParts<S> for ContractPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Path::<T>::from_request_parts(parts, state)
            .await
            .map(|Path(value)| Self(value))
            .map_err(|error| {
                rejected(
                    error.status(),
                    "INVALID_PATH",
                    parts.extensions.get::<RequestId>(),
                )
            })
    }
}

#[path = "../../../../test/application/unit/server_cli/http/tests.rs"]
#[cfg(test)]
mod tests;
