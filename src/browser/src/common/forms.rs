//! Shared form inputs and file selection; mutation ownership stays with the page.
use super::{flag, t, text};
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use serde_json::Value;

#[component]
pub(crate) fn Field(
    label: String,
    key: &'static str,
    form: RwSignal<Value>,
    #[prop(default = false)] textarea: bool,
    #[prop(default = false)] required: bool,
    #[prop(default = "text")] kind: &'static str,
    #[prop(default = 32768)] max: u32,
) -> impl IntoView {
    View::new(
        blog_interface::html::label()
            .child((
                blog_interface::html::span()
                    .child(label)
                    .class("blog-field-label"),
                if textarea {
                    View::new(
                        blog_interface::html::textarea()
                            .class("blog-input")
                            .required(required)
                            .maxlength(max)
                            .prop("value", move || text(&form.get(), key))
                            .on(blog_interface::ev::input, move |ev| {
                                form.update(|f| f[key] = Value::String(event_target_value(&ev)))
                            }),
                    )
                    .into_any()
                } else {
                    View::new(
                        blog_interface::html::input()
                            .class("blog-input")
                            .r#type(kind)
                            .required(required)
                            .maxlength(max)
                            .prop("value", move || {
                                let f = form.get();
                                if kind == "number" {
                                    f[key].as_i64().map(|v| v.to_string()).unwrap_or_default()
                                } else {
                                    text(&f, key)
                                }
                            })
                            .on(blog_interface::ev::input, move |ev| {
                                let v = event_target_value(&ev);
                                form.update(|f| {
                                    f[key] = if kind == "number" {
                                        Value::from(v.parse::<i64>().unwrap_or(0))
                                    } else {
                                        Value::String(v)
                                    };
                                })
                            }),
                    )
                    .into_any()
                },
            ))
            .class("blog-form-field"),
    )
}

#[component]
pub(crate) fn ControlledField(
    label: String,
    value: Signal<String>,
    change: Callback<String>,
    #[prop(default = false)] required: bool,
    #[prop(default = 32768)] max: u32,
) -> impl IntoView {
    View::new(
        blog_interface::html::label()
            .child((
                blog_interface::html::span()
                    .child(label)
                    .class("blog-field-label"),
                blog_interface::html::input()
                    .class("blog-input")
                    .r#type("text")
                    .required(required)
                    .maxlength(max)
                    .prop("value", move || value.get())
                    .on(blog_interface::ev::input, move |ev| {
                        change.run(event_target_value(&ev))
                    }),
            ))
            .class("blog-form-field"),
    )
}

#[component]
pub(crate) fn ControlledCheck(
    label: String,
    value: Signal<bool>,
    change: Callback<bool>,
) -> impl IntoView {
    View::new(
        blog_interface::html::label().child((
            blog_interface::html::input()
                .r#type("checkbox")
                .prop("checked", move || value.get())
                .on(blog_interface::ev::change, move |ev| {
                    change.run(event_target_checked(&ev))
                }),
            label,
        )),
    )
}

#[component]
pub(crate) fn Check(label: String, key: &'static str, form: RwSignal<Value>) -> impl IntoView {
    View::new(
        blog_interface::html::label().child((
            blog_interface::html::input()
                .r#type("checkbox")
                .prop("checked", move || flag(&form.get(), key))
                .on(blog_interface::ev::change, move |ev| {
                    form.update(|f| f[key] = Value::Bool(event_target_checked(&ev)))
                }),
            label,
        )),
    )
}

#[component]
pub(crate) fn FilePicker(
    label: String,
    choose: Callback<SendWrapper<web_sys::File>>,
    #[prop(default = "image/png,image/jpeg,image/gif,image/webp".into())] accept: String,
    #[prop(default=Signal::derive(||false))] disabled: Signal<bool>,
    #[prop(default=Signal::derive(String::new))] filename: Signal<String>,
) -> impl IntoView {
    let input = NodeRef::<blog_interface::html::Input>::new();
    let id = format!(
        "blog-file-{}",
        web_sys::window().unwrap().crypto().unwrap().random_uuid()
    );
    View::new(
        blog_interface::html::div()
            .child((
                blog_interface::html::label()
                    .child(label.clone())
                    .r#for(id.clone()),
                blog_interface::html::div()
                    .child((
                        blog_interface::html::input()
                            .id(id.clone())
                            .node_ref(input)
                            .r#type("file")
                            .hidden(true)
                            .disabled(move || disabled.get())
                            .accept(accept)
                            .on(blog_interface::ev::change, move |ev| {
                                let node = event_target::<web_sys::HtmlInputElement>(&ev);
                                let file = node.files().and_then(|f| f.get(0));
                                node.set_value("");
                                if let Some(file) = file {
                                    choose.run(SendWrapper::new(file));
                                }
                            }),
                        blog_interface::html::button()
                            .child(t("选择文件", "Choose file"))
                            .class("blog-button")
                            .r#type("button")
                            .disabled(move || disabled.get())
                            .aria_label(format!("{}{}", t("选择文件：", "Choose file: "), label))
                            .aria_describedby(format!("{id}-filename"))
                            .on(blog_interface::ev::click, move |_| {
                                if let Some(input) = input.get() {
                                    input.click();
                                }
                            }),
                        blog_interface::html::span()
                            .child(move || {
                                let f = filename.get();
                                if f.is_empty() {
                                    t("未选择文件", "No file chosen")
                                } else {
                                    f
                                }
                            })
                            .class("blog-file-picker-name")
                            .id(format!("{id}-filename"))
                            .aria_live("polite"),
                    ))
                    .class("blog-file-picker"),
            ))
            .class("blog-form-field"),
    )
}
