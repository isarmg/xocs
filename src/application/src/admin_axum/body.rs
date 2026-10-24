//! Bounded readers for untrusted administrator HTTP bodies, shared by both adapters.
use super::{RequestId, error};
use crate::admin_core::{ADMIN_BODY_MAX_BYTES, ADMIN_BODY_READERS_GLOBAL, ADMIN_BODY_TIMEOUT};
use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Response,
};
use bytes::Bytes;
use std::{
    error::Error,
    sync::{Arc, Mutex, OnceLock},
};

#[derive(Default)]
struct Readers {
    total: usize,
}

#[derive(Clone)]
pub(super) struct BodyAdmission(Arc<Mutex<Readers>>);

impl Default for BodyAdmission {
    fn default() -> Self {
        static READERS: OnceLock<Arc<Mutex<Readers>>> = OnceLock::new();
        Self(READERS.get_or_init(Arc::default).clone())
    }
}

struct Permit {
    admission: BodyAdmission,
}

impl Drop for Permit {
    fn drop(&mut self) {
        // A poisoned budget remains closed; never resume with uncertain counters.
        if let Ok(mut readers) = self.admission.0.lock() {
            readers.total -= 1;
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Scope {
    Login,
    Management,
}

impl Scope {
    fn content_type(self) -> &'static str {
        match self {
            Self::Login => "auth.content_type_required",
            Self::Management => "admin.content_type_required",
        }
    }
    fn too_large(self) -> &'static str {
        match self {
            Self::Login => "auth.body_too_large",
            Self::Management => "admin.body_too_large",
        }
    }
    fn invalid(self) -> &'static str {
        match self {
            Self::Login => "auth.invalid_request",
            Self::Management => "admin.invalid_request",
        }
    }
}

impl BodyAdmission {
    #[cfg(test)]
    pub(super) fn isolated() -> Self {
        Self(Arc::default())
    }

    fn acquire(&self, id: Option<&RequestId>) -> Result<Permit, Box<Response>> {
        let mut readers = self.0.lock().map_err(|_| {
            Box::new(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "platform.internal",
                false,
                id,
            ))
        })?;
        if readers.total >= ADMIN_BODY_READERS_GLOBAL {
            let mut response = error(
                StatusCode::TOO_MANY_REQUESTS,
                "auth.body_capacity",
                true,
                id,
            );
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
            return Err(Box::new(response));
        }
        readers.total += 1;
        Ok(Permit {
            admission: self.clone(),
        })
    }

    pub(super) async fn read(
        &self,
        request: Request,
        scope: Scope,
        json: bool,
        id: Option<&RequestId>,
    ) -> Result<Bytes, Box<Response>> {
        if json {
            require_json_content_type(request.headers(), scope, id)?;
        }
        let _permit = self.acquire(id)?;
        read_bounded(request.into_body(), scope, id).await
    }
}

fn require_json_content_type(
    headers: &HeaderMap,
    scope: Scope,
    id: Option<&RequestId>,
) -> Result<(), Box<Response>> {
    let mut values = headers.get_all(header::CONTENT_TYPE).iter();
    let valid = match (values.next(), values.next()) {
        (Some(value), None) => value
            .to_str()
            .ok()
            .and_then(|value| value.parse::<mime::Mime>().ok())
            .is_some_and(|value| value.essence_str().eq_ignore_ascii_case("application/json")),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(Box::new(error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            scope.content_type(),
            false,
            id,
        )))
    }
}

async fn read_bounded(
    body: Body,
    scope: Scope,
    id: Option<&RequestId>,
) -> Result<Bytes, Box<Response>> {
    match tokio::time::timeout(ADMIN_BODY_TIMEOUT, to_bytes(body, ADMIN_BODY_MAX_BYTES)).await {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(failure)) => {
            let oversized = failure
                .source()
                .is_some_and(|source| source.is::<http_body::LengthLimitError>());
            Err(Box::new(error(
                if oversized {
                    StatusCode::PAYLOAD_TOO_LARGE
                } else {
                    StatusCode::BAD_REQUEST
                },
                if oversized {
                    scope.too_large()
                } else {
                    scope.invalid()
                },
                false,
                id,
            )))
        }
        Err(_) => Err(Box::new(error(
            StatusCode::REQUEST_TIMEOUT,
            "auth.body_timeout",
            true,
            id,
        ))),
    }
}

#[path = "../../../../test/application/unit/admin_axum/body/tests.rs"]
#[cfg(test)]
mod tests;
