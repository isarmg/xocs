//! Visible read, empty and error states with product-specific message policy.
use super::{ApiError, ApiErrorCode, Data, LocalErrorCode, rows, t};
use blog_interface::prelude::*;

pub(crate) fn public_error(e: &ApiError, fallback: &str) -> String {
    if e.code == ApiErrorCode::Local(LocalErrorCode::DisplayError) {
        return e.message.clone();
    }
    if matches!(
        e.code,
        ApiErrorCode::Local(
            LocalErrorCode::InvalidResponse
                | LocalErrorCode::InvalidJsonResponse
                | LocalErrorCode::InvalidResponseShape
                | LocalErrorCode::InvalidTypedResponseShape
                | LocalErrorCode::InvalidResponseContract
                | LocalErrorCode::InvalidContentType
        )
    ) {
        return t(
            "服务器响应格式无效，无法确认操作结果。",
            "The server response was invalid. The operation result could not be confirmed.",
        );
    }
    match e.status {
        400 => t("请检查输入内容后重试。", "Check the input and try again."),
        401 => t(
            "请检查登录信息，或重新登录后重试。",
            "Check your credentials or sign in again.",
        ),
        403 => t(
            "没有执行此操作的权限。",
            "You do not have permission to perform this action.",
        ),
        404 => t(
            "内容不存在或已被删除。",
            "This content was not found or has been deleted.",
        ),
        409 => t(
            "内容或状态已变化，请刷新后重试。",
            "The content or state has changed. Refresh and try again.",
        ),
        413 => t(
            "上传内容过大，请缩小后重试。",
            "The upload is too large. Reduce its size and try again.",
        ),
        429 => t(
            "请求过于频繁，请稍后重试。",
            "Too many requests. Please try again later.",
        ),
        500.. => t(
            "服务暂时不可用，请稍后重试。",
            "The service is temporarily unavailable. Please try again later.",
        ),
        _ => fallback.into(),
    }
}

#[component]
pub(crate) fn Feedback(
    error: RwSignal<Option<ApiError>>,
    #[prop(default=t("请求失败，请重试。","Request failed. Please retry."))] fallback: String,
    #[prop(optional_no_strip)] retry: Option<Callback<()>>,
    #[prop(default = false)] public: bool,
) -> impl IntoView {
    move || {
        error.get().map(|e| {
            let message = if !public && retry.is_some() {
                fallback.clone()
            } else {
                public_error(&e, &fallback)
            };
            if public {
                View::new(
                    blog_interface::html::p()
                        .child((
                            message,
                            retry.map(|retry| {
                                View::new(
                                    blog_interface::html::button()
                                        .child(t("重试", "Try again"))
                                        .r#type("button")
                                        .on(blog_interface::ev::click, move |_| retry.run(())),
                                )
                            }),
                        ))
                        .role("alert"),
                )
                .into_any()
            } else {
                View::new(
                    blog_interface::html::div()
                        .child((
                            message,
                            e.request_id.map(|id| {
                                View::new(
                                    blog_interface::html::p()
                                        .child((
                                            t("请求标识：", "Request ID: "),
                                            blog_interface::html::code().child(id.to_string()),
                                        ))
                                        .class("blog-request-id"),
                                )
                            }),
                            retry.map(|retry| {
                                View::new(
                                    blog_interface::html::button()
                                        .child(t("重试", "Try again"))
                                        .class("blog-button")
                                        .r#type("button")
                                        .on(blog_interface::ev::click, move |_| retry.run(())),
                                )
                            }),
                        ))
                        .class("blog-error")
                        .role("alert"),
                )
                .into_any()
            }
        })
    }
}

#[component]
pub(crate) fn ListState(
    data: Data,
    empty: String,
    #[prop(default = false)] array: bool,
) -> impl IntoView {
    move || {
        if data.busy.get() {
            View::new(
                blog_interface::html::div()
                    .child(t("正在加载…", "Loading…"))
                    .class("blog-loading")
                    .role("status"),
            )
            .into_any()
        } else if data.value.get().is_some_and(|v| {
            if array {
                v.as_array().is_some_and(|a| a.is_empty())
            } else {
                rows(&v, "items").is_empty()
            }
        }) {
            View::new(
                blog_interface::html::div()
                    .child(empty.clone())
                    .class("blog-empty"),
            )
            .into_any()
        } else {
            ().into_any()
        }
    }
}
