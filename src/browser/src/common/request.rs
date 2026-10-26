//! Contract-checked reads and the page-owned state of the latest request.
use super::t;
pub(crate) use blog_interface::api::{ApiError, ApiErrorCode, LocalErrorCode, RequestOptions};
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub(crate) fn invalid() -> ApiError {
    ApiError {
        status: 0,
        code: LocalErrorCode::InvalidResponse.into(),
        message: t(
            "服务器响应格式无效，无法确认操作结果。",
            "The server response was invalid. The operation result could not be confirmed.",
        ),
        request_id: None,
        retryable: false,
        retry_after_seconds: None,
        source: None,
    }
}

pub(crate) async fn request(
    path: &str,
    method: &str,
    body: Option<Value>,
    options: RequestOptions,
) -> Result<Value, ApiError> {
    let v = blog_interface::api::request_with_options(path, method, body, options).await?;
    if crate::contracts::validate(path, method, &v)
        && (method != "GET" || crate::contracts::matches_page(path, &v))
    {
        Ok(v)
    } else {
        Err(invalid())
    }
}

pub(crate) struct Data<T: Send + Sync + 'static = Value> {
    pub value: RwSignal<Option<T>>,
    pub previous: RwSignal<Option<T>>,
    pub error: RwSignal<Option<ApiError>>,
    pub busy: RwSignal<bool>,
    pub revision: RwSignal<u64>,
}

impl<T: Send + Sync + 'static> Copy for Data<T> {}

impl<T: Send + Sync + 'static> Clone for Data<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Send + Sync + 'static> Data<T> {
    pub fn reload(self) {
        self.revision.update(|v| *v += 1)
    }
}

pub(crate) fn load(path: Signal<String>) -> Data {
    load_typed(path)
}

async fn request_typed<T: DeserializeOwned>(
    path: &str,
    method: &str,
    body: Option<Value>,
    options: RequestOptions,
) -> Result<T, ApiError> {
    blog_interface::api::request_typed_validated_with_options(
        path,
        method,
        body,
        options,
        |value| {
            crate::contracts::validate(path, method, value)
                && (method != "GET" || crate::contracts::matches_page(path, value))
        },
    )
    .await
}

pub(crate) fn load_typed<T: DeserializeOwned + Clone + Send + Sync + 'static>(
    path: Signal<String>,
) -> Data<T> {
    let data = Data {
        value: RwSignal::new(None),
        previous: RwSignal::new(None),
        error: RwSignal::new(None),
        busy: RwSignal::new(false),
        revision: RwSignal::new(0),
    };
    Effect::new(move |_| {
        let path = path.get();
        data.revision.get();
        data.value.set(None);
        data.error.set(None);
        if path.is_empty() {
            data.busy.set(false);
            return;
        }
        data.busy.set(true);
        let controller = web_sys::AbortController::new()
            .expect("invariant: AbortController construction has no failure conditions");
        let signal = controller.signal();
        let cleanup = SendWrapper::new(controller);
        on_cleanup(move || cleanup.abort());
        blog_interface::task::spawn_local(async move {
            let result = request_typed::<T>(
                &path,
                "GET",
                None,
                RequestOptions {
                    signal: Some(signal.clone()),
                    max_response_bytes: 4 * 1024 * 1024,
                    ..Default::default()
                },
            )
            .await;
            if signal.aborted() {
                return;
            }
            match result {
                Ok(v) => {
                    data.previous.set(Some(v.clone()));
                    data.value.set(Some(v));
                }
                Err(e) => data.error.set(Some(e)),
            }
            data.busy.set(false);
        });
    });
    data
}
