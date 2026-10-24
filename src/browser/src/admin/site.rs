use crate::common::{
    Action, ControlledField, Feedback, Field, FilePicker, load, safe_url, t, text,
};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::{Value, json};
use wasm_bindgen::JsCast;

fn media_field_value(form: &Value, key: &str, nested: bool) -> String {
    text(if nested { &form["media"] } else { form }, key)
}

fn set_media_field(form: &mut Value, key: &str, nested: bool, value: Option<String>) {
    if nested {
        if !form["media"].is_object() {
            form["media"] = json!({});
        }
        form["media"][key] = value.map_or(Value::Null, Value::String);
    } else {
        form[key] = value.map_or(Value::Null, Value::String);
    }
}

#[component]
fn SiteMediaField(
    label: String,
    key: &'static str,
    form: RwSignal<Value>,
    upload: Action,
    save_busy: RwSignal<bool>,
    #[prop(default = true)] nested: bool,
    #[prop(default = false)] video: bool,
) -> impl IntoView {
    let value = Signal::derive(move || media_field_value(&form.get(), key, nested));
    let disabled = Signal::derive(move || upload.busy.get() || save_busy.get());
    let filename = RwSignal::new(String::new());
    Effect::new(move |_| {
        if !upload.busy.get() {
            filename.set(String::new());
        }
    });
    let preview_label = format!("{}{}", label, t("预览", " preview"));
    let clear_label = format!("{}{}", t("清除：", "Clear "), label);
    let picker_label = format!("{}{}", t("上传：", "Upload "), label);
    View::new(
        blog_interface::html::section()
            .class("site-media-field")
            .child((
                ControlledField(
                    component_props_builder(&ControlledField)
                        .label(label)
                        .value(value)
                        .max(2048)
                        .change(Callback::new(move |next: String| {
                            if !disabled.get_untracked() {
                                form.update(|form| set_media_field(form, key, nested, Some(next)));
                            }
                        }))
                        .build(),
                ),
                FilePicker(
                    component_props_builder(&FilePicker)
                        .label(picker_label)
                        .disabled(disabled)
                        .filename(Signal::derive(move || filename.get()))
                        .accept(
                            if video {
                                "video/mp4,video/webm"
                            } else {
                                "image/png,image/jpeg,image/gif,image/webp"
                            }
                            .into(),
                        )
                        .choose(Callback::new(
                            move |file: blog_interface::thread_bound::SendWrapper<
                                web_sys::File,
                            >| {
                                if disabled.get_untracked() {
                                    return;
                                }
                                let previous = value.get_untracked();
                                let name = file.name();
                                let done = move |response: Value| {
                                    let path = text(&response, "path");
                                    form.update(|form| {
                                        if media_field_value(form, key, nested) == previous {
                                            set_media_field(form, key, nested, Some(path));
                                        }
                                    });
                                    filename.set(String::new());
                                };
                                if video {
                                    upload.upload_video(file, done);
                                } else {
                                    upload.upload(file, done);
                                }
                                if upload.busy.get_untracked() {
                                    filename.set(name);
                                }
                            },
                        ))
                        .build(),
                ),
                blog_interface::html::button()
                    .class("blog-button")
                    .r#type("button")
                    .child(t("清除", "Clear"))
                    .aria_label(clear_label)
                    .disabled(move || disabled.get() || value.get().is_empty())
                    .on(blog_interface::ev::click, move |_| {
                        if !disabled.get_untracked() {
                            form.update(|form| set_media_field(form, key, nested, None));
                            filename.set(String::new());
                        }
                    }),
                move || {
                    safe_url(&value.get(), false).map(|url| {
                        if video {
                            View::new(
                                blog_interface::html::video()
                                    .class("site-media-preview")
                                    .src(url)
                                    .controls(true)
                                    .preload("metadata")
                                    .aria_label(preview_label.clone()),
                            )
                            .into_any()
                        } else {
                            View::new(
                                blog_interface::html::img()
                                    .class("site-media-preview")
                                    .src(url)
                                    .alt(preview_label.clone())
                                    .loading("lazy"),
                            )
                            .into_any()
                        }
                    })
                },
            )),
    )
}

#[component]
fn SiteAppearance(
    form: RwSignal<Value>,
    upload: Action,
    save_busy: RwSignal<bool>,
) -> impl IntoView {
    let fields = [
        (
            "background_image",
            "背景图片",
            "Background image",
            false,
            false,
        ),
        ("avatar", "站长头像", "Site owner avatar", false, false),
        ("logo", "网站标识", "Site logo", true, false),
        (
            "journey_background",
            "时间轴背景",
            "Journey background",
            true,
            false,
        ),
        (
            "journal_background",
            "随笔背景",
            "Journal background",
            true,
            false,
        ),
        (
            "guestbook_background",
            "留言板背景",
            "Message board background",
            true,
            false,
        ),
        (
            "love_letter_background",
            "恋爱信笺背景",
            "Love letter background",
            true,
            false,
        ),
        (
            "love_moments_cover",
            "点点滴滴封面",
            "Love moments cover",
            true,
            false,
        ),
        (
            "love_photos_cover",
            "时光相册封面",
            "Love photos cover",
            true,
            false,
        ),
        (
            "love_wishes_cover",
            "祝福墙封面",
            "Love wishes cover",
            true,
            false,
        ),
        (
            "toolbox_background",
            "百宝箱背景图片",
            "Toolbox background image",
            true,
            false,
        ),
        (
            "toolbox_video",
            "百宝箱背景视频",
            "Toolbox background video",
            true,
            true,
        ),
    ];
    View::new(
        blog_interface::html::section()
            .class("blog-content-panel")
            .child((
                blog_interface::html::h2().class("blog-row").child(t("外观媒体", "Appearance media")),
                blog_interface::html::p()
                    .class("blog-field-help blog-row")
                    .child(t("填写图片或视频地址，或直接上传文件。留空时使用主题配色或文字，不加载默认媒体。恋爱页头图和双方头像在恋爱笔记中设置。图片最大 10 MB，MP4 / WebM 视频最大 50 MB。", "Enter an image or video URL, or upload a file. Leave empty to use theme colors or text, without bundled media. Set the love journal header and couple avatars in Love journal. Images: up to 10 MB; MP4 / WebM videos: up to 50 MB.")),
                blog_interface::html::div()
                    .class("form-grid site-media-grid")
                    .child(
                        fields
                            .into_iter()
                            .map(move |(key, chinese, english, nested, video)| {
                                SiteMediaField(
                                    component_props_builder(&SiteMediaField)
                                        .label(t(chinese, english))
                                        .key(key)
                                        .form(form)
                                        .upload(upload)
                                        .save_busy(save_busy)
                                        .nested(nested)
                                        .video(video)
                                        .build(),
                                )
                            })
                            .collect_view(),
                    ),
                blog_interface::html::p().class("blog-row")
                    .role("status")
                    .aria_live("polite")
                    .child(move || {
                        if upload.busy.get() {
                            t("正在上传媒体…", "Uploading media…")
                        } else {
                            String::new()
                        }
                    }),
            )),
    )
}

#[component]
pub(super) fn SiteSettings() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/site".into()));
    let form = RwSignal::new(Value::Null);
    let save = Action::new();
    let upload = Action::new();
    let form_node = NodeRef::<blog_interface::html::Form>::new();
    let validation = RwSignal::new(String::new());
    let ready = Memo::new(move |_| !form.get().is_null());
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            upload.abort();
            validation.set(String::new());
            form.set(v);
        }
    });
    View::new(
        blog_interface::html::section()
            .child((
                blog_interface::html::h1().class("blog-row").child(t("网站设置", "Site settings")),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .retry(Some(Callback::new(move |()| {
                            upload.abort();
                            data.reload();
                        })))
                        .build(),
                ),
                Feedback(component_props_builder(&Feedback).error(save.error).build()),
                move || {
                    (!validation.get().is_empty()).then(|| {
                        blog_interface::html::p().class("blog-row").role("alert").child(validation.get())
                    })
                },
                move || {
                    ready.get().then(|| {
                        View::new(
                            blog_interface::html::form()
                                .node_ref(form_node)
                                .child((
                                    blog_interface::html::fieldset()
                                        .child((
                                            blog_interface::html::section()
                                                .child((
                                                    blog_interface::html::h2().class("blog-row")
                                                        .child(t("基本信息", "Basic information")),
                                                    blog_interface::html::div()
                                                        .child((
                                                            Field(
                                                                component_props_builder(&Field)
                                                                    .label(t(
                                                                        "网站名称",
                                                                        "Site name",
                                                                    ))
                                                                    .key("web_name")
                                                                    .form(form)
                                                                    .required(true)
                                                                    .build(),
                                                            ),
                                                            Field(
                                                                component_props_builder(&Field)
                                                                    .label(t(
                                                                        "网站标题",
                                                                        "Site title",
                                                                    ))
                                                                    .key("web_title")
                                                                    .form(form)
                                                                    .build(),
                                                            ),
                                                            Field(
                                                                component_props_builder(&Field)
                                                                    .label(t(
                                                                        "站长显示名",
                                                                        "Site owner display name",
                                                                    ))
                                                                    .key("author_name")
                                                                    .form(form)
                                                                    .required(true)
                                                                    .max(100)
                                                                    .build(),
                                                            ),
                                                        ))
                                                        .class("form-grid"),
                                                    blog_interface::html::p()
                                                        .child(t("用于文章和随笔署名；管理员登录名与密码在右上角头像的账号设置中修改。", "Used as the author of articles and notes. Change administrator sign-in credentials through the account settings avatar."))
                                                        .class("blog-field-help blog-row"),
                                                    Field(
                                                        component_props_builder(&Field)
                                                            .label(t("公告", "Notices"))
                                                            .key("notices")
                                                            .form(form)
                                                            .textarea(true)
                                                            .build(),
                                                    ),
                                                    blog_interface::html::p()
                                                        .child(t("使用 JSON 字符串数组或每行一条公告。", "Use a JSON string array or one notice per line."))
                                                        .class("blog-field-help blog-row"),
                                                    blog_interface::html::details().child((
                                                        blog_interface::html::summary().class("blog-row").child(t(
                                                            "推送格式",
                                                            "Promotion format",
                                                        )),
                                                        blog_interface::html::p().class("blog-row").child((
                                                            blog_interface::html::code()
                                                                .child("推送标题："),
                                                            " ",
                                                            blog_interface::html::code()
                                                                .child("推送封面："),
                                                            " ",
                                                            blog_interface::html::code()
                                                                .child("推送链接："),
                                                        )),
                                                    )),
                                                ))
                                                .class("blog-content-panel"),
                                            SiteAppearance(
                                                component_props_builder(&SiteAppearance)
                                                    .form(form)
                                                    .upload(upload)
                                                    .save_busy(save.busy)
                                                    .build(),
                                            ),
                                            blog_interface::html::section()
                                                .child((
                                                    blog_interface::html::h2().class("blog-row")
                                                        .child(t("随机封面", "Random covers")),
                                                    blog_interface::html::p()
                                                        .child(t("使用 JSON 图片地址数组或每行一个图片地址。", "Use a JSON array of image URLs or one image URL per line."))
                                                        .class("blog-field-help blog-row"),
                                                    Field(
                                                        component_props_builder(&Field)
                                                            .label(t("随机封面", "Random covers"))
                                                            .key("random_cover")
                                                            .form(form)
                                                            .textarea(true)
                                                            .build(),
                                                    ),
                                                ))
                                                .class("blog-content-panel"),
                                        ))
                                        .class("blog-fieldset")
                                        .disabled(move || save.busy.get() || upload.busy.get())
                                        .aria_label(t("网站设置", "Site settings")),
                                    Feedback(
                                        component_props_builder(&Feedback)
                                            .error(upload.error)
                                            .build(),
                                    ),
                                    blog_interface::html::button()
                                        .class("blog-button")
                                        .r#type("button")
                                        .child(t("取消上传", "Cancel upload"))
                                        .disabled(move || !upload.busy.get())
                                        .on(blog_interface::ev::click, move |_| upload.abort()),
                                    blog_interface::html::div()
                                        .child(
                                            blog_interface::html::button()
                                                .child(t("保存设置", "Save settings"))
                                                .class("blog-button")
                                                .r#type("submit")
                                                .disabled(move || {
                                                    save.busy.get() || upload.busy.get()
                                                }),
                                        )
                                        .class("blog-actions"),
                                ))
                                .class("blog-settings-form")
                                .on(blog_interface::ev::input, move |_| validation.set(String::new()))
                                .on(blog_interface::ev::submit, move |ev| {
                                    ev.prevent_default();
                                    if save.busy.get_untracked() || upload.busy.get_untracked() {
                                        return;
                                    }
                                    let mut f = form.get_untracked();
                                    let name = text(&f, "author_name").trim().to_string();
                                    let len = name.chars().count();
                                    if !(1..=100).contains(&len) {
                                        validation.set(t(
                                            "站长显示名须包含 1–100 个字符。",
                                            "Site owner display name must contain 1–100 characters.",
                                        ));
                                        if let Some(field) = form_node.get_untracked()
                                            .and_then(|node| node.query_selector("[name=author_name]").ok().flatten())
                                            .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                                        {
                                            let _ = field.focus();
                                        }
                                        return;
                                    }
                                    validation.set(String::new());
                                    f["author_name"] = json!(name);
                                    save.run(
                                        "/api/v1/content/site".into(),
                                        "PUT",
                                        Some(f),
                                        move |v| {
                                            form.set(v);
                                            notify(t("站点设置已保存", "Site settings saved"));
                                        },
                                    );
                                }),
                        )
                    })
                },
            ))
            .class("blog-content-stack"),
    )
}
