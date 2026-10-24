//! Shared form inputs and file selection; mutation ownership stays with the page.
use super::{flag, t, text, tr};
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use serde_json::Value;

fn scalar_error(value: &str, max: u32) -> String {
    if value.chars().count() > max as usize {
        tr(
            "最多可输入 {0} 个字符。",
            "Use no more than {0} characters.",
            &[max.to_string()],
        )
    } else {
        String::new()
    }
}

#[component]
pub(crate) fn Field(
    label: String,
    key: &'static str,
    form: RwSignal<Value>,
    #[prop(default = false)] textarea: bool,
    #[prop(default = false)] required: bool,
    /// Maximum Unicode scalar count; native maxlength uses a UTF-16 budget.
    #[prop(default = 32768)]
    max: u32,
) -> impl IntoView {
    let input = NodeRef::<blog_interface::html::Input>::new();
    let area = NodeRef::<blog_interface::html::Textarea>::new();
    let error = Memo::new(move |_| scalar_error(&text(&form.get(), key), max));
    Effect::new(move |_| {
        let message = error.get();
        if let Some(input) = input.get() {
            input.set_custom_validity(&message);
        }
        if let Some(area) = area.get() {
            area.set_custom_validity(&message);
        }
    });
    View::new(
        blog_interface::html::label()
            .child((
                blog_interface::html::span()
                    .child(label.clone())
                    .class("blog-field-label blog-row"),
                if textarea {
                    View::new(
                        blog_interface::html::textarea()
                            .node_ref(area)
                            .aria_label(label.clone())
                            .class("blog-input")
                            .name(key)
                            .required(required)
                            .maxlength(max.saturating_mul(2))
                            .aria_invalid(move || (!error.get().is_empty()).to_string())
                            .prop("value", move || text(&form.get(), key))
                            .on(blog_interface::ev::change, move |ev| {
                                let node = event_target::<web_sys::HtmlTextAreaElement>(&ev);
                                node.set_custom_validity(&scalar_error(&node.value(), max));
                            })
                            .on(blog_interface::ev::input, move |ev| {
                                let value = event_target_value(&ev);
                                event_target::<web_sys::HtmlTextAreaElement>(&ev)
                                    .set_custom_validity(&scalar_error(&value, max));
                                form.update(|f| f[key] = Value::String(value))
                            }),
                    )
                    .into_any()
                } else {
                    View::new(
                        blog_interface::html::input()
                            .node_ref(input)
                            .aria_label(label.clone())
                            .class("blog-input")
                            .name(key)
                            .r#type("text")
                            .required(required)
                            .maxlength(max.saturating_mul(2))
                            .aria_invalid(move || (!error.get().is_empty()).to_string())
                            .prop("value", move || text(&form.get(), key))
                            .on(blog_interface::ev::change, move |ev| {
                                let node = event_target::<web_sys::HtmlInputElement>(&ev);
                                node.set_custom_validity(&scalar_error(&node.value(), max));
                            })
                            .on(blog_interface::ev::input, move |ev| {
                                let value = event_target_value(&ev);
                                event_target::<web_sys::HtmlInputElement>(&ev)
                                    .set_custom_validity(&scalar_error(&value, max));
                                form.update(|f| f[key] = Value::String(value))
                            }),
                    )
                    .into_any()
                },
                move || {
                    (!error.get().is_empty()).then(|| {
                        blog_interface::html::span()
                            .role("alert")
                            .class("blog-row blog-danger")
                            .child(error.get())
                    })
                },
            ))
            .class("blog-form-field blog-stack"),
    )
}

#[component]
pub(crate) fn ControlledField(
    label: String,
    value: Signal<String>,
    change: Callback<String>,
    #[prop(default = false)] required: bool,
    /// Maximum Unicode scalar count; native maxlength uses a UTF-16 budget.
    #[prop(default = 32768)]
    max: u32,
) -> impl IntoView {
    let input = NodeRef::<blog_interface::html::Input>::new();
    let error = Memo::new(move |_| scalar_error(&value.get(), max));
    Effect::new(move |_| {
        let message = error.get();
        if let Some(input) = input.get() {
            input.set_custom_validity(&message);
        }
    });
    View::new(
        blog_interface::html::label()
            .child((
                blog_interface::html::span()
                    .child(label.clone())
                    .class("blog-field-label blog-row"),
                blog_interface::html::input()
                    .node_ref(input)
                    .aria_label(label)
                    .class("blog-input")
                    .r#type("text")
                    .required(required)
                    .maxlength(max.saturating_mul(2))
                    .aria_invalid(move || (!error.get().is_empty()).to_string())
                    .prop("value", move || value.get())
                    .on(blog_interface::ev::change, move |ev| {
                        let node = event_target::<web_sys::HtmlInputElement>(&ev);
                        node.set_custom_validity(&scalar_error(&node.value(), max));
                    })
                    .on(blog_interface::ev::input, move |ev| {
                        let value = event_target_value(&ev);
                        event_target::<web_sys::HtmlInputElement>(&ev)
                            .set_custom_validity(&scalar_error(&value, max));
                        change.run(value)
                    }),
                move || {
                    (!error.get().is_empty()).then(|| {
                        blog_interface::html::span()
                            .role("alert")
                            .class("blog-row blog-danger")
                            .child(error.get())
                    })
                },
            ))
            .class("blog-form-field blog-stack"),
    )
}

#[component]
pub(crate) fn ControlledCheck(
    label: String,
    value: Signal<bool>,
    change: Callback<bool>,
) -> impl IntoView {
    View::new(
        blog_interface::html::label()
            .class("blog-row blog-flow blog-check")
            .child((
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
        blog_interface::html::label()
            .class("blog-row blog-flow blog-check")
            .child((
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
                    .class("blog-row")
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
                            .class("blog-file-picker-name blog-row")
                            .id(format!("{id}-filename"))
                            .aria_live("polite"),
                    ))
                    .class("blog-file-picker blog-flow"),
            ))
            .class("blog-form-field blog-stack"),
    )
}
