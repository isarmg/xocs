use crate::common::{Action, Check, Feedback, Field, href, load, t, text};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::Value;

#[component]
pub(super) fn LetterSettings() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/letter".into()));
    let form = RwSignal::new(Value::Null);
    let original = RwSignal::new(Value::Null);
    let save = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            form.set(v.clone());
            original.set(v);
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
                    (!form.get().is_null()).then(|| {
                        View::new(
                            blog_interface::html::form()
                                .child(
                                    blog_interface::html::fieldset()
                                        .child((
                                            Field(
                                                component_props_builder(&Field)
                                                    .label(t("收信人", "Recipient"))
                                                    .key("recipient")
                                                    .form(form)
                                                    .required(true)
                                                    .max(200)
                                                    .build(),
                                            ),
                                            Field(
                                                component_props_builder(&Field)
                                                    .label(t("署名", "Sender"))
                                                    .key("sender")
                                                    .form(form)
                                                    .required(true)
                                                    .max(200)
                                                    .build(),
                                            ),
                                            Field(
                                                component_props_builder(&Field)
                                                    .label(t("信笺正文", "Letter content"))
                                                    .key("content")
                                                    .form(form)
                                                    .required(true)
                                                    .textarea(true)
                                                    .max(32768)
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
                                .on(blog_interface::ev::submit, move |ev| {
                                    ev.prevent_default();
                                    let f = form.get_untracked();
                                    let valid =
                                        [("recipient", 100), ("sender", 100), ("content", 16384)]
                                            .iter()
                                            .all(|(k, max)| {
                                                let n = text(&f, k).trim().chars().count();
                                                n >= 1 && n <= *max
                                            });
                                    if valid {
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
