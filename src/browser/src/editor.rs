use crate::common::Action;
use crate::common::*;
use crate::models::{Article, ArticleDraft, Category, Label, News, Page};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::{Value, json};

#[derive(Clone)]
struct CreationReceipt {
    key: String,
    body: Value,
    ambiguous: bool,
}

// Keep the first wire payload with its key: an unsuccessful fetch may follow a
// committed write. Subsequent edits belong to a later PUT after confirmation.
fn creation_receipt(save: Action) -> RwSignal<Option<CreationReceipt>> {
    let receipt = RwSignal::new(None::<CreationReceipt>);
    Effect::new(move |_| {
        let Some(error) = save.error.get() else {
            return;
        };
        receipt.update(|pending| {
            if let Some(request) = pending {
                if !request.ambiguous
                    && [400, 401, 403, 404, 409, 413, 429].contains(&error.status)
                    && error.code != "invalid_error_response"
                {
                    *pending = None;
                } else {
                    request.ambiguous = true;
                }
            }
        });
    });
    receipt
}

fn prepare_creation(
    receipt: RwSignal<Option<CreationReceipt>>,
    body: Value,
) -> (Value, RequestOptions) {
    let request = receipt.get_untracked().unwrap_or_else(|| {
        let request = CreationReceipt {
            key: web_sys::window().unwrap().crypto().unwrap().random_uuid(),
            body,
            ambiguous: false,
        };
        receipt.set(Some(request.clone()));
        request
    });
    (
        request.body,
        RequestOptions {
            headers: vec![("Idempotency-Key".into(), request.key)],
            ..Default::default()
        },
    )
}

fn pending_creation(receipt: RwSignal<Option<CreationReceipt>>) -> impl IntoView {
    move || {
        receipt.get().map(|_| {
            View::new(blog_interface::html::p().class("blog-row").role("status").child(t(
                "原提交等待确认；确认后会保留较新的修改。",
                "The original submission is awaiting confirmation. Your newer edits will be kept.",
            )))
        })
    }
}

#[component]
pub fn ArticleEditor(#[prop(default = 0)] id: i64) -> impl IntoView {
    let categories = load_typed::<Vec<Category>>(Signal::derive(|| "/api/v1/categories".into()));
    let labels = load_typed::<Vec<Label>>(Signal::derive(|| "/api/v1/content/labels".into()));
    let article = load_typed::<Article>(Signal::derive(move || {
        if id > 0 {
            format!("/api/v1/content/articles/{id}")
        } else {
            String::new()
        }
    }));
    let form = RwSignal::new(ArticleDraft::default());
    let existing = RwSignal::new(false);
    let ready = RwSignal::new(id == 0);
    let preview = RwSignal::new(false);
    let save = Action::new();
    let upload = Action::new();
    let validation = RwSignal::new(String::new());
    let creation = creation_receipt(save);
    let created_id = RwSignal::new(None::<i64>);
    Effect::new(move |_| {
        if let Some(v) = article.value.get() {
            form.set(ArticleDraft::from(&v));
            existing.set(v.password_required);
            ready.set(true);
        }
    });
    let upload_file = move |file: blog_interface::thread_bound::SendWrapper<web_sys::File>,
                            cover: bool| {
        if !ready.get_untracked() || save.busy.get_untracked() {
            return;
        }
        let name = file.name().replace(['[', ']'], "");
        upload.upload(file, move |v| {
            let path = text(&v, "path");
            form.update(|f| {
                if cover {
                    f.article_cover = Some(path)
                } else {
                    f.article_content = format!("{}\n\n![{name}]({path})\n", f.article_content)
                }
            });
            notify(t("图片已上传", "Image uploaded"));
        });
    };
    let render_form = move || {
        (
            blog_interface::html::form()
                .child(
                    blog_interface::html::fieldset()
                        .child((
                            ControlledField(
                                component_props_builder(&ControlledField)
                                    .label(t("标题", "Title"))
                                    .value(Signal::derive(move || form.get().article_title))
                                    .change(Callback::new(move |v: String| {
                                        form.update(|f| f.article_title = v)
                                    }))
                                    .required(true)
                                    .max(120)
                                    .build(),
                            ),
                            blog_interface::html::div()
                                .child((
                                    blog_interface::html::label()
                                        .child((
                                            blog_interface::html::span().class("blog-field-label blog-row").child(t("分类", "Category")),
                                            crate::select::Select(
                                                component_props_builder(&crate::select::Select)
                                                    .label(Signal::derive(|| t("分类", "Category")))
                                                    .value(Signal::derive(move || {
                                                        form.get().sort_id.to_string()
                                                    }))
                                                    .options(Signal::derive(move || {
                                                        std::iter::once((
                                                            "0".to_string(),
                                                            t("选择分类", "Select a category"),
                                                        ))
                                                        .chain(
                                                            categories
                                                                .value
                                                                .get()
                                                                .unwrap_or_default()
                                                                .into_iter()
                                                                .map(|c| {
                                                                    (c.id.to_string(), c.sort_name)
                                                                }),
                                                        )
                                                        .collect()
                                                    }))
                                                    .required(true)
                                                    .change(Callback::new(move |v: String| {
                                                        form.update(|f| {
                                                            f.sort_id =
                                                                v.parse::<i64>().unwrap_or(0);
                                                            f.label_id = 0;
                                                        })
                                                    }))
                                                    .build(),
                                            ),
                                        ))
                                        .class("blog-form-field blog-stack"),
                                    blog_interface::html::label()
                                        .child((
                                            blog_interface::html::span().class("blog-field-label blog-row").child(t("标签", "Tags")),
                                            crate::select::Select(
                                                component_props_builder(&crate::select::Select)
                                                    .label(Signal::derive(|| t("标签", "Tags")))
                                                    .value(Signal::derive(move || {
                                                        form.get().label_id.to_string()
                                                    }))
                                                    .options(Signal::derive(move || {
                                                        std::iter::once((
                                                            "0".to_string(),
                                                            t("选择标签", "Select a tag"),
                                                        ))
                                                        .chain(
                                                            labels
                                                                .value
                                                                .get()
                                                                .unwrap_or_default()
                                                                .into_iter()
                                                                .filter(|c| {
                                                                    c.sort_id == form.get().sort_id
                                                                })
                                                                .map(|c| {
                                                                    (c.id.to_string(), c.label_name)
                                                                }),
                                                        )
                                                        .collect()
                                                    }))
                                                    .required(true)
                                                    .change(Callback::new(move |v: String| {
                                                        form.update(|f| {
                                                            f.label_id =
                                                                v.parse::<i64>().unwrap_or(0)
                                                        })
                                                    }))
                                                    .build(),
                                            ),
                                        ))
                                        .class("blog-form-field blog-stack"),
                                ))
                                .class("form-grid"),
                            blog_interface::html::div()
                                .child((
                                    ControlledField(
                                        component_props_builder(&ControlledField)
                                            .label(t("封面图片地址", "Cover image URL"))
                                            .value(Signal::derive(move || {
                                                form.get().article_cover.unwrap_or_default()
                                            }))
                                            .change(Callback::new(move |v: String| {
                                                form.update(|f| f.article_cover = Some(v))
                                            }))
                                            .build(),
                                    ),
                                    FilePicker(
                                        component_props_builder(&FilePicker)
                                            .label(t("上传封面图片", "Upload cover image"))
                                            .disabled(Signal::derive(move || {
                                                save.busy.get() || upload.busy.get()
                                            }))
                                            .choose(Callback::new(move |file| {
                                                upload_file(file, true)
                                            }))
                                            .build(),
                                    ),
                                ))
                                .class("form-grid"),
                            blog_interface::html::label()
                                .child((
                                    blog_interface::html::span().class("blog-field-label blog-row").child(t("Markdown 正文", "Markdown content")),
                                    blog_interface::html::textarea()
                                        .class("blog-input editor-textarea")
                                        .required(true)
                                        .prop("value", move || form.get().article_content)
                                        .on(blog_interface::ev::input, move |ev| {
                                            form.update(|f| {
                                                f.article_content = event_target_value(&ev)
                                            })
                                        }),
                                ))
                                .class("blog-form-field blog-stack"),
                            blog_interface::html::div()
                                .child((
                                    blog_interface::html::button()
                                        .child(move || {
                                            if preview.get() {
                                                t("收起预览", "Hide preview")
                                            } else {
                                                t("预览正文", "Preview content")
                                            }
                                        })
                                        .class("blog-button")
                                        .r#type("button")
                                        .on(blog_interface::ev::click, move |_| {
                                            preview.update(|v| *v = !*v)
                                        }),
                                    FilePicker(
                                        component_props_builder(&FilePicker)
                                            .label(t("插入正文图片", "Insert content image"))
                                            .disabled(Signal::derive(move || {
                                                save.busy.get() || upload.busy.get()
                                            }))
                                            .choose(Callback::new(move |file| {
                                                upload_file(file, false)
                                            }))
                                            .build(),
                                    ),
                                ))
                                .class("blog-actions blog-flow"),
                            move || {
                                preview.get().then(|| {
                                    View::new(
                                        blog_interface::html::div()
                                            .class("editor-markdown-preview blog-frame blog-frame--padded blog-stack blog-stack--spaced")
                                            .inner_html(crate::browser::markdown_html(
                                                &form.get().article_content,
                                            )),
                                    )
                                })
                            },
                            blog_interface::html::div()
                                .child((
                                    ControlledField(
                                        component_props_builder(&ControlledField)
                                            .label(t("视频地址", "Video URL"))
                                            .value(Signal::derive(move || {
                                                form.get().video_url.unwrap_or_default()
                                            }))
                                            .change(Callback::new(move |v: String| {
                                                form.update(|f| f.video_url = Some(v))
                                            }))
                                            .build(),
                                    ),
                                    move || {
                                        (!form.get().view_status).then(|| {
                                            View::new(
    blog_interface::html::div()
        .child(
            (
                    blog_interface::html::label()
                    .child((blog_interface::html::span().class("blog-field-label blog-row").child(t("文章访问密码", "Article access password")),
                    blog_interface::html::input()
                    .class("blog-input").r#type("password")
                    .aria_describedby(move
                    | | crate ::dates::password_error(& form.get().password
                    .unwrap_or_default(), existing.get()).map(| _ |
                    "blog-article-password-error"))
                    .aria_invalid(move
                    | | crate ::dates::password_error(& form.get().password
                    .unwrap_or_default(), existing.get()).is_some().to_string())
                    .placeholder(move
                    | | if existing.get() { t("留空则保留原密码",
                    "Leave empty to keep the current password") } else {
                    t("至少 12 字节", "At least 12 bytes") }).prop("value",
                    move | | form.get().password
                    .unwrap_or_default()).on(blog_interface::ev::input,
                    move | ev | form.update(| f | f.password =
                    Some(event_target_value(& ev)))),)).class("blog-form-field blog-stack"),
                    move | | crate ::dates::password_error(& form.get().password
                    .unwrap_or_default(), existing.get()).map(| e | View::new(
    blog_interface::html::p().class("blog-row blog-danger")
        .child(

            e,
        )
        .id("blog-article-password-error")
        .role("alert"),
))
                ,),
        ),
)
                                        })
                                    },
                                ))
                                .class("form-grid"),
                            blog_interface::html::div()
                                .child((
                                    blog_interface::html::label().class("blog-check blog-row blog-flow").child((
                                        blog_interface::html::input()
                                            .r#type("checkbox")
                                            .prop("checked", move || form.get().view_status)
                                            .on(blog_interface::ev::change, move |ev| {
                                                let v = event_target_checked(&ev);
                                                form.update(|f| {
                                                    f.view_status = v;
                                                    if v {
                                                        f.password = None;
                                                    }
                                                });
                                            }),
                                        t(
                                            " 公开访问（关闭后需设置密码）",
                                            " Public access (a password is required when disabled)",
                                        ),
                                    )),
                                    ControlledCheck(
                                        component_props_builder(&ControlledCheck)
                                            .label(t(" 推荐", " Recommended"))
                                            .value(Signal::derive(move || {
                                                form.get().recommend_status
                                            }))
                                            .change(Callback::new(move |v| {
                                                form.update(|f| f.recommend_status = v)
                                            }))
                                            .build(),
                                    ),
                                    ControlledCheck(
                                        component_props_builder(&ControlledCheck)
                                            .label(t(" 允许评论", " Allow comments"))
                                            .value(Signal::derive(move || {
                                                form.get().comment_status
                                            }))
                                            .change(Callback::new(move |v| {
                                                form.update(|f| f.comment_status = v)
                                            }))
                                            .build(),
                                    ),
                                ))
                                .class("check-row blog-flow"),
                            move || {
                                (!form.get().view_status).then(|| {
                                    View::new((
                                        ControlledField(
                                            component_props_builder(&ControlledField)
                                                .label(t("密码提示", "Password hint"))
                                                .value(Signal::derive(move || {
                                                    form.get().tips.unwrap_or_default()
                                                }))
                                                .change(Callback::new(move |v: String| {
                                                    form.update(|f| f.tips = Some(v))
                                                }))
                                                .build(),
                                        ),
                                        blog_interface::html::span()
                                            .child(t(
                                                "显示在文章访问密码输入页。",
                                                "Displayed on the article access password screen.",
                                            ))
                                            .class("blog-field-help blog-row"),
                                    ))
                                })
                            },
                            blog_interface::html::div()
                                .child((
                                    blog_interface::html::button()
                                        .child(t("取消", "Cancel"))
                                        .class("blog-button")
                                        .r#type("button")
                                        .disabled(move || save.busy.get() || creation.get().is_some())
                                        .on(blog_interface::ev::click, move |_| {
                                            upload.abort();
                                            navigate("articles")
                                        }),
                                    blog_interface::html::button()
                                        .child(move || {
                                            if upload.busy.get() {
                                                t("图片上传中…", "Uploading image…")
                                            } else if save.busy.get() {
                                                t("保存中…", "Saving…")
                                            } else if creation.get().is_some() {
                                                t("确认原文章提交", "Confirm original article")
                                            } else {
                                                t("保存文章", "Save article")
                                            }
                                        })
                                        .class("blog-button")
                                        .r#type("submit")
                                        .formnovalidate(move || creation.get().is_some())
                                        .disabled(move || save.busy.get() || upload.busy.get()),
                                ))
                                .class("blog-actions blog-flow"),
                        ))
                        .class("blog-fieldset blog-stack")
                        .disabled(move || save.busy.get()),
                )
                .class("editor-form blog-frame blog-frame--padded blog-stack")
                .on(blog_interface::ev::submit, move |ev| {
                    ev.prevent_default();
                    if save.busy.get_untracked()
                        || upload.busy.get_untracked()
                        || !ready.get_untracked()
                    {
                        return;
                    }
                    let f = form.get_untracked();
                    let issue = if creation.get_untracked().is_some() {
                        None
                    } else if f.article_title.trim().is_empty()
                        || f.article_title.chars().count() > 120
                    {
                        Some(t("文章标题须为 1–120 个字符", "Article title must contain 1–120 characters"))
                    } else if f.view_status {
                        None
                    } else {
                        crate::dates::password_error(
                            f.password.as_deref().unwrap_or_default(),
                            existing.get_untracked(),
                        )
                    };
                    if let Some(issue) = issue {
                        validation.set(issue);
                        return;
                    }
                    validation.set(String::new());
                    let f = f.for_submission();
                    let Ok(body) = blog_interface::api::encode_json_body(&f) else {
                        save.error.set(Some(invalid()));
                        return;
                    };
                    let target = if id > 0 {
                        Some(id)
                    } else {
                        created_id.get_untracked()
                    };
                    let path = target
                        .map(|id| format!("/api/v1/content/articles/{id}"))
                        .unwrap_or("/api/v1/content/articles".into());
                    let (body, opts) = if target.is_none() {
                        prepare_creation(creation, body)
                    } else {
                        (body, RequestOptions::default())
                    };
                    let submitted = body.clone();
                    save.run_typed_options(
                        path,
                        if target.is_some() { "PUT" } else { "POST" },
                        Some(body),
                        opts,
                        move |v: Article| {
                            if target.is_some_and(|target| target != v.id) {
                                save.error.set(Some(invalid()));
                                return;
                            }
                            if target.is_none() {
                                created_id.set(Some(v.id));
                                existing.set(v.password_required);
                                creation.set(None);
                                if blog_interface::api::encode_json_body(
                                    &form.get_untracked().for_submission(),
                                ).ok().as_ref() != Some(&submitted) {
                                    notify(t(
                                        "原文章已确认，较新的修改可继续保存。",
                                        "Original article confirmed. Your newer edits are ready to save.",
                                    ));
                                    return;
                                }
                            }
                            notify(t("文章已保存", "Article saved"));
                            navigate("articles");
                        },
                    );
                }),
            move || {
                (if id > 0 { Some(id) } else { created_id.get() }).map(|id| {
                    View::new(NewsEditor(
                        component_props_builder(&NewsEditor).id(id).build(),
                    ))
                })
            },
        )
    };
    View::new(
    blog_interface::html::section()
        .child(
            (
                    blog_interface::html::h1().class("blog-row")
                    .child( move || if id > 0 || created_id.get().is_some() {
                    t("编辑文章", "Edit article") } else { t("新增文章",
                    "Add article") }),
                    blog_interface::html::p().class("blog-row")
                    .child( t("支持 Markdown 正文，发布后可从网站访问。",
                    "Write in Markdown. Published articles are available on the site.")), Feedback(component_props_builder(& Feedback)
                    .error( categories.error)
                    .fallback( t("分类加载失败",
                    "Unable to load categories")).retry( Some(Callback::new(move | () | categories.reload()))).build()),
                    Feedback(component_props_builder(& Feedback)
                    .error( labels.error)
                    .fallback( t("标签加载失败",
                    "Unable to load tags")).retry( Some(Callback::new(move | () | labels.reload()))).build()),
                    Feedback(component_props_builder(& Feedback)
                    .error( article.error)
                    .fallback( t("文章加载失败",
                    "Unable to load article")).retry( Some(Callback::new(move | () | article.reload()))).build()),
                    Feedback(component_props_builder(& Feedback)
                    .error( save.error)
                    .fallback( t("尚未确认文章是否已创建，请确认原提交；较新的修改会保留。",
                    "Article creation is not yet confirmed. Confirm the original submission; your newer edits will be kept.")).build()),
                    pending_creation(creation),
                    Feedback(component_props_builder(& Feedback)
                    .error( upload.error).build()),
                    move | | (! validation.get().is_empty()).then(| | View::new(
    blog_interface::html::p().class("blog-row blog-danger")
        .child(

            validation.get(),
        )
        .role("alert"),
)),
                    Show(component_props_builder(& Show)
                    .when( move | | ready.get())
                    .fallback( move | | View::new(
    (
            blog_interface::html::p().class("blog-row")
            .child( t("正在加载…",
            "Loading…")).role("status"),
            blog_interface::html::button()
            .child( t("取消", "Cancel"))
            .class("blog-button").on(blog_interface::ev::click, move | _ |
            navigate("articles"))
        ,),
))
                    .children(::blog_interface::children::ToChildren::to_children(render_form))
                    .build())
                ,),
        )
        .class("blog-content-stack blog-stack"),
)
}
#[component]
fn NewsEditor(id: i64) -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load_typed::<Page<News>>(Signal::derive(move || {
        format!(
            "/api/v1/content/articles/{id}/news?page={}&size=20",
            page.get()
        )
    }));
    let form = RwSignal::new(json!({"content":"","create_time":""}));
    let original = RwSignal::new(json!({"content":"","create_time":""}));
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    let receipt = creation_receipt(save);
    let validation = RwSignal::new(String::new());
    let reset = Callback::new(move |()| {
        if receipt.get_untracked().is_some() {
            return;
        }
        let empty = json!({"content":"","create_time":""});
        form.set(empty.clone());
        original.set(empty);
        editing.set(None);
        validation.set(String::new());
    });
    let render_row = move |item: News| {
        let entry_id = item.id;
        let edit = item.clone();
        View::new(
            blog_interface::html::tr().child((
                blog_interface::html::td()
                    .child(crate::dates::format(&item.create_time.unwrap_or_default())),
                blog_interface::html::td()
                    .child(item.content)
                    .class("comment-cell"),
                blog_interface::html::td().child(
                    blog_interface::html::div()
                        .child((
                            blog_interface::html::button()
                                .child(t("编辑", "Edit"))
                                .class("blog-button")
                                .disabled(move || save.busy.get() || receipt.get().is_some())
                                .on(blog_interface::ev::click, move |_| {
                                    if form.get_untracked() != original.get_untracked()
                                        && !web_sys::window()
                                            .unwrap()
                                            .confirm_with_message(&t(
                                                "放弃当前未保存的进展修改？",
                                                "Discard the current unsaved update changes?",
                                            ))
                                            .unwrap_or(false)
                                    {
                                        return;
                                    }
                                    let f = json!({ "content" : edit.content,
                    "create_time" : crate ::dates::input(& edit.create_time.clone()
                    .unwrap_or_default()) });
                                    form.set(f.clone());
                                    original.set(f);
                                    editing.set(Some(entry_id));
                                }),
                            blog_interface::html::button()
                                .child(t("删除", "Delete"))
                                .class("blog-button blog-danger")
                                .disabled(move || save.busy.get() || receipt.get().is_some())
                                .on(blog_interface::ev::click, move |_| {
                                    if web_sys::window()
                                        .unwrap()
                                        .confirm_with_message(&t(
                                            "确定删除这条文章进展？",
                                            "Delete this article update?",
                                        ))
                                        .unwrap_or(false)
                                    {
                                        save.run(
                                            format!(
                                                "/api/v1/content/articles/{id}/news/{entry_id}"
                                            ),
                                            "DELETE",
                                            None,
                                            move |_| {
                                                if editing.get_untracked() == Some(entry_id) {
                                                    reset.run(())
                                                }
                                                data.reload();
                                            },
                                        );
                                    }
                                }),
                        ))
                        .class("blog-actions blog-flow"),
                ),
            )),
        )
    };
    View::new(
    blog_interface::html::section()
        .child(
            (
                    blog_interface::html::h2().class("blog-row")
                    .child( t("最新进展",
                    "Recent updates")),
                    Feedback(component_props_builder(& Feedback)
                    .error( data.error)
                    .fallback( t("文章进展加载失败",
                    "Unable to load article updates"))
                    .retry( Some(Callback::new(move | () |
                    data.reload()))).build()),
                    Feedback(component_props_builder(& Feedback)
                    .error( save.error).build()),
                    pending_creation(receipt),
                    move | | (! validation.get().is_empty()).then(| | View::new(
    blog_interface::html::p().class("blog-row blog-danger")
        .child(

            validation.get(),
        )
        .role("alert"),
)),
                    blog_interface::html::form()
                    .child( blog_interface::html::fieldset()
                    .child((blog_interface::html::h3().class("blog-row")
                    .child( move | | if editing.get().is_some() { t("编辑进展",
                    "Edit update") } else { t("新增进展", "Add update") }), Field(component_props_builder(& Field)
                    .label( t("进展内容",
                    "Update content")).key( "content")
                    .form( form)
                    .textarea( true)
                    .required( true)
                    .max( 1024).build()),
                    blog_interface::html::label()
                    .child((blog_interface::html::span().class("blog-field-label blog-row").child(t("进展时间（UTC）", "Update time (UTC)")),
                    blog_interface::html::input()
                    .class("blog-input").r#type("datetime-local").step("1")
                    .prop("value", move | | text(&
                    form.get(), "create_time"))
                    .on(blog_interface::ev::input, move | ev | form
                    .update(| f | f["create_time"] = json!(event_target_value(&
                    ev)))),)).class("blog-form-field blog-stack"),
                    blog_interface::html::p()
                    .child( t("新增时留空使用当前 UTC；编辑时留空保留原时间。",
                    "Leave blank to use current UTC for a new update, or keep its existing time when editing.")).class("blog-field-help blog-row"),
                    blog_interface::html::div()
                    .child((blog_interface::html::button()
                    .child( move | | if receipt.get().is_some() {
                    t("确认原进展提交", "Confirm original update")
                    } else if editing.get().is_some() { t("保存修改",
                    "Save changes") } else { t("发布进展", "Publish update") }).class("blog-button").r#type("submit")
                    .formnovalidate(move || receipt.get().is_some()),
                    move | | (editing.get().is_some() || form.get() != original
                    .get()).then(| | View::new(
    blog_interface::html::button()
        .child(

            t("取消编辑", "Cancel editing"),
        )
        .class("blog-button")
        .r#type("button")
        .disabled(move || receipt.get().is_some())
        .on(
            blog_interface::ev::click,
            move |_| if form.get_untracked() == original.get_untracked()
                    || web_sys::window()
                        .unwrap()
                        .confirm_with_message(
                            &t(
                                "放弃当前未保存的进展修改？",
                                "Discard the current unsaved update changes?",
                            ),
                        )
                        .unwrap_or(false)
                {
                    reset.run(())
                },
        ),
)),)).class("blog-actions blog-flow"),))
                    .class("blog-fieldset blog-stack")
                    .disabled(move
                    | | save.busy.get() || data.value.get().is_none()))
                    .class("editor-form blog-frame blog-frame--padded blog-stack").on(blog_interface::ev::submit,
                    move |ev| {
                        ev.prevent_default();
                        if save.busy.get_untracked() {
                            return;
                        }
                        let f = form.get_untracked();
                        let content = text(&f, "content");
                        let confirming = receipt.get_untracked().is_some();
                        if !confirming && !(1..=1024).contains(&content.trim().chars().count()) {
                            validation.set(t("进展正文须为 1–1024 个字符", "Update content must contain 1–1024 characters"));
                            return;
                        }
                        validation.set(String::new());
                        let input = json!({"content":content,"create_time":crate::dates::iso(&text(&f,"create_time"))});
                        let target = editing.get_untracked();
                        let path = format!("/api/v1/content/articles/{id}/news{}",target.map(|id|format!("/{id}")).unwrap_or_default());
                        let (input, opts) = if target.is_none() {
                            prepare_creation(receipt, input)
                        } else {
                            (input, RequestOptions::default())
                        };
                        let submitted = input.clone();
                        save.run_options(path, if target.is_some() { "PUT" } else { "POST" }, Some(input), opts, move |v| {
                            if (!confirming && text(&v,"content") != text(&submitted,"content").trim())
                                || target.is_some_and(|id|num(&v,"id") != id)
                            {
                                save.error.set(Some(invalid()));
                                return;
                            }
                            receipt.set(None);
                            let current = form.get_untracked();
                            let current_body = json!({"content":text(&current,"content"),"create_time":crate::dates::iso(&text(&current,"create_time"))});
                            if target.is_none() && current_body != submitted {
                                editing.set(Some(num(&v,"id")));
                                original.set(json!({"content":v["content"],"create_time":crate::dates::input(&text(&v,"create_time"))}));
                                notify(t("原进展已确认，较新的修改可继续保存。", "Original update confirmed. Your newer edits are ready to save."));
                            } else {
                                reset.run(());
                                notify(t("文章进展已保存", "Article update saved"));
                            }
                            page.set(1);
                            data.reload();
                        });
                    }),
                    move | | data.value.get().map(| v | View::new(
    blog_interface::html::div()
        .child(

            blog_interface::html::table()
                    .child(
                        (
                                blog_interface::html::thead()
                                .child( blog_interface::html::tr()
                                .child((blog_interface::html::th()
                                .child( t("时间",
                                "Time")),
                                blog_interface::html::th()
                                .child( t("内容",
                                "Content")),
                                blog_interface::html::th()
                                .child( t("操作",
                                "Actions")),))),
                                blog_interface::html::tbody()
                                .child( v.items
                                .into_iter().map(render_row).collect_view())
                            ,),
                    )
                    .class("blog-table")
                    .aria_label(
                        t("文章进展", "Article updates"),
                    ),
        )
        .class("blog-table-scroll blog-frame")
        .tabindex("0")
        .role("region")
        .aria_label(
            t("数据表格", "Data table"),
        ),
)),
                    Pager(component_props_builder(& Pager)
                    .data( data)
                    .page( page)
                    .label( t("进展分页",
                    "Update pages")).build())
                ,),
        )
        .class("blog-content-stack blog-stack blog-stack--spaced"),
)
}
fn empty_note() -> Value {
    json!({"content":"","image_path":null,"is_public":true})
}
#[component]
pub fn Notes() -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load(Signal::derive(move || {
        format!("/api/v1/content/notes?page={}&size=20", page.get())
    }));
    let form = RwSignal::new(empty_note());
    let original = RwSignal::new(empty_note());
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    let receipt = creation_receipt(save);
    let upload = Action::new();
    let filename = RwSignal::new(String::new());
    let lightbox = RwSignal::new(None::<String>);
    let validation = RwSignal::new(String::new());
    let reset = Callback::new(move |()| {
        if receipt.get_untracked().is_some() {
            return;
        }
        upload.abort();
        form.set(empty_note());
        original.set(empty_note());
        editing.set(None);
        filename.set(String::new());
        validation.set(String::new());
    });
    let render_row = move |item: Value| {
        let id = num(&item, "id");
        let edit = item.clone();
        let toggle = item.clone();
        View::new(
            blog_interface::html::tr().child((
                blog_interface::html::td().child({
                    let n = text(&item, "username");
                    if n.is_empty() {
                        t("站长", "Site owner")
                    } else {
                        n
                    }
                }),
                blog_interface::html::td()
                    .child(text(&item, "content"))
                    .class("comment-cell"),
                blog_interface::html::td().child(safe_url(&text(&item, "image_path"), true).map(
                    |u| {
                        let click = u.clone();
                        View::new(
                            blog_interface::html::button()
                                .child(
                                    blog_interface::html::img()
                                        .src(u)
                                        .alt(t("查看随笔图片", "View note image")),
                                )
                                .class("moderation-photo")
                                .r#type("button")
                                .on(blog_interface::ev::click, move |_| {
                                    lightbox.set(Some(click.clone()))
                                }),
                        )
                    },
                )),
                blog_interface::html::td().child(if flag(&item, "is_public") {
                    t("公开", "Public")
                } else {
                    t("私密", "Private")
                }),
                blog_interface::html::td().child(table_time(&text(&item, "create_time"))),
                blog_interface::html::td().child(
                    blog_interface::html::div()
                        .child((
                            blog_interface::html::button()
                                .child(t("编辑", "Edit"))
                                .class("blog-button")
                                .disabled(move || {
                                    save.busy.get() || upload.busy.get() || receipt.get().is_some()
                                })
                                .on(blog_interface::ev::click, move |_| {
                                    if form.get_untracked() != original.get_untracked()
                                        && !web_sys::window()
                                            .unwrap()
                                            .confirm_with_message(&t(
                                                "放弃当前未保存的随笔修改？",
                                                "Discard the current unsaved note changes?",
                                            ))
                                            .unwrap_or(false)
                                    {
                                        return;
                                    }
                                    let f = json!({ "content" : edit["content"],
                    "image_path" : edit["image_path"], "is_public" : flag(& edit,
                    "is_public") });
                                    form.set(f.clone());
                                    original.set(f);
                                    editing.set(Some(id));
                                    filename.set(String::new());
                                }),
                            blog_interface::html::button()
                                .child(if flag(&item, "is_public") {
                                    t("隐藏", "Hide")
                                } else {
                                    t("公开", "Publish")
                                })
                                .class("blog-button")
                                .disabled(move || {
                                    save.busy.get()
                                        || upload.busy.get()
                                        || receipt.get().is_some()
                                        || editing.get() == Some(id) && form.get() != original.get()
                                })
                                .on(blog_interface::ev::click, move |_| {
                                    let visible = !flag(&toggle, "is_public");
                                    let input = json!({
                    "content" : toggle["content"], "image_path" :
                    toggle["image_path"], "is_public" : visible });
                                    save.run(
                                        format!("/api/v1/content/notes/{id}"),
                                        "PUT",
                                        Some(input),
                                        move |v| {
                                            if editing.get_untracked() == Some(id) {
                                                form.update(|f| f["is_public"] = json!(visible));
                                                original
                                                    .update(|f| f["is_public"] = json!(visible));
                                            }
                                            if num(&v, "id") != id
                                                || flag(&v, "is_public") != visible
                                            {
                                                save.error.set(Some(invalid()));
                                                return;
                                            }
                                            data.reload();
                                        },
                                    );
                                }),
                            blog_interface::html::button()
                                .child(t("删除", "Delete"))
                                .class("blog-button blog-danger")
                                .disabled(move || {
                                    save.busy.get() || upload.busy.get() || receipt.get().is_some()
                                })
                                .on(blog_interface::ev::click, move |_| {
                                    if web_sys::window()
                                        .unwrap()
                                        .confirm_with_message(&t(
                                            "确定删除这条随笔？",
                                            "Delete this note?",
                                        ))
                                        .unwrap_or(false)
                                    {
                                        save.run(
                                            format!("/api/v1/content/notes/{id}"),
                                            "DELETE",
                                            None,
                                            move |_| {
                                                if editing.get_untracked() == Some(id) {
                                                    reset.run(())
                                                }
                                                data.reload();
                                            },
                                        );
                                    }
                                }),
                        ))
                        .class("blog-actions blog-flow"),
                ),
            )),
        )
    };
    View::new(
    blog_interface::html::section()
        .child(
            (
                    blog_interface::html::h1().class("blog-row")
                    .child( t("随笔管理",
                    "Notes")),
                    blog_interface::html::p().class("blog-row")
                    .child( t("维护随笔及恋爱日常中的内容。",
                    "Manage notes shown in the journal and everyday moments.")), Feedback(component_props_builder(& Feedback)
                    .error( data.error)
                    .fallback( t("随笔加载失败",
                    "Unable to load notes")).retry( Some(Callback::new(move | () | data.reload()))).build()), Feedback(component_props_builder(& Feedback)
                    .error( save.error).build()), Feedback(component_props_builder(& Feedback)
                    .error( upload.error).build()),
                    pending_creation(receipt),
                    move | | (! validation.get().is_empty()).then(| | View::new(
    blog_interface::html::p().class("blog-row blog-danger")
        .child(

            validation.get(),
        )
        .role("alert"),
)),
                    blog_interface::html::form()
                    .child( blog_interface::html::fieldset()
                    .child((blog_interface::html::h2().class("blog-row")
                    .child( move | | if editing.get().is_some() { t("编辑随笔",
                    "Edit note") } else { t("新增随笔", "Add note") }),
                    Field(component_props_builder(& Field)
                    .label( t("随笔正文",
                    "Note content")).key( "content")
                    .form( form)
                    .textarea( true)
                    .required( true)
                    .max( 1024).build()),
                    FilePicker(component_props_builder(& FilePicker)
                    .label( t("随笔图片", "Note image")).filename( Signal::derive(move | |
                    filename.get())).disabled( Signal::derive(move | | upload.busy.get() || save.busy.get()))
                    .choose( Callback::new(move | file :
                    blog_interface::thread_bound::SendWrapper < web_sys::File > | { let name = file
                    .name(); upload.upload(file, move | v | { form.update(| f |
                    f["image_path"] = v["path"].clone()); filename.set(name);
                    notify(t("图片已上传", "Image uploaded")); }); }))
                    .build()),
                    move | | safe_url(& text(& form.get(), "image_path"), true).map(|
                    url | { let click = url.clone(); View::new(
    blog_interface::html::div()
        .child(
            (
                    blog_interface::html::button()
                    .child( blog_interface::html::img()
                    .src(url)
                    .alt(t("查看随笔图片",
                    "View note image"))).class("moderation-photo")
                    .r#type("button").on(blog_interface::ev::click, move |
                    _ | lightbox.set(Some(click.clone()))),
                    blog_interface::html::button()
                    .child( t("移除图片",
                    "Remove image")).class("blog-button").r#type("button")
                    .disabled(move
                    | | upload.busy.get() || save.busy.get())
                    .on(blog_interface::ev::click, move | _ | { form
                    .update(| f | f["image_path"] = Value::Null); filename
                    .set(String::new()) })
                ,),
        ),
) }),
                    Check(component_props_builder(& Check)
                    .label( t(" 公开可见",
                    " Publicly visible")).key( "is_public").form( form).build()),
                    blog_interface::html::div()
                    .child((blog_interface::html::button()
                    .child( move | | if upload.busy.get() { t("图片上传中…",
                    "Uploading image…") } else if save.busy.get() {
                    t("保存中…", "Saving…") } else if receipt.get().is_some() {
                    t("确认原随笔提交", "Confirm original note") } else if editing.get().is_some()
                    { t("保存修改", "Save changes") } else { t("发布",
                    "Publish") }).class("blog-button").r#type("submit")
                    .formnovalidate(move || receipt.get().is_some())
                    .disabled(move
                    | | save.busy.get() || upload.busy.get()),
                    move | | (editing.get().is_some() || form.get() != original.get()
                    || upload.busy.get()).then(| | View::new(
    blog_interface::html::button()
        .child(

            t("取消编辑", "Cancel editing"),
        )
        .class("blog-button")
        .r#type("button")
        .disabled(
            move || save.busy.get() || receipt.get().is_some(),
        )
        .on(
            blog_interface::ev::click,
            move |_| if form.get_untracked() == original.get_untracked()
                    || web_sys::window()
                        .unwrap()
                        .confirm_with_message(
                            &t(
                                "放弃当前未保存的随笔修改？",
                                "Discard the current unsaved note changes?",
                            ),
                        )
                        .unwrap_or(false)
                {
                    reset.run(())
                },
        ),
)),)).class("blog-actions blog-flow"),))
                    .class("blog-fieldset blog-stack")
                    .disabled(move
                    | | save.busy.get())).class("editor-form blog-frame blog-frame--padded blog-stack")
                    .on(blog_interface::ev::submit, move |ev| {
                        ev.prevent_default();
                        if save.busy.get_untracked() || upload.busy.get_untracked() {
                            return;
                        }
                        let f = form.get_untracked();
                        let content = text(&f,"content");
                        let confirming = receipt.get_untracked().is_some();
                        if !confirming && !(1..=1024).contains(&content.trim().chars().count()) {
                            validation.set(t("随笔正文须为 1–1024 个字符", "Note content must contain 1–1024 characters"));
                            return;
                        }
                        validation.set(String::new());
                        let target = editing.get_untracked();
                        let path = target.map(|id|format!("/api/v1/content/notes/{id}")).unwrap_or("/api/v1/content/notes".into());
                        let (input, opts) = if target.is_none() {
                            prepare_creation(receipt, f)
                        } else {
                            (f, RequestOptions::default())
                        };
                        let submitted = input.clone();
                        save.run_options(path, if target.is_some() { "PUT" } else { "POST" }, Some(input), opts, move |v| {
                            if (!confirming && (text(&v,"content") != text(&submitted,"content").trim()
                                || v["image_path"] != submitted["image_path"]
                                || flag(&v,"is_public") != flag(&submitted,"is_public")))
                                || target.is_some_and(|id|num(&v,"id") != id)
                            {
                                save.error.set(Some(invalid()));
                                return;
                            }
                            receipt.set(None);
                            if target.is_none() && form.get_untracked() != submitted {
                                editing.set(Some(num(&v,"id")));
                                original.set(json!({"content":v["content"],"image_path":v["image_path"],"is_public":flag(&v,"is_public")}));
                                notify(t("原随笔已确认，较新的修改可继续保存。", "Original note confirmed. Your newer edits are ready to save."));
                            } else {
                                reset.run(());
                                notify(t("随笔已保存", "Note saved"));
                            }
                            page.set(1);
                            data.reload();
                        });
                    }),
                    move | | data.value.get().map(| v | View::new(
    blog_interface::html::div()
        .child(

            blog_interface::html::table()
                    .child(
                        (
                                blog_interface::html::thead()
                                .child( blog_interface::html::tr()
                                .child( [t("作者",
                                "Author"), t("内容", "Content"), t("图片", "Image"),
                                t("状态", "Status"), t("时间", "Time"), t("操作",
                                "Actions")].into_iter().map(| l | View::new(
    blog_interface::html::th()
        .child(

            l,
        ),
)).collect_view())),
                                blog_interface::html::tbody()
                                .child( rows(& v,
                                "items").into_iter().map(render_row).collect_view())
                            ,),
                    )
                    .class("blog-table")
                    .aria_label(
                        t("随笔列表", "Note list"),
                    ),
        )
        .class("blog-table-scroll blog-frame")
        .tabindex("0")
        .role("region")
        .aria_label(
            t("数据表格", "Data table"),
        ),
)),
                    ListState(component_props_builder(& ListState)
                    .data( data)
                    .empty( t("暂无随笔",
                    "No notes yet")).build()),
                    Pager(component_props_builder(& Pager)
                    .data( data)
                    .page( page).build()),
                    Lightbox(component_props_builder(& Lightbox)
                    .src( lightbox).build())
                ,),
        )
        .class("blog-content-stack blog-stack"),
)
}
