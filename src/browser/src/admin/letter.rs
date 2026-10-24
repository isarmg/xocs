use crate::common::{Action, Check, Feedback, Field, href, load, t, text};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::Value;
use wasm_bindgen::JsCast;

#[component]
pub(super) fn LetterSettings() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/letter".into()));
    let form = RwSignal::new(Value::Null);
    let original = RwSignal::new(Value::Null);
    let save = Action::new();
    let form_node = NodeRef::<blog_interface::html::Form>::new();
    let validation = RwSignal::new(String::new());
    let ready = Memo::new(move |_| !form.get().is_null());
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            form.set(v.clone());
            original.set(v);
            validation.set(String::new());
        }
    });
    View::new(
        blog_interface::html::section()
            .child((
                blog_interface::html::h1().child(t("信笺管理", "Letter")),
                blog_interface::html::p().child(t(
                    "维护公开信笺的收信人、署名和正文。",
                    "Manage the recipient, sender, and content of the public letter.",
                )),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .retry(Some(Callback::new(move |()| data.reload())))
                        .build(),
                ),
                Feedback(component_props_builder(&Feedback).error(save.error).build()),
                move || {
                    (!validation.get().is_empty()).then(|| {
                        blog_interface::html::p()
                            .role("alert")
                            .child(validation.get())
                    })
                },
                move || {
                    ready.get().then(|| {
                        View::new(
                            blog_interface::html::form()
                                .node_ref(form_node)
                                .child(
                                    blog_interface::html::fieldset()
                                        .child((
                                            Field(
                                                component_props_builder(&Field)
                                                    .label(t("收信人", "Recipient"))
                                                    .key("recipient")
                                                    .form(form)
                                                    .required(true)
                                                    .max(100)
                                                    .build(),
                                            ),
                                            Field(
                                                component_props_builder(&Field)
                                                    .label(t("署名", "Sender"))
                                                    .key("sender")
                                                    .form(form)
                                                    .required(true)
                                                    .max(100)
                                                    .build(),
                                            ),
                                            Field(
                                                component_props_builder(&Field)
                                                    .label(t("信笺正文", "Letter content"))
                                                    .key("content")
                                                    .form(form)
                                                    .required(true)
                                                    .textarea(true)
                                                    .max(16384)
                                                    .build(),
                                            ),
                                            Check(
                                                component_props_builder(&Check)
                                                    .label(t(" 公开显示", " Show publicly"))
                                                    .key("is_public")
                                                    .form(form)
                                                    .build(),
                                            ),
                                            blog_interface::html::div()
                                                .child((
                                                    blog_interface::html::button()
                                                        .child(t("保存信笺", "Save letter"))
                                                        .class("blog-button")
                                                        .r#type("submit")
                                                        .disabled(move || save.busy.get()),
                                                    blog_interface::html::button()
                                                        .child(t("撤销修改", "Reset changes"))
                                                        .class("blog-button")
                                                        .r#type("button")
                                                        .disabled(move || {
                                                            save.busy.get()
                                                                || form.get() == original.get()
                                                        })
                                                        .on(blog_interface::ev::click, move |_| {
                                                            form.set(original.get_untracked());
                                                            save.error.set(None);
                                                            validation.set(String::new());
                                                        }),
                                                    blog_interface::html::a()
                                                        .child(t(
                                                            "查看公开页面",
                                                            "View public page",
                                                        ))
                                                        .href(href("/letter"))
                                                        .target("_blank")
                                                        .rel("noopener noreferrer"),
                                                ))
                                                .class("blog-actions"),
                                        ))
                                        .class("blog-fieldset")
                                        .disabled(move || save.busy.get()),
                                )
                                .class("editor-form")
                                .on(blog_interface::ev::input, move |_| {
                                    validation.set(String::new())
                                })
                                .on(blog_interface::ev::submit, move |ev| {
                                    ev.prevent_default();
                                    if save.busy.get_untracked() {
                                        return;
                                    }
                                    let f = form.get_untracked();
                                    let invalid = [
                                        ("recipient", 100, t("收信人", "Recipient")),
                                        ("sender", 100, t("署名", "Sender")),
                                        ("content", 16384, t("信笺正文", "Letter content")),
                                    ]
                                    .iter()
                                    .find(|(k, max, _)| {
                                        let n = text(&f, k).trim().chars().count();
                                        n == 0 || n > *max
                                    })
                                    .cloned();
                                    if let Some((key, max, label)) = invalid {
                                        validation.set(crate::common::tr(
                                            "{0}须包含 1–{1} 个字符。",
                                            "{0} must contain 1–{1} characters.",
                                            &[label, max.to_string()],
                                        ));
                                        if let Some(field) = form_node
                                            .get_untracked()
                                            .and_then(|node| {
                                                node.query_selector(&format!("[name={key}]"))
                                                    .ok()
                                                    .flatten()
                                            })
                                            .and_then(|node| {
                                                node.dyn_into::<web_sys::HtmlElement>().ok()
                                            })
                                        {
                                            let _ = field.focus();
                                        }
                                    } else {
                                        validation.set(String::new());
                                        save.run(
                                            "/api/v1/content/letter".into(),
                                            "PUT",
                                            Some(f),
                                            move |v| {
                                                form.set(v.clone());
                                                original.set(v);
                                                notify(t("信笺已保存", "Letter saved"));
                                            },
                                        );
                                    }
                                }),
                        )
                    })
                },
            ))
            .class("blog-content-stack"),
    )
}
