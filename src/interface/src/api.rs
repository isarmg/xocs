//! Same-origin, bounded HTTP and serialized in-memory administrator sessions.
pub use crate::contracts::AdministratorSession;
#[path = "api_codes.rs"]
mod codes;
use crate::contracts::*;
use blog_dom::events::EventListener;
use blog_dom::thread_bound::SendWrapper;
use blog_dom::timers::callback::Timeout;
pub use codes::{ApiErrorCode, LocalErrorCode, RemoteErrorCode, RequestId};
use futures::{
    FutureExt,
    future::{LocalBoxFuture, Shared},
    lock::Mutex,
};
use js_sys::{Reflect, Uint8Array};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    fmt,
    rc::Rc,
    sync::Arc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AbortController, AbortSignal, RequestCache, RequestCredentials, RequestInit, RequestRedirect,
    Response, Url,
};

pub const DEFAULT_TIMEOUT_MS: u32 = 10_000;
pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 64 * 1024 * 1024;
pub const AUTH_CHANGE_EVENT: &str = "blog-auth-change";

// A request owns its network operation as well as its timeout. In particular,
// rejecting response headers or dropping the Rust future must not leave an
// unread response downloading after the timeout and caller listener are gone.
struct AbortOnDrop(AbortController);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub status: u16,
    pub code: ApiErrorCode,
    pub message: String,
    pub request_id: Option<RequestId>,
    pub retryable: bool,
    pub retry_after_seconds: Option<u32>,
    #[serde(skip)]
    pub source: Option<Arc<ApiErrorSource>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiErrorSource {
    #[error("JSON conversion failed")]
    Json(#[from] serde_json::Error),
    #[error("invalid numeric response header")]
    InvalidNumber(#[from] std::num::ParseIntError),
    #[error("browser operation failed: {0:?}")]
    JavaScript(SendWrapper<JsValue>),
}

impl From<JsValue> for ApiErrorSource {
    fn from(value: JsValue) -> Self {
        Self::JavaScript(SendWrapper::new(value))
    }
}

impl PartialEq for ApiError {
    fn eq(&self, other: &Self) -> bool {
        // UI equality follows the protocol values, not the diagnostic source's identity.
        self.status == other.status
            && self.code == other.code
            && self.message == other.message
            && self.request_id == other.request_id
            && self.retryable == other.retryable
            && self.retry_after_seconds == other.retry_after_seconds
    }
}
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            crate::i18n::t(
                "请求未能完成，请重试。",
                "The request could not be completed. Please retry."
            )
        )?;
        if let Some(id) = &self.request_id {
            write!(f, " Request ID: {id}")?;
        }
        Ok(())
    }
}
impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|source| source as _)
    }
}
impl ApiError {
    pub fn local(code: LocalErrorCode) -> Self {
        Self {
            status: 0,
            code: code.into(),
            message: code.as_str().into(),
            request_id: None,
            retryable: matches!(
                code,
                LocalErrorCode::NetworkError | LocalErrorCode::RequestTimeout
            ),
            retry_after_seconds: None,
            source: None,
        }
    }

    fn with_source(code: LocalErrorCode, source: impl Into<ApiErrorSource>) -> Self {
        Self {
            source: Some(Arc::new(source.into())),
            ..Self::local(code)
        }
    }
}

#[derive(Clone)]
pub struct RequestOptions {
    pub signal: Option<AbortSignal>,
    pub timeout_ms: u32,
    pub max_response_bytes: usize,
    pub headers: Vec<(String, String)>,
}
impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            signal: None,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            headers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthPhase {
    Loading,
    Anonymous,
    Authenticated,
    LogoutUnconfirmed,
    Error,
}
#[derive(Clone, PartialEq)]
pub struct AuthSnapshot {
    pub session: Option<AdministratorSession>,
    pub phase: AuthPhase,
    pub error: Option<ApiError>,
    pub account_updated: bool,
}
type Restore = Shared<LocalBoxFuture<'static, Result<AdministratorSession, ApiError>>>;
struct State {
    session: Option<AdministratorSession>,
    transport: Option<AdministratorSession>,
    generation: u64,
    phase: AuthPhase,
    error: Option<ApiError>,
    account_updated: bool,
    revoking: Option<u64>,
    restore: Option<(u64, Restore)>,
}
thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State { session: None, transport: None, generation: 0, phase: AuthPhase::Loading, error: None, account_updated: false, revoking: None, restore: None }) };
    static MUTATIONS: Rc<Mutex<()>> = Rc::new(Mutex::new(()));
}
pub fn current_session() -> Option<AdministratorSession> {
    STATE.with(|s| s.borrow().session.clone())
}
pub fn snapshot() -> AuthSnapshot {
    STATE.with(|s| {
        let s = s.borrow();
        AuthSnapshot {
            session: s.session.clone(),
            phase: s.phase.clone(),
            error: s.error.clone(),
            account_updated: s.account_updated,
        }
    })
}
fn changed() {
    if let Some(window) = web_sys::window() {
        let event = web_sys::Event::new(AUTH_CHANGE_EVENT)
            .expect("invariant: the auth event name is valid");
        window
            .dispatch_event(&event)
            .expect("invariant: an initialized auth event can be dispatched");
    }
}
fn invalidate(error: Option<ApiError>) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.generation += 1;
        s.session = None;
        s.restore = None;
        s.phase = if error.is_some() {
            AuthPhase::Error
        } else {
            AuthPhase::Anonymous
        };
        s.error = error;
    });
    changed();
}
fn begin_mutation(revoking: bool) -> u64 {
    let generation = STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.generation += 1;
        if revoking {
            s.revoking = Some(s.generation);
        }
        s.restore = None;
        s.session = None;
        // Keep the revocation warning until a successful login replaces its
        // session or the server confirms logout. A failed login is no proof.
        if s.phase != AuthPhase::LogoutUnconfirmed {
            s.phase = AuthPhase::Anonymous;
            s.error = None;
        }
        s.generation
    });
    changed();
    generation
}
fn require_generation(generation: u64) -> Result<(), ApiError> {
    if STATE.with(|s| s.borrow().generation) == generation {
        Ok(())
    } else {
        Err(ApiError::local(LocalErrorCode::AuthOperationSuperseded))
    }
}
fn publish(
    session: AdministratorSession,
    generation: u64,
) -> Result<AdministratorSession, ApiError> {
    STATE.with(|s| s.borrow_mut().transport = Some(session.clone()));
    require_generation(generation)?;
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.session = Some(session.clone());
        s.phase = AuthPhase::Authenticated;
        s.error = None;
        s.account_updated = false;
        s.revoking = None;
    });
    changed();
    Ok(session)
}

pub fn validate_api_path(path: &str) -> bool {
    path.starts_with("/api/v1/")
        && !path.contains('#')
        && !path.contains('\\')
        && !path.chars().any(|c| c <= '\u{1f}' || c == '\u{7f}')
}
fn resolve(path: &str) -> Result<String, ApiError> {
    if !validate_api_path(path) {
        return Err(ApiError::local(LocalErrorCode::InvalidApiPath));
    }
    let window =
        web_sys::window().ok_or_else(|| ApiError::local(LocalErrorCode::BrowserRequired))?;
    let base = Url::new(
        &window
            .location()
            .href()
            .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidApiPath, source))?,
    )
    .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidApiPath, source))?;
    let url = Url::new_with_base(path, &base.href())
        .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidApiPath, source))?;
    if url.origin() != base.origin()
        || !url.pathname().starts_with("/api/v1/")
        || !url.username().is_empty()
        || !url.password().is_empty()
        || !["http:", "https:"].contains(&url.protocol().as_str())
    {
        return Err(ApiError::local(LocalErrorCode::InvalidApiPath));
    }
    Ok(url.href())
}

/// Serialize a typed request without silently replacing serialization failures with null.
pub fn encode_json_body<T: Serialize + ?Sized>(body: &T) -> Result<Value, ApiError> {
    serde_json::to_value(body)
        .map_err(|source| ApiError::with_source(LocalErrorCode::BodySerializationFailed, source))
}

/// Decode a successful JSON response. Nullable or empty responses require `Option<T>` or `()`.
pub fn decode_response<T: DeserializeOwned>(value: Value) -> Result<T, ApiError> {
    serde_json::from_value(value)
        .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidTypedResponseShape, source))
}

/// Keep product-specific bounds, revisions and path binding before constructing a DTO.
pub fn decode_validated_response<T: DeserializeOwned>(
    value: Value,
    validate: impl FnOnce(&Value) -> bool,
) -> Result<T, ApiError> {
    if !validate(&value) {
        return Err(ApiError::local(LocalErrorCode::InvalidResponseContract));
    }
    decode_response(value)
}

pub async fn request_typed<T: DeserializeOwned>(
    path: &str,
    method: &str,
    body: Option<Value>,
) -> Result<T, ApiError> {
    request_typed_with_options(path, method, body, RequestOptions::default()).await
}

/// Same bounded transport, cancellation and session handling as `request_with_options`.
pub async fn request_typed_with_options<T: DeserializeOwned>(
    path: &str,
    method: &str,
    body: Option<Value>,
    options: RequestOptions,
) -> Result<T, ApiError> {
    decode_response(request_with_options(path, method, body, options).await?)
}

/// Deserialize only after the existing runtime response contract has accepted the wire value.
pub async fn request_typed_validated_with_options<T: DeserializeOwned>(
    path: &str,
    method: &str,
    body: Option<Value>,
    options: RequestOptions,
    validate: impl FnOnce(&Value) -> bool,
) -> Result<T, ApiError> {
    decode_validated_response(
        request_with_options(path, method, body, options).await?,
        validate,
    )
}

pub async fn request(path: &str, method: &str, body: Option<Value>) -> Result<Value, ApiError> {
    request_with_options(path, method, body, RequestOptions::default()).await
}
pub async fn request_with_options(
    path: &str,
    method: &str,
    body: Option<Value>,
    options: RequestOptions,
) -> Result<Value, ApiError> {
    let (generation, token) = STATE.with(|s| {
        let s = s.borrow();
        (
            s.generation,
            s.session.as_ref().map(|v| v.csrf_token.clone()),
        )
    });
    send(path, method, body, options, token, generation).await
}

/// Stream the browser's Blob directly; do not copy uploads into WASM memory.
pub async fn request_blob(
    path: &str,
    method: &str,
    body: web_sys::Blob,
    options: RequestOptions,
) -> Result<Value, ApiError> {
    let (generation, token) = STATE.with(|s| {
        let s = s.borrow();
        (
            s.generation,
            s.session.as_ref().map(|v| v.csrf_token.clone()),
        )
    });
    send_body(path, method, None, Some(body), options, token, generation).await
}

async fn send(
    path: &str,
    method: &str,
    body: Option<Value>,
    options: RequestOptions,
    token: Option<String>,
    generation: u64,
) -> Result<Value, ApiError> {
    send_body(path, method, body, None, options, token, generation).await
}

async fn send_body(
    path: &str,
    method: &str,
    body: Option<Value>,
    raw_body: Option<web_sys::Blob>,
    options: RequestOptions,
    token: Option<String>,
    generation: u64,
) -> Result<Value, ApiError> {
    if !(1..=120_000).contains(&options.timeout_ms)
        || !(1..=MAX_RESPONSE_BYTES).contains(&options.max_response_bytes)
    {
        return Err(ApiError::local(LocalErrorCode::InvalidRequestBudget));
    }
    let target = resolve(path)?;
    let controller = AbortController::new()
        .map_err(|source| ApiError::with_source(LocalErrorCode::NetworkError, source))?;
    let _abort = AbortOnDrop(controller.clone());
    let timed_out = Rc::new(Cell::new(false));
    let timeout = {
        let controller = controller.clone();
        let timed_out = timed_out.clone();
        Timeout::new(options.timeout_ms, move || {
            timed_out.set(true);
            controller.abort();
        })
    };
    let _caller = options.signal.as_ref().map(|signal| {
        if signal.aborted() {
            controller.abort();
        }
        let controller = controller.clone();
        EventListener::once(signal, "abort", move |_| controller.abort())
    });
    let init = RequestInit::new();
    let method = method.to_ascii_uppercase();
    init.set_method(&method);
    init.set_credentials(RequestCredentials::SameOrigin);
    init.set_cache(RequestCache::NoStore);
    init.set_redirect(RequestRedirect::Error);
    init.set_signal(Some(&controller.signal()));
    let headers = web_sys::Headers::new()
        .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))?;
    headers
        .set("accept", "application/json")
        .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))?;
    for (name, value) in &options.headers {
        if name.eq_ignore_ascii_case("x-csrf-token") {
            return Err(ApiError::local(LocalErrorCode::CsrfHeaderOwnedBySession));
        }
        headers
            .set(name, value)
            .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))?;
    }
    if !["GET", "HEAD", "OPTIONS", "TRACE"].contains(&method.as_str())
        && let Some(token) = token.as_deref()
    {
        headers
            .set("x-csrf-token", token)
            .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))?;
    }
    if let Some(body) = body {
        headers
            .set("content-type", "application/json")
            .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))?;
        init.set_body(&JsValue::from_str(&body.to_string()));
    }
    if let Some(body) = raw_body {
        if !headers
            .has("content-type")
            .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))?
        {
            let media = body.type_();
            headers
                .set(
                    "content-type",
                    if media.is_empty() {
                        "application/octet-stream"
                    } else {
                        &media
                    },
                )
                .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))?;
        }
        init.set_body(body.as_ref());
    }
    init.set_headers(&headers);
    let operation = async {
        let response: Response = JsFuture::from(
            web_sys::window()
                .unwrap()
                .fetch_with_str_and_init(&target, &init),
        )
        .await
        .map_err(|source| ApiError::with_source(LocalErrorCode::NetworkError, source))?
        .dyn_into()
        .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidResponseShape, source))?;
        // Malformed optional diagnostic identifiers are omitted from the UI.
        let request_id = response_header(&response, "x-request-id")?
            .and_then(|value| RequestId::try_from(value).ok());
        let status = response.status();
        // Invalidate on headers, even if a malformed/slow 401 body cannot be read.
        if status == 401
            && path != LOGIN_PATH
            && path != LOGOUT_PATH
            && require_generation(generation).is_ok()
        {
            invalidate(None);
        }
        if status == 204
            || status == 205
            || method == "HEAD"
            || response_header(&response, "content-length")?.as_deref() == Some("0")
        {
            return if response.ok() {
                Ok(Value::Null)
            } else {
                Err(ApiError {
                    status,
                    request_id,
                    ..ApiError::local(LocalErrorCode::InvalidErrorResponse)
                })
            };
        }
        let is_json = response_header(&response, "content-type")?.is_some_and(|v| {
            let media = v
                .split(';')
                .next()
                .expect("invariant: split always yields a first segment")
                .trim()
                .to_ascii_lowercase();
            media == "application/json" || media.ends_with("+json")
        });
        // The UI supports delay-seconds, not the alternative HTTP-date form.
        let retry_after_seconds = response_header(&response, "retry-after")?
            .and_then(|value| value.parse::<u64>().ok())
            .map(|v| v.min(86_400) as u32);
        if !is_json {
            return Err(ApiError {
                status,
                request_id,
                retry_after_seconds,
                ..ApiError::local(if response.ok() {
                    LocalErrorCode::InvalidContentType
                } else {
                    LocalErrorCode::InvalidErrorResponse
                })
            });
        }
        let limit = if response.ok() {
            options.max_response_bytes
        } else {
            64 * 1024
        };
        let bytes = read_bounded(&response, limit).await.map_err(|mut error| {
            error.status = status;
            error.request_id = request_id.clone();
            error
        })?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|source| ApiError {
            status,
            request_id: request_id.clone(),
            ..ApiError::with_source(
                if response.ok() {
                    LocalErrorCode::InvalidJsonResponse
                } else {
                    LocalErrorCode::InvalidErrorResponse
                },
                source,
            )
        })?;
        if response.ok() {
            return Ok(value);
        }
        let envelope = decode_error_envelope(value).map_err(|error| ApiError {
            status,
            request_id: request_id.clone(),
            retry_after_seconds,
            ..error
        })?;
        Err(ApiError {
            status,
            code: envelope.code,
            message: envelope.message,
            request_id: envelope.request_id.or(request_id),
            retryable: envelope.retryable,
            retry_after_seconds,
            source: None,
        })
    }
    .await;
    drop(timeout);
    if timed_out.get() {
        return Err(ApiError::local(LocalErrorCode::RequestTimeout));
    }
    if options.signal.is_some_and(|signal| signal.aborted()) {
        return Err(ApiError::local(LocalErrorCode::RequestAborted));
    }
    operation
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ErrorEnvelope {
    code: ApiErrorCode,
    message: String,
    retryable: bool,
    #[serde(default, deserialize_with = "non_null_request_id")]
    request_id: Option<RequestId>,
    #[serde(default, rename = "details", deserialize_with = "non_null_details")]
    _details: Option<serde_json::Map<String, Value>>,
}

fn non_null_request_id<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<RequestId>, D::Error> {
    RequestId::deserialize(deserializer).map(Some)
}

fn non_null_details<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<serde_json::Map<String, Value>>, D::Error> {
    serde_json::Map::deserialize(deserializer).map(Some)
}

fn decode_error_envelope(value: Value) -> Result<ErrorEnvelope, ApiError> {
    serde_json::from_value(value)
        .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidErrorResponse, source))
}

fn response_header(response: &Response, name: &str) -> Result<Option<String>, ApiError> {
    response
        .headers()
        .get(name)
        .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))
}

async fn read_bounded(response: &Response, maximum: usize) -> Result<Vec<u8>, ApiError> {
    if let Some(value) = response_header(response, "content-length")? {
        let length = value
            .parse::<u64>()
            .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidHeaders, source))?;
        if length > maximum as u64 {
            return Err(ApiError::local(LocalErrorCode::ResponseTooLarge));
        }
    }
    let Some(body) = response.body() else {
        return Ok(Vec::new());
    };
    let reader: web_sys::ReadableStreamDefaultReader =
        body.get_reader().dyn_into().map_err(|source| {
            ApiError::with_source(LocalErrorCode::InvalidResponseShape, JsValue::from(source))
        })?;
    let result = async {
        let mut bytes = Vec::new();
        loop {
            let item = JsFuture::from(reader.read())
                .await
                .map_err(|source| ApiError::with_source(LocalErrorCode::NetworkError, source))?;
            let done = Reflect::get(&item, &"done".into())
                .map_err(|source| {
                    ApiError::with_source(LocalErrorCode::InvalidResponseShape, source)
                })?
                .as_bool()
                .ok_or_else(|| ApiError::local(LocalErrorCode::InvalidResponseShape))?;
            if done {
                break;
            }
            let value = Reflect::get(&item, &"value".into()).map_err(|source| {
                ApiError::with_source(LocalErrorCode::InvalidResponseShape, source)
            })?;
            let chunk = Uint8Array::new(&value);
            if bytes.len().saturating_add(chunk.length() as usize) > maximum {
                return Err(ApiError::local(LocalErrorCode::ResponseTooLarge));
            }
            bytes.extend(chunk.to_vec());
        }
        Ok(bytes)
    }
    .await;
    // AbortOnDrop owns cancellation on every exit, including oversized or unread bodies.
    reader.release_lock();
    result
}

fn session(value: Value) -> Result<AdministratorSession, ApiError> {
    let session: AdministratorSession = serde_json::from_value(value)
        .map_err(|source| ApiError::with_source(LocalErrorCode::InvalidResponseShape, source))?;
    if !session.validate() {
        return Err(ApiError::local(LocalErrorCode::InvalidResponseShape));
    }
    Ok(session)
}

pub async fn login(username: &str, password: &str) -> Result<AdministratorSession, ApiError> {
    if !valid_username_candidate(username) || !valid_password(password) {
        return Err(ApiError::local(LocalErrorCode::InvalidCredentials));
    }
    let generation = begin_mutation(false);
    let mutex = MUTATIONS.with(Clone::clone);
    let _guard = mutex.lock().await;
    let received = send(
        LOGIN_PATH,
        "POST",
        Some(json!({"username":username,"password":password})),
        RequestOptions::default(),
        None,
        generation,
    )
    .await?;
    publish(session(received)?, generation)
}

pub async fn restore() -> Result<AdministratorSession, ApiError> {
    // A cookie may still be valid while logout is pending or failed. Restoring it
    // would undo the user's local sign-out and discard the revocation warning.
    // Only an explicit successful login or confirmed logout ends this barrier.
    if STATE.with(|s| s.borrow().revoking.is_some()) {
        return Err(ApiError::local(LocalErrorCode::AuthLogoutUnconfirmed));
    }
    let (generation, pending) = STATE.with(|s| {
        let mut s = s.borrow_mut();
        if let Some((generation, pending)) = &s.restore {
            return (*generation, pending.clone());
        }
        let generation = s.generation;
        let pending = async move {
            let mutex = MUTATIONS.with(Clone::clone);
            let _guard = mutex.lock().await;
            let received = send(
                SESSION_PATH,
                "GET",
                None,
                RequestOptions::default(),
                None,
                generation,
            )
            .await
            .and_then(session);
            match received {
                Ok(session) => publish(session, generation),
                Err(error) => {
                    if require_generation(generation).is_ok() {
                        if error.status == 401 {
                            invalidate(None);
                        } else {
                            invalidate(Some(error.clone()));
                        }
                    }
                    Err(error)
                }
            }
        }
        .boxed_local()
        .shared();
        s.restore = Some((generation, pending.clone()));
        (generation, pending)
    });
    let result = pending.await;
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.restore.as_ref().is_some_and(|(g, _)| *g == generation) {
            s.restore = None;
        }
    });
    result
}

pub async fn logout() -> Result<(), ApiError> {
    let generation = begin_mutation(true);
    let mutex = MUTATIONS.with(Clone::clone);
    let _guard = mutex.lock().await;
    let token = STATE.with(|s| s.borrow().transport.as_ref().map(|v| v.csrf_token.clone()));
    let result = send(
        LOGOUT_PATH,
        "POST",
        None,
        RequestOptions::default(),
        token,
        generation,
    )
    .await
    .and_then(|value| {
        if value.is_null() {
            Ok(())
        } else {
            Err(ApiError::local(LocalErrorCode::InvalidResponseShape))
        }
    });
    let error = match result {
        Ok(()) => None,
        Err(error)
            if error.status == 401
                && error.code == ApiErrorCode::Remote(RemoteErrorCode::SessionRequired) =>
        {
            None
        }
        Err(error) => Some(error),
    };
    let notify = STATE.with(|s| settle_revocation(&mut s.borrow_mut(), generation, error.as_ref()));
    if notify {
        changed();
    }
    if let Some(error) = error {
        return Err(error);
    }
    require_generation(generation)
}

// Called while holding MUTATIONS: no later login has reached the server yet.
// Its queued intent can supersede this operation's return value, but cannot
// undo the server's revocation result. A newer logout owns its own barrier.
fn settle_revocation(state: &mut State, generation: u64, error: Option<&ApiError>) -> bool {
    if error.is_none() {
        state.transport = None;
    }
    if state.revoking != Some(generation) {
        return false;
    }
    if let Some(error) = error {
        state.phase = AuthPhase::LogoutUnconfirmed;
        state.error = Some(error.clone());
    } else {
        state.revoking = None;
        state.phase = AuthPhase::Anonymous;
        state.error = None;
    }
    true
}

pub async fn update_account(input: Value) -> Result<(), ApiError> {
    let object = input
        .as_object()
        .ok_or_else(|| ApiError::local(LocalErrorCode::InvalidAccount))?;
    if !object
        .keys()
        .all(|k| ["username", "current_password", "new_password"].contains(&k.as_str()))
        || !input
            .get("username")
            .and_then(Value::as_str)
            .is_some_and(valid_username_candidate)
        || !input
            .get("current_password")
            .and_then(Value::as_str)
            .is_some_and(|p| !p.is_empty() && p.len() <= 1024)
        || input
            .get("new_password")
            .is_some_and(|p| !p.as_str().is_some_and(valid_password))
    {
        return Err(ApiError::local(LocalErrorCode::InvalidAccount));
    }
    let generation = STATE.with(|s| s.borrow().generation);
    let mutex = MUTATIONS.with(Clone::clone);
    let _guard = mutex.lock().await;
    require_generation(generation)?;
    let result = request(ACCOUNT_PATH, "POST", Some(input)).await?;
    if !result.is_null() {
        return Err(ApiError::local(LocalErrorCode::InvalidResponseShape));
    }
    require_generation(generation)?;
    invalidate(None);
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.transport = None;
        s.account_updated = true;
    });
    changed();
    Ok(())
}

#[path = "../../../test/interface/unit/api/tests.rs"]
#[cfg(test)]
mod tests;
