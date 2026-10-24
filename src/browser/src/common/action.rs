//! Product mutations and uploads: serialized work, cancellation and conflict feedback.
use super::{ApiError, RequestOptions, enc, invalid, request, t};
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use serde::de::DeserializeOwned;
use serde_json::Value;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
pub(crate) struct Action {
    pub busy: RwSignal<bool>,
    pub error: RwSignal<Option<ApiError>>,
    controller: StoredValue<SendWrapper<Option<web_sys::AbortController>>>,
}

impl Action {
    pub fn new() -> Self {
        let s = Self {
            busy: RwSignal::new(false),
            error: RwSignal::new(None),
            controller: StoredValue::new(SendWrapper::new(None)),
        };
        on_cleanup(move || s.abort());
        s
    }
    pub fn abort(self) {
        self.controller.update_value(|c| {
            if let Some(c) = (**c).take() {
                c.abort()
            }
        });
        let _ = self.busy.try_set(false);
    }
    pub fn run(
        self,
        path: String,
        method: &'static str,
        body: Option<Value>,
        done: impl FnOnce(Value) + 'static,
    ) {
        self.run_options(path, method, body, RequestOptions::default(), done)
    }
    pub fn run_typed<T: DeserializeOwned + 'static>(
        self,
        path: String,
        method: &'static str,
        body: Option<Value>,
        done: impl FnOnce(T) + 'static,
    ) {
        self.run_typed_options(path, method, body, RequestOptions::default(), done)
    }
    pub fn run_typed_options<T: DeserializeOwned + 'static>(
        self,
        path: String,
        method: &'static str,
        body: Option<Value>,
        options: RequestOptions,
        done: impl FnOnce(T) + 'static,
    ) {
        let error = self.error;
        self.run_options(
            path,
            method,
            body,
            options,
            move |value| match serde_json::from_value(value) {
                Ok(value) => done(value),
                Err(_) => error.set(Some(invalid())),
            },
        );
    }
    pub fn run_options(
        self,
        path: String,
        method: &'static str,
        body: Option<Value>,
        mut opts: RequestOptions,
        done: impl FnOnce(Value) + 'static,
    ) {
        if self.busy.get_untracked() {
            return;
        }
        self.busy.set(true);
        self.error.set(None);
        let c = web_sys::AbortController::new().unwrap();
        let signal = c.signal();
        opts.signal = Some(signal.clone());
        self.controller.set_value(SendWrapper::new(Some(c)));
        blog_interface::task::spawn_local(async move {
            let r = request(&path, method, body, opts).await;
            if signal.aborted() {
                return;
            }
            match r {
                Ok(v) => done(v),
                Err(mut e) => {
                    if e.status == 409 && path.starts_with("/api/v1/content/resources/") {
                        e.code = "display_error".into();
                        e.message = t(
                            "文件仍被引用或记录已变化。请刷新列表，移除引用后再重试。",
                            "This file is referenced or its record has changed. Refresh the list and remove its references before retrying.",
                        );
                    } else if e.status == 409
                        && method == "PUT"
                        && path.starts_with("/api/v1/content/labels/")
                    {
                        e.code = "display_error".into();
                        e.message = t(
                            "标签仍被文章使用，不能更改分类。请先修改文章的分类和标签。",
                            "This tag is still used by articles. Update their categories and tags before moving this tag.",
                        );
                    }
                    self.error.set(Some(e));
                }
            }
            self.controller.set_value(SendWrapper::new(None));
            self.busy.set(false);
        });
    }
}

impl Action {
    pub fn upload(self, file: SendWrapper<web_sys::File>, done: impl FnOnce(Value) + 'static) {
        self.upload_file(file, false, done);
    }

    pub fn upload_video(
        self,
        file: SendWrapper<web_sys::File>,
        done: impl FnOnce(Value) + 'static,
    ) {
        self.upload_file(file, true, done);
    }

    fn upload_file(
        self,
        file: SendWrapper<web_sys::File>,
        video: bool,
        done: impl FnOnce(Value) + 'static,
    ) {
        if self.busy.get_untracked() {
            return;
        }
        let (limit, accepted, route) = if video {
            (
                50.,
                &["video/mp4", "video/webm"][..],
                "/api/v1/content/upload-video",
            )
        } else {
            (
                10.,
                &["image/png", "image/jpeg", "image/gif", "image/webp"][..],
                "/api/v1/content/upload",
            )
        };
        if file.size() <= 0.
            || file.size() > limit * 1024. * 1024.
            || !accepted.contains(&file.type_().as_str())
        {
            self.error.set(Some(ApiError {
                message: if video {
                    t(
                        "请选择不超过 50 MB 的 MP4 或 WebM 视频",
                        "Choose an MP4 or WebM video up to 50 MB",
                    )
                } else {
                    t(
                        "请选择不超过 10 MB 的 PNG、JPEG、GIF 或 WebP 图片",
                        "Choose a PNG, JPEG, GIF, or WebP image up to 10 MB",
                    )
                },
                code: "display_error".into(),
                ..invalid()
            }));
            return;
        }
        self.busy.set(true);
        self.error.set(None);
        let c = web_sys::AbortController::new().unwrap();
        let signal = c.signal();
        self.controller.set_value(SendWrapper::new(Some(c)));
        blog_interface::task::spawn_local(async move {
            let headers = vec![
                ("Content-Type".into(), file.type_()),
                (
                    "X-File-Name".into(),
                    enc(crate::dates::filename_prefix(&file.name())),
                ),
            ];
            let body: web_sys::Blob = (*file).clone().unchecked_into();
            let r = blog_interface::api::request_blob(
                route,
                "POST",
                body,
                RequestOptions {
                    signal: Some(signal.clone()),
                    timeout_ms: if video {
                        120_000
                    } else {
                        blog_interface::api::DEFAULT_TIMEOUT_MS
                    },
                    headers,
                    ..Default::default()
                },
            )
            .await;
            if signal.aborted() {
                return;
            }
            match r {
                Ok(v)
                    if if video {
                        crate::contracts::upload_video(&v)
                    } else {
                        crate::contracts::upload(&v)
                    } =>
                {
                    done(v)
                }
                Ok(_) => self.error.set(Some(invalid())),
                Err(e) => self.error.set(Some(e)),
            }
            self.controller.set_value(SendWrapper::new(None));
            self.busy.set(false);
        });
    }
}
