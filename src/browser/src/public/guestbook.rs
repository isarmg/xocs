//! Anonymous guestbook posting and message display.
use super::appearance::{background_style, site_media_url};
use super::shell::{Chrome, site_data};
use crate::comments::{CommentKind, Comments};
use crate::common::{Action, Feedback, load, safe_url, t, text};
use blog_interface::children::ToChildren;
use blog_interface::component::component_props_builder;
use blog_interface::ev;
use blog_interface::html;
use blog_interface::prelude::*;
use serde_json::{Value, json};

#[component]
pub(super) fn Message() -> impl IntoView {
    let site = site_data();
    let data = load(Signal::derive(|| "/api/v1/tree-hole".into()));
    let draft = RwSignal::new(String::new());
    let posts = RwSignal::new(Vec::<Value>::new());
    let action = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            posts.update(|current| {
                for item in v.as_array().cloned().unwrap_or_default() {
                    if !current.iter().any(|v| v["id"] == item["id"]) {
                        current.push(item)
                    }
                }
            });
        }
    });
    View::new(Chrome(component_props_builder(&Chrome)
                .plain( true)
                .title(

                    Signal::derive(|| t("留言板", "Message board")),
                )
                .children(ToChildren::to_children(move || html::div()
                            .child(
                                (
                                        html::section()
                                        .child((html::div()
                                        .child( move | | posts.get().into_iter().enumerate().map(| (i, item)
                                        | View::new(
    html::div()
        .child(
            (
                    safe_url(& text(& item, "avatar"), false).map(| u | View::new(
    html::img()
        .src(u)
        .alt(""),
)),
                    html::span()
                    .child( text(& item,
                    "message"))
                ,),
        )
        .style(
            format!(
                "animation-delay:{}s;top:{}%",- ((i % 10) as f64) * 2.3, 8 + i % 7 *
                12
            ),
        )
        .class("message-barrage-item"),
))
                                        .collect_view()).class("message-barrage")
                                        .aria_label(t("留言弹幕",
                                        "Message wall")),
                                        html::div()
                                        .child((html::h1()
                                        .child( t("弹幕",
                                        "Message wall")),
                                        html::form()
                                        .child((html::input()
                                        .aria_label(t("发送弹幕",
                                        "Send a message")).maxlength("60").prop("value",
                                        move | | draft.get())
                                        .on(ev::input, move | ev | draft
                                        .set(event_target_value(& ev)))
                                        .placeholder(t("留下点什么啦～",
                                        "Leave a message…")),
                                        html::button()
                                        .child( move | | if action.busy.get() { t("发射中…",
                                        "Sending…") } else { t("发射", "Send") })
                                        .disabled(move
                                        | | action.busy.get() || draft.get().trim()
                                        .is_empty()),)).on(ev::submit,
                                        move | ev | { ev.prevent_default(); let content = draft
                                        .get_untracked(); if content.trim().is_empty() { return; }
                                        action.run("/api/v1/tree-hole/guest".into(), "POST",
                                        Some(json!({ "message" : content.trim() })), move | v | {
                                        posts.update(| items | { items.retain(| i | i["id"] !=
                                        v["id"]); items.insert(0, v) }); if draft.get_untracked() ==
                                        content { draft.set(String::new()) } }); }),
                                        html::p()
                                        .child( t("弹幕将匿名发布，无需登录。",
                                        "Messages are posted anonymously. No account is needed.")).class("message-anonymous"),
                                        Feedback(component_props_builder(& Feedback)
                                        .error( data.error)
                                        .public( true)
                                        .retry( Some(Callback::new(move |
                                        () | data.reload()))).build()),
                                        Feedback(component_props_builder(& Feedback)
                                        .error( action.error)
                                        .public( true).build()),)).class("message-form-wrap"),))
                                        .style(move || background_style(site_media_url(site, |site| site.media.guestbook_background.as_deref())))
                                        .class("message-hero"),
                                        html::div()
                                        .child( Comments(component_props_builder(& Comments)
                                        .kind(CommentKind::Message).build())).class("message-comment-wrap")
                                    ,),
                            )
                            .class("message-page")))
                .build()))
}
