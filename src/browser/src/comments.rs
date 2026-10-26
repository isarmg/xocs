use crate::common::*;
use blog_interface::component::component_props_builder;
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use serde_json::{Value, json};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommentKind {
    Article,
    Message,
    Love,
}

impl CommentKind {
    fn endpoint(self) -> &'static str {
        match self {
            Self::Article => "/api/v1/comments",
            Self::Message => "/api/v1/message-comments",
            Self::Love => "/api/v1/love-comments",
        }
    }
}
fn commenter(v: &Value) -> String {
    let name = text(v, "username");
    if name.is_empty() {
        t("匿名访客", "Anonymous guest")
    } else {
        name
    }
}
pub fn valid_submission(v: &Value) -> bool {
    let id = text(v, "request_id");
    let b = id.as_bytes();
    b.len() == 36
        && b[14] == b'4'
        && [8, 13, 18, 23].iter().all(|i| b[*i] == b'-')
        && b.iter().enumerate().all(|(i, c)| {
            [8, 13, 18, 23].contains(&i) || c.is_ascii_digit() || (b'a'..=b'f').contains(c)
        })
        && b"89ab".contains(&b[19])
        && v["content"].as_str().is_some_and(|s| !s.trim().is_empty())
        && v.get("parent_comment_id").is_some_and(|v| {
            v.is_null()
                || v.as_u64()
                    .is_some_and(|i| i > 0 && i <= 9_007_199_254_740_991)
        })
        && v.get("article_id").is_none_or(|v| {
            v.as_u64()
                .is_some_and(|i| i > 0 && i <= 9_007_199_254_740_991)
        })
}
#[derive(Debug, thiserror::Error)]
enum PendingError {
    #[error("session storage is unavailable")]
    Unavailable,
    #[error("session storage failed: {0:?}")]
    Browser(wasm_bindgen::JsValue),
    #[error("pending comment is not valid JSON")]
    Json(#[from] serde_json::Error),
    #[error("pending comment has an invalid submission shape")]
    InvalidSubmission,
}

fn storage() -> Result<web_sys::Storage, PendingError> {
    web_sys::window()
        .expect("invariant: comments run in a browser window")
        .session_storage()
        .map_err(PendingError::Browser)?
        .ok_or(PendingError::Unavailable)
}

fn clear_pending(key: &str) -> Result<(), PendingError> {
    storage()?.remove_item(key).map_err(PendingError::Browser)
}

fn read_pending(key: &str) -> Result<Option<Value>, PendingError> {
    let Some(raw) = storage()?.get_item(key).map_err(PendingError::Browser)? else {
        return Ok(None);
    };
    let v = serde_json::from_str(&raw)?;
    if !valid_submission(&v) {
        return Err(PendingError::InvalidSubmission);
    }
    let mut normalized = json!({"request_id":v["request_id"],"content":v["content"],"parent_comment_id":v["parent_comment_id"]});
    if let Some(article) = v.get("article_id") {
        normalized["article_id"] = article.clone();
    }
    Ok(Some(normalized))
}
#[component]
pub(crate) fn Comments(
    #[prop(default = CommentKind::Article)] kind: CommentKind,
    #[prop(default = 0)] article_id: i64,
    #[prop(optional)] title: String,
) -> impl IntoView {
    let path = kind.endpoint();
    let key = format!("blog.comment-submission:{path}:{article_id}");
    let (restored, unavailable) = match read_pending(&key) {
        Ok(pending) => (pending, false),
        Err(_) => (None, true),
    };
    let pending = RwSignal::new(restored);
    let storage_failure = RwSignal::new(unavailable);
    let draft = RwSignal::new(
        pending
            .get_untracked()
            .map(|v| text(&v, "content"))
            .unwrap_or_default(),
    );
    let reply = RwSignal::new(None::<Value>);
    let failure = RwSignal::new(String::new());
    let feedback = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let page = RwSignal::new(1u64);
    let url = Signal::derive(move || {
        format!(
            "{path}?page={}&size=10{}",
            page.get(),
            if kind == CommentKind::Article {
                format!("&article_id={article_id}")
            } else {
                String::new()
            }
        )
    });
    let data = load(url);
    let controller = StoredValue::new(SendWrapper::new(None::<web_sys::AbortController>));
    on_cleanup(move || {
        controller.update_value(|c| {
            if let Some(c) = (**c).take() {
                c.abort()
            }
        })
    });
    let key = StoredValue::new(key);
    let choose = Callback::new(move |v: Value| {
        if !busy.get_untracked()
            && pending.get_untracked().is_none()
            && !storage_failure.get_untracked()
        {
            reply.set(Some(v))
        }
    });
    View::new(
        blog_interface::html::section()
            .child((
                blog_interface::html::h3().child((
                    if title.is_empty() {
                        t("评论", "Comments")
                    } else {
                        title
                    },
                    " | ",
                    move || {
                        format!(
                            "{} {}",
                            data.previous.get().map(|v| num(&v, "total")).unwrap_or(0),
                            t("条评论", "comments")
                        )
                    },
                )),
                move || {
                    (!failure.get().is_empty()).then(|| {
                        View::new(blog_interface::html::p().child(failure.get()).role("alert"))
                    })
                },
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .fallback(t("评论加载失败", "Unable to load comments"))
                        .public(true)
                        .retry(Some(Callback::new(move |()| data.reload())))
                        .build(),
                ),
                move || {
                    data.busy.get().then(|| {
                        View::new(
                            blog_interface::html::p()
                                .child(t("评论加载中…", "Loading comments…"))
                                .role("status"),
                        )
                    })
                },
                move || {
                    storage_failure.get().then(|| {
                        View::new(blog_interface::html::p().child(t("无法读取或保存待确认的提交。请先检查评论，再清除待确认状态；当前不会发送新评论。", "The pending submission could not be read or saved. Check the comments before clearing it. No new comment will be sent.")).role("alert"))
                    })
                },
                move || {
                    (pending.get().is_some() && failure.get().is_empty()).then(|| {
                        View::new(blog_interface::html::p().child(t("有一条评论等待确认，请继续确认原提交。", "A comment is awaiting confirmation. Continue confirming the original submission.")).role("status"))
                    })
                },
                For(component_props_builder(&For)
                    .each(move || {
                        data.value
                            .get()
                            .map(|v| rows(&v, "items"))
                            .unwrap_or_default()
                    })
                    .key(|v| (num(v, "id"), num(v, "reply_count")))
                    .children(move |item| {
                        View::new(
                            blog_interface::html::div()
                                .child((
                                    CommentEntry(
                                        component_props_builder(&CommentEntry)
                                            .item(item.clone())
                                            .reply(choose)
                                            .build(),
                                    ),
                                    Replies(
                                        component_props_builder(&Replies)
                                            .root(item)
                                            .kind(kind)
                                            .choose(choose)
                                            .refresh(data.revision)
                                            .build(),
                                    ),
                                ))
                                .class("comment-floor"),
                        )
                    })
                    .build()),
                move || {
                    data.value
                        .get()
                        .filter(|v| rows(v, "items").is_empty())
                        .map(|_| {
                            View::new(
                                blog_interface::html::p()
                                    .child(t("还没有评论。", "No comments yet."))
                                    .class("muted"),
                            )
                        })
                },
                Pager(
                    component_props_builder(&Pager)
                        .data(data)
                        .page(page)
                        .public(true)
                        .preserve(true)
                        .locked(Signal::derive(move || busy.get()))
                        .label(t("评论分页", "Comment pages"))
                        .build(),
                ),
                blog_interface::html::form()
                    .child((
                        blog_interface::html::p()
                            .child(t("无需登录，以匿名访客身份发表评论。", "Post a comment as an anonymous guest. No account is needed."))
                            .class("comment-identity"),
                        move || {
                            reply.get().map(|v| {
                                View::new(
                                    blog_interface::html::button()
                                        .child(format!(
                                            "{}{} · {}",
                                            t("回复 ", "Reply to "),
                                            commenter(&v),
                                            t("取消", "Cancel")
                                        ))
                                        .class("text-action")
                                        .r#type("button")
                                        .disabled(move || busy.get() || pending.get().is_some())
                                        .on(blog_interface::ev::click, move |_| reply.set(None)),
                                )
                            })
                        },
                        blog_interface::html::textarea()
                            .disabled(move || {
                                busy.get() || pending.get().is_some() || storage_failure.get()
                            })
                            .aria_label(t("写下你的想法", "Write your thoughts"))
                            .required(true)
                            .maxlength("2048")
                            .prop("value", move || draft.get())
                            .on(blog_interface::ev::input, move |ev| {
                                draft.set(event_target_value(&ev))
                            })
                            .placeholder(t("写下你的想法", "Write your thoughts")),
                        blog_interface::html::button()
                            .child(move || {
                                if busy.get() {
                                    t("发表中…", "Posting…")
                                } else if pending.get().is_some() {
                                    t("确认原提交", "Confirm original submission")
                                } else {
                                    t("匿名发表评论", "Post anonymous comment")
                                }
                            })
                            .r#type("submit")
                            .disabled(move || {
                                busy.get()
                                    || storage_failure.get()
                                    || (pending.get().is_none() && draft.get().trim().is_empty())
                            }),
                        move || {
                            (pending.get().is_some() || storage_failure.get()).then(|| {
                                View::new(
                                    blog_interface::html::button()
                                        .child(t("清除待确认状态", "Clear pending submission"))
                                        .r#type("button")
                                        .disabled(move || busy.get())
                                        .on(blog_interface::ev::click, move |_| {
                                            if !web_sys::window().expect("invariant: the application is mounted in a browser window")
                                                .confirm_with_message(&t("这条评论可能已经发表。请先检查评论列表；确定放弃确认原提交并清空草稿？", "This comment may already be posted. Check the comments first. Stop confirming this submission and clear its draft?"))
                                                .unwrap_or(false)
                                            {
                                                return;
                                            }
                                            if clear_pending(&key.get_value()).is_err()
                                            {
                                                storage_failure.set(true);
                                                return;
                                            }
                                            pending.set(None);
                                            draft.set(String::new());
                                            reply.set(None);
                                            failure.set(String::new());
                                            storage_failure.set(false);
                                        }),
                                )
                            })
                        },
                        move || {
                            (!feedback.get().is_empty()).then(|| {
                                View::new(
                                    blog_interface::html::p()
                                        .child(feedback.get())
                                        .role("status"),
                                )
                            })
                        },
                    ))
                    .class("comment-form")
                    .on(blog_interface::ev::submit, move |ev| {
                        ev.prevent_default();
                        if busy.get_untracked() || storage_failure.get_untracked() {
                            return;
                        }
                        let (submission, retry) = match pending.get_untracked() {
                            Some(submission) => (submission, true),
                            None => {
                                let mut submission = json!({
                                    "request_id": web_sys::window().expect("invariant: the application is mounted in a browser window").crypto().expect("invariant: the application window exposes browser crypto").random_uuid(),
                                    "content": draft.get_untracked().trim(),
                                    "parent_comment_id": reply.get_untracked().map(|v| v["id"].clone()),
                                });
                                if kind == CommentKind::Article {
                                    submission["article_id"] = json!(article_id);
                                }
                                (submission, false)
                            }
                        };
                        if text(&submission, "content").is_empty() {
                            return;
                        }
                        if !retry && text(&submission, "content").chars().count() > 1024 {
                            failure.set(t(
                                "评论正文须为 1–1024 个字符",
                                "Comment content must contain 1–1024 characters",
                            ));
                            return;
                        }
                        if storage()
                            .and_then(|s| {
                                s.set_item(&key.get_value(), &submission.to_string())
                                    .map_err(PendingError::Browser)
                            })
                            .is_err()
                        {
                            storage_failure.set(true);
                            return;
                        }
                        pending.set(Some(submission.clone()));
                        busy.set(true);
                        failure.set(String::new());
                        feedback.set(String::new());
                        let c = web_sys::AbortController::new().expect("invariant: AbortController construction has no failure conditions");
                        let signal = c.signal();
                        controller.set_value(SendWrapper::new(Some(c)));
                        blog_interface::task::spawn_local(async move {
                            let result = request(
                                path,
                                "POST",
                                Some(submission.clone()),
                                RequestOptions {
                                    signal: Some(signal.clone()),
                                    ..Default::default()
                                },
                            )
                            .await;
                            if signal.aborted() {
                                return;
                            }
                            match result {
                                Ok(v)
                                    if text(&v, "comment_content")
                                        == text(&submission, "content").trim() =>
                                {
                                    storage_failure.set(clear_pending(&key.get_value()).is_err());
                                    pending.set(None);
                                    draft.set(String::new());
                                    reply.set(None);
                                    page.set(1);
                                    data.reload();
                                    feedback
                                        .set(t("评论已发表。", "Your comment has been posted."));
                                }
                                Err(e)
                                    if !retry
                                        && [400, 401, 403, 404, 409, 413, 429]
                                            .contains(&e.status)
                                        && e.code != blog_interface::api::ApiErrorCode::Local(blog_interface::api::LocalErrorCode::InvalidErrorResponse) =>
                                {
                                    storage_failure.set(clear_pending(&key.get_value()).is_err());
                                    pending.set(None);
                                    failure.set(public_error(
                                        &e,
                                        &t("请求失败，请重试。", "Request failed. Please retry."),
                                    ));
                                }
                                Ok(_) | Err(_) => failure.set(t("评论提交结果尚未确认。请确认原提交；重试会使用同一请求，不会重复发表。", "Your comment submission is not yet confirmed. Confirm the original submission; retrying uses the same request and will not post it twice.")),
                            }
                            busy.set(false);
                            controller.set_value(SendWrapper::new(None));
                        });
                    }),
            ))
            .class("article-comments"),
    )
}
#[component]
fn CommentEntry(item: Value, reply: Callback<Value>) -> impl IntoView {
    let target = item.clone();
    View::new(
        blog_interface::html::article()
            .child((
                blog_interface::html::div()
                    .child((
                        safe_url(&text(&item, "avatar"), true)
                            .map(|url| View::new(blog_interface::html::img().src(url).alt(""))),
                        blog_interface::html::div().child((
                            blog_interface::html::strong().child(commenter(&item)),
                            blog_interface::html::small()
                                .child(crate::dates::format(&text(&item, "create_time"))),
                        )),
                    ))
                    .class("public-comment-head"),
                blog_interface::html::p().child((
                    (!text(&item, "parent_username").is_empty()).then(|| {
                        View::new(
                            blog_interface::html::span()
                                .child(format!("@{} ", text(&item, "parent_username")))
                                .class("comment-mention"),
                        )
                    }),
                    text(&item, "comment_content"),
                )),
                blog_interface::html::div()
                    .child(
                        blog_interface::html::button()
                            .child(t("回复", "Reply"))
                            .r#type("button")
                            .on(blog_interface::ev::click, move |_| {
                                reply.run(target.clone())
                            }),
                    )
                    .class("comment-actions"),
            ))
            .class("public-comment"),
    )
}
#[component]
fn Replies(
    root: Value,
    kind: CommentKind,
    choose: Callback<Value>,
    refresh: RwSignal<u64>,
) -> impl IntoView {
    let id = num(&root, "id");
    let count = num(&root, "reply_count");
    let page = RwSignal::new(1);
    let path = Signal::derive(move || {
        refresh.get();
        if count == 0 {
            String::new()
        } else {
            format!(
                "{}/{id}/replies?page={}&size=5",
                kind.endpoint(),
                page.get()
            )
        }
    });
    let data = load(path);
    let items = RwSignal::new(Vec::<Value>::new());
    let expandable = Memo::new(move |_| {
        data.previous
            .get()
            .is_some_and(|value| num(&value, "total") > num(&value, "size"))
    });
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            let next = rows(&v, "items");
            items.update(|current| {
                if page.get_untracked() == 1 {
                    *current = next;
                } else {
                    for n in next {
                        if !current.iter().any(|v| v["id"] == n["id"]) {
                            current.push(n)
                        }
                    }
                }
            });
        }
    });
    View::new(
        blog_interface::html::div()
            .child((
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .public(true)
                        .retry(Some(Callback::new(move |()| data.reload())))
                        .build(),
                ),
                move || {
                    items
                        .get()
                        .into_iter()
                        .map(|item| {
                            View::new(CommentEntry(
                                component_props_builder(&CommentEntry)
                                    .item(item)
                                    .reply(choose)
                                    .build(),
                            ))
                        })
                        .collect_view()
                },
                move || {
                    expandable.get().then(|| {
                        View::new(
                            blog_interface::html::button()
                                .child(move || {
                                    let total =
                                        data.previous.get().map(|v| num(&v, "total")).unwrap_or(0);
                                    let remaining = (total - items.get().len() as i64).max(0);
                                    if remaining == 0 {
                                        t("已展开全部回复", "All replies shown")
                                    } else {
                                        format!(
                                            "{} {} {}",
                                            t("展开剩余", "Show"),
                                            remaining,
                                            t("条回复", "more replies")
                                        )
                                    }
                                })
                                .class("comment-more")
                                .r#type("button")
                                .aria_disabled(move || {
                                    (data.busy.get()
                                        || data.value.get().is_none()
                                        || data.previous.get().is_none_or(|v| {
                                            items.get().len() as i64 >= num(&v, "total")
                                        }))
                                    .to_string()
                                })
                                .on(blog_interface::ev::click, move |_| {
                                    if !data.busy.get_untracked()
                                        && data.value.get_untracked().is_some()
                                        && data.previous.get_untracked().is_some_and(|v| {
                                            (items.get_untracked().len() as i64) < num(&v, "total")
                                        })
                                    {
                                        page.update(|p| *p += 1)
                                    }
                                }),
                        )
                    })
                },
            ))
            .class("comment-replies"),
    )
}
#[path = "../../../test/browser/unit/comments/tests.rs"]
#[cfg(test)]
mod tests;
