use crate::common::*;
use leptos::component::{component_props_builder, component_view};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde_json::{Value, json};
fn endpoint(kind: &str) -> &'static str {
    match kind {
        "message" => "/api/v1/message-comments",
        "love" => "/api/v1/love-comments",
        _ => "/api/v1/comments",
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
fn storage() -> Result<web_sys::Storage, ()> {
    web_sys::window()
        .unwrap()
        .session_storage()
        .map_err(|_| ())?
        .ok_or(())
}
fn read_pending(key: &str) -> Result<Option<Value>, ()> {
    let Some(raw) = storage()?.get_item(key).map_err(|_| ())? else {
        return Ok(None);
    };
    let v = serde_json::from_str(&raw).map_err(|_| ())?;
    if valid_submission(&v) {
        let mut normalized = json!({"request_id":v["request_id"],"content":v["content"],"parent_comment_id":v["parent_comment_id"]});
        if let Some(article) = v.get("article_id") {
            normalized["article_id"] = article.clone();
        }
        Ok(Some(normalized))
    } else {
        Err(())
    }
}
#[component]
pub fn Comments(
    #[prop(default = "article")] kind: &'static str,
    #[prop(default = 0)] article_id: i64,
    #[prop(optional)] title: String,
) -> impl IntoView {
    let path = endpoint(kind);
    let key = format!("xocs.comment-submission:{path}:{article_id}");
    let loaded = read_pending(&key);
    let pending = RwSignal::new(loaded.as_ref().ok().cloned().flatten());
    let storage_failure = RwSignal::new(loaded.is_err());
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
            if kind == "article" {
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
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    if title.is_empty() { t("评论", "Comments") } else { title }
                    })),
                    IntoMaybeErased::into_maybe_erased(" | "),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | format!("{} {}", data.previous.get().map(| v | num(& v,
                    "total")).unwrap_or(0), t("条评论", "comments")) })),))),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! failure.get().is_empty()).then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ failure.get() }) },
        )
        .role("alert"),
)) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .fallback(#[allow(unused_braces)] { t("评论加载失败",
                    "Unable to load comments") }).public(#[allow(unused_braces)] {
                    true }).retry(#[allow(unused_braces)] { Some(Callback::new(move |
                    () | data.reload())) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.busy.get().then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("评论加载中…", "Loading comments…")
                })
            },
        )
        .role("status"),
)) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | storage_failure.get().then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t(
                        "无法读取或保存待确认的提交。请先检查评论，再清除待确认状态；当前不会发送新评论。",
                        "The pending submission could not be read or saved. Check the comments before clearing it. No new comment will be sent.",
                    )
                })
            },
        )
        .role("alert"),
)) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (pending.get().is_some() && failure.get().is_empty())
                    .then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t(
                        "有一条评论等待确认，请继续确认原提交。",
                        "A comment is awaiting confirmation. Continue confirming the original submission.",
                    )
                })
            },
        )
        .role("status"),
)) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & For, { let mut props =
                    component_props_builder(& For)
                    .each(#[allow(unused_braces)] { move | | data.value.get().map(| v
                    | rows(& v, "items")).unwrap_or_default() })
                    .key(#[allow(unused_braces)] { | v | (num(v, "id"), num(v,
                    "reply_count")) }).children(#[allow(unused_braces)] { move | item
                    | View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & CommentEntry, { let mut props =
                    component_props_builder(& CommentEntry)
                    .item(#[allow(unused_braces)] { item.clone() })
                    .reply(#[allow(unused_braces)] { choose }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Replies, { let mut props =
                    component_props_builder(& Replies)
                    .root(#[allow(unused_braces)] { item })
                    .kind(#[allow(unused_braces)] { kind })
                    .choose(#[allow(unused_braces)] { choose })
                    .refresh(#[allow(unused_braces)] { data.revision }).build();
                    props }) })
                ,),
        )
        .class("comment-floor"),
) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.value.get().filter(| v | rows(v, "items")
                    .is_empty()).map(| _ | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("还没有评论。", "No comments yet.")
                })
            },
        )
        .class("muted"),
)) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Pager, { let mut props =
                    component_props_builder(& Pager)
                    .data(#[allow(unused_braces)] { data })
                    .page(#[allow(unused_braces)] { page })
                    .public(#[allow(unused_braces)] { true })
                    .preserve(#[allow(unused_braces)] { true })
                    .locked(#[allow(unused_braces)] { Signal::derive(move | | busy
                    .get()) }).label(#[allow(unused_braces)] { t("评论分页",
                    "Comment pages") }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::form()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("无需登录，以匿名访客身份发表评论。",
                    "Post a comment as an anonymous guest. No account is needed.") })
                    }).class("comment-identity")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | reply.get().map(| v | View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    format!(
                        "{}{} · {}", t("回复 ", "Reply to "), commenter(& v),
                        t("取消", "Cancel")
                    )
                })
            },
        )
        .class("text-action")
        .r#type("button")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                move || {
                    busy.get() || pending.get().is_some()
                },
            ),
        )
        .on(leptos::ev::click, move |_| reply.set(None)),
)) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::textarea()
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | busy.get() || pending.get().is_some() || storage_failure
                    .get()))
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("写下你的想法",
                    "Write your thoughts"))).required(true).maxlength("1024")
                    .prop("value", move | | draft
                    .get()).on(leptos::ev::input, move | ev |
                    draft.set(event_target_value(& ev)))
                    .placeholder(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("写下你的想法",
                    "Write your thoughts")))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if busy.get() { t("发表中…", "Posting…") } else if pending.get().is_some() { t("确认原提交",
                    "Confirm original submission") } else { t("匿名发表评论",
                    "Post anonymous comment") } }) }).r#type("submit")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | busy.get() || storage_failure.get() || (pending.get()
                    .is_none() && draft.get().trim().is_empty())))),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (pending.get().is_some() || storage_failure.get())
                    .then(| | View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("清除待确认状态", "Clear pending submission")
                })
            },
        )
        .r#type("button")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                move || busy.get(),
            ),
        )
        .on(
            leptos::ev::click,
            move |_| {
                if !web_sys::window()
                    .unwrap()
                    .confirm_with_message(
                        &t(
                            "这条评论可能已经发表。请先检查评论列表；确定放弃确认原提交并清空草稿？",
                            "This comment may already be posted. Check the comments first. Stop confirming this submission and clear its draft?",
                        ),
                    )
                    .unwrap_or(false)
                {
                    return;
                }
                if storage()
                    .and_then(|s| s.remove_item(&key.get_value()).map_err(|_| ()))
                    .is_err()
                {
                    storage_failure.set(true);
                    return;
                }
                pending.set(None);
                draft.set(String::new());
                reply.set(None);
                failure.set(String::new());
                storage_failure.set(false);
            },
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! feedback.get().is_empty()).then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ feedback.get() }) },
        )
        .role("status"),
)) })),))
                    .class("comment-form").on(leptos::ev::submit,
                    move | ev | { ev.prevent_default(); if busy.get_untracked() ||
                    storage_failure.get_untracked() { return; } let retry = pending
                    .get_untracked().is_some(); let submission = pending
                    .get_untracked().unwrap_or_else(| | { let mut v = json!({
                    "request_id" : web_sys::window().unwrap().crypto().unwrap()
                    .random_uuid(), "content" : draft.get_untracked().trim(),
                    "parent_comment_id" : reply.get_untracked().map(| v | v["id"]
                    .clone()).unwrap_or(Value::Null) }); if kind == "article" {
                    v["article_id"] = json!(article_id); } v }); if text(&
                    submission, "content").is_empty() { return; }
                    if storage()
                    .and_then(| s | s.set_item(& key.get_value(), & submission
                    .to_string()).map_err(| _ | ())).is_err() { storage_failure
                    .set(true); return; } pending.set(Some(submission.clone())); busy
                    .set(true); failure.set(String::new()); feedback
                    .set(String::new()); let c = web_sys::AbortController::new()
                    .unwrap(); let signal = c.signal(); controller
                    .set_value(SendWrapper::new(Some(c)));
                    leptos::task::spawn_local(async move { let result = request(path,
                    "POST", Some(submission.clone()), RequestOptions { signal :
                    Some(signal.clone()), ..Default::default() }). await; if signal
                    .aborted() { return; } match result { Ok(v) if text(& v,
                    "comment_content") == text(& submission, "content").trim() => {
                    let _ = storage().and_then(| s | s.remove_item(& key.get_value())
                    .map_err(| _ | ())); pending.set(None); draft.set(String::new());
                    reply.set(None); page.set(1); data.reload(); feedback
                    .set(t("评论已发表。", "Your comment has been posted."));
                    }, Err(e) if ! retry && [400, 401, 403, 404, 409, 413, 429]
                    .contains(& e.status) && e.code != "invalid_error_response" => {
                    let _ = storage().and_then(| s | s.remove_item(& key.get_value())
                    .map_err(| _ | ())); pending.set(None); failure
                    .set(public_error(& e, & t("请求失败，请重试。",
                    "Request failed. Please retry."))); }, _ => failure
                    .set(t("评论提交结果尚未确认。请确认原提交；重试会使用同一请求，不会重复发表。",
                    "Your comment submission is not yet confirmed. Confirm the original submission; retrying uses the same request and will not post it twice."))
                    } busy.set(false); controller.set_value(SendWrapper::new(None));
                    }); }))
                ,),
        )
        .class("article-comments"),
)
}
#[component]
fn CommentEntry(item: Value, reply: Callback<Value>) -> impl IntoView {
    let target = item.clone();
    View::new(
        leptos::html::article()
            .child((
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child((
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                safe_url(&text(&item, "avatar"), true).map(|url| {
                                    View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(url))
        .alt(""),
)
                                })
                            })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div().child((
                                IntoMaybeErased::into_maybe_erased(leptos::html::strong().child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({ commenter(&item) })
                                    },
                                )),
                                IntoMaybeErased::into_maybe_erased(leptos::html::small().child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({
                                            crate::dates::format(&text(&item, "create_time"))
                                        })
                                    },
                                )),
                            ))),
                        ))
                        .class("public-comment-head"),
                ),
                IntoMaybeErased::into_maybe_erased(leptos::html::p().child((
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                        (!text(&item, "parent_username").is_empty()).then(|| {
                            View::new(
                                leptos::html::span()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                format!("@{} ", text(&item, "parent_username"))
                                            })
                                        },
                                    )
                                    .class("comment-mention"),
                            )
                        })
                    })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                        text(&item, "comment_content")
                    })),
                ))),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child(
                            #[allow(unused_braces)]
                            {
                                leptos::html::button()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({ t("回复", "Reply") })
                                        },
                                    )
                                    .r#type("button")
                                    .on(leptos::ev::click, move |_| reply.run(target.clone()))
                            },
                        )
                        .class("comment-actions"),
                ),
            ))
            .class("public-comment"),
    )
}
#[component]
fn Replies(
    root: Value,
    kind: &'static str,
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
            format!("{}/{id}/replies?page={}&size=5", endpoint(kind), page.get())
        }
    });
    let data = load(path);
    let items = RwSignal::new(Vec::<Value>::new());
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
        leptos::html::div()
            .child((
                IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)]
                    #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Feedback,
                        {
                            let mut props = component_props_builder(&Feedback)
                                .error(
                                    #[allow(unused_braces)]
                                    {
                                        data.error
                                    },
                                )
                                .public(
                                    #[allow(unused_braces)]
                                    {
                                        true
                                    },
                                )
                                .retry(
                                    #[allow(unused_braces)]
                                    {
                                        Some(Callback::new(move |()| data.reload()))
                                    },
                                )
                                .build();
                            props
                        },
                    )
                }),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move || {
                        items
                            .get()
                            .into_iter()
                            .map(|item| {
                                View::new({
                                    #[allow(unreachable_code)]
                                    #[allow(unused_mut)]
                                    #[allow(
                                        clippy::let_and_return,
                                        clippy::unit_arg,
                                        clippy::let_unit_value
                                    )]
                                    component_view(
                                        #[allow(clippy::needless_borrows_for_generic_args)]
                                        &CommentEntry,
                                        {
                                            let mut props = component_props_builder(&CommentEntry)
                                                .item(
                                                    #[allow(unused_braces)]
                                                    {
                                                        item
                                                    },
                                                )
                                                .reply(
                                                    #[allow(unused_braces)]
                                                    {
                                                        choose
                                                    },
                                                )
                                                .build();
                                            props
                                        },
                                    )
                                })
                            })
                            .collect_view()
                    }
                })),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move || {
                        data.value
                            .get()
                            .filter(|v| (items.get().len() as i64) < num(v, "total"))
                            .map(|v| {
                                View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    format!(
                        "{} {} {}", t("展开剩余", "Show"), num(& v, "total") -
                        items.get().len() as i64, t("条回复", "more replies")
                    )
                })
            },
        )
        .class("comment-more")
        .r#type("button")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                move || data.busy.get(),
            ),
        )
        .on(leptos::ev::click, move |_| page.update(|p| *p += 1)),
)
                            })
                    }
                })),
            ))
            .class("comment-replies"),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_identity_and_unicode_trim() {
        assert!(valid_submission(
            &json!({"request_id":"12345678-1234-4123-8123-123456789abc","content":"hello","parent_comment_id":null})
        ));
        assert!(!valid_submission(
            &json!({"request_id":"12345678-1234-7123-8123-123456789abc","content":"hello","parent_comment_id":null})
        ));
        assert_eq!("\u{85} text \u{85}".trim(), "text");
    }
}
