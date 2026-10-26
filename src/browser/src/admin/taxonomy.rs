use crate::common::{Action, Feedback, Field, list, load, num, t, text};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub(super) enum TaxonomyKind {
    Categories,
    Labels,
}

#[component]
pub(super) fn Taxonomy(kind: TaxonomyKind) -> impl IntoView {
    let labels = match kind {
        TaxonomyKind::Categories => false,
        TaxonomyKind::Labels => true,
    };
    let endpoint = if labels {
        "/api/v1/content/labels"
    } else {
        "/api/v1/content/categories"
    };
    let data = load(Signal::derive(move || {
        if labels {
            endpoint.into()
        } else {
            "/api/v1/categories".into()
        }
    }));
    let categories = load(Signal::derive(move || {
        if labels {
            "/api/v1/categories".into()
        } else {
            String::new()
        }
    }));
    let form = RwSignal::new(json!({"name":"","description":"","priority":0,"sort_id":0}));
    let editing = RwSignal::new(None::<i64>);
    let priority = RwSignal::new("0".to_string());
    let priority_input = NodeRef::<blog_interface::html::Input>::new();
    let validation = RwSignal::new(String::new());
    let save = Action::new();
    let deletion = Action::new();
    let deleting = RwSignal::new(None::<i64>);
    let reset = Callback::new(move |()| {
        editing.set(None);
        priority.set("0".into());
        validation.set(String::new());
        form.set(json!({"name":"","description":"","priority":0,"sort_id":0}));
    });
    let submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if save.busy.get_untracked()
            || deletion.busy.get_untracked() && deleting.get_untracked() == editing.get_untracked()
        {
            return;
        }
        let current = form.get_untracked();
        if labels
            && !list(categories.value.get_untracked())
                .iter()
                .any(|category| num(category, "id") == num(&current, "sort_id"))
        {
            return;
        }
        let mut body = json!({
            "name": current["name"],
            "description": if text(&current, "description").is_empty() {
                Value::Null
            } else {
                current["description"].clone()
            }
        });
        if labels {
            body["sort_id"] = current["sort_id"].clone();
        } else {
            match priority.get_untracked().trim().parse::<i64>() {
                Ok(value) if (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value) => {
                    body["priority"] = json!(value);
                }
                _ => {
                    validation.set(t(
                        "优先级须为 -9007199254740991 至 9007199254740991 之间的整数。",
                        "Priority must be a whole number between -9007199254740991 and 9007199254740991.",
                    ));
                    if let Some(input) = priority_input.get_untracked() {
                        let _ = input.focus();
                    }
                    return;
                }
            }
        }
        validation.set(String::new());
        let path = editing
            .get_untracked()
            .map(|id| format!("{endpoint}/{id}"))
            .unwrap_or(endpoint.into());
        save.run(
            path,
            if editing.get_untracked().is_some() {
                "PUT"
            } else {
                "POST"
            },
            Some(body),
            move |_| {
                reset.run(());
                data.reload();
                notify(if labels {
                    t("标签已保存", "Tag saved")
                } else {
                    t("分类已保存", "Category saved")
                });
            },
        );
    };
    let render_row = move |item: Value| {
        let id = num(&item, "id");
        let edit = item.clone();
        View::new(
            blog_interface::html::tr().child((
                labels.then(|| View::new(blog_interface::html::td().child(id))),
                blog_interface::html::td()
                    .class("blog-title-cell")
                    .child(text(&item, if labels { "label_name" } else { "sort_name" })),
                blog_interface::html::td().class("comment-cell").child(text(
                    &item,
                    if labels {
                        "label_description"
                    } else {
                        "sort_description"
                    },
                )),
                blog_interface::html::td().child(if labels {
                    list(categories.value.get())
                        .iter()
                        .find(|c| num(c, "id") == num(&item, "sort_id"))
                        .map(|c| text(c, "sort_name"))
                        .unwrap_or_default()
                } else {
                    num(&item, "priority").to_string()
                }),
                blog_interface::html::td().child(
                    blog_interface::html::div()
                        .child((
                            blog_interface::html::button()
                                .child(t("编辑", "Edit"))
                                .class("blog-button")
                                .on(blog_interface::ev::click, move |_| {
                                    editing.set(Some(id));
                                    priority.set(num(&edit, "priority").to_string());
                                    validation.set(String::new());
                                    form.set(json!({ "name" : text(& edit, if labels
                    { "label_name" } else { "sort_name" }), "description" : text(&
                    edit, if labels { "label_description" } else { "sort_description"
                    }), "priority" : num(& edit, "priority"), "sort_id" : num(& edit,
                    "sort_id") }));
                                }),
                            blog_interface::html::button()
                                .child(t("删除", "Delete"))
                                .class("blog-button blog-danger")
                                .disabled(move || deletion.busy.get())
                                .on(blog_interface::ev::click, move |_| {
                                    if save.busy.get_untracked()
                                        || !web_sys::window()
                                            .unwrap()
                                            .confirm_with_message(&if labels {
                                                t("确定删除这个标签？", "Delete this tag?")
                                            } else {
                                                t("确定删除这个分类？", "Delete this category?")
                                            })
                                            .unwrap_or(false)
                                    {
                                        return;
                                    }
                                    deleting.set(Some(id));
                                    deletion.run(
                                        format!("{endpoint}/{id}"),
                                        "DELETE",
                                        None,
                                        move |_| {
                                            if editing.get_untracked() == Some(id) {
                                                reset.run(())
                                            }
                                            for state in [data.value, data.previous] {
                                                state.update(|value| {
                                                    if let Some(rows) =
                                                        value.as_mut().and_then(Value::as_array_mut)
                                                    {
                                                        rows.retain(|row| num(row, "id") != id);
                                                    }
                                                });
                                            }
                                            data.reload();
                                            notify(if labels {
                                                t("标签已删除", "Tag deleted")
                                            } else {
                                                t("分类已删除", "Category deleted")
                                            });
                                        },
                                    );
                                }),
                        ))
                        .class("blog-actions blog-flow"),
                ),
            )),
        )
    };
    View::new(
        blog_interface::html::section()
            .child((
                blog_interface::html::h1()
                    .class("blog-row")
                    .child(if labels {
                        t("标签管理", "Tags")
                    } else {
                        t("分类管理", "Categories")
                    }),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .retry(Some(Callback::new(move |()| data.reload())))
                        .build(),
                ),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(categories.error)
                        .fallback(t("分类加载失败", "Unable to load categories"))
                        .retry(Some(Callback::new(move |()| categories.reload())))
                        .build(),
                ),
                Feedback(component_props_builder(&Feedback).error(save.error).build()),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(deletion.error)
                        .build(),
                ),
                move || {
                    (!validation.get().is_empty()).then(|| {
                        blog_interface::html::p()
                            .class("blog-row blog-danger")
                            .role("alert")
                            .child(validation.get())
                    })
                },
                blog_interface::html::fieldset()
                    .child((
                        blog_interface::html::form()
                            .child((
                                Field(
                                    component_props_builder(&Field)
                                        .label(if labels {
                                            t("标签名称", "Tag name")
                                        } else {
                                            t("分类名称", "Category name")
                                        })
                                        .key("name")
                                        .form(form)
                                        .required(true)
                                        .max(if labels { 32 } else { 64 })
                                        .build(),
                                ),
                                Field(
                                    component_props_builder(&Field)
                                        .label(if labels {
                                            t("标签描述", "Tag description")
                                        } else {
                                            t("分类描述", "Category description")
                                        })
                                        .key("description")
                                        .form(form)
                                        .max(200)
                                        .build(),
                                ),
                                if labels {
                                    View::new(
                                        blog_interface::html::label()
                                            .child((
                                                blog_interface::html::span()
                                                    .class("blog-field-label blog-row")
                                                    .child(t("分类", "Category")),
                                                crate::select::Select(
                                                    component_props_builder(&crate::select::Select)
                                                        .label(Signal::derive(|| {
                                                            t("分类", "Category")
                                                        }))
                                                        .value(Signal::derive(move || {
                                                            num(&form.get(), "sort_id").to_string()
                                                        }))
                                                        .options(Signal::derive(move || {
                                                            std::iter::once((
                                                                "0".to_string(),
                                                                t("选择分类", "Select a category"),
                                                            ))
                                                            .chain(
                                                                list(categories.value.get())
                                                                    .into_iter()
                                                                    .map(|c| {
                                                                        (
                                                                            num(&c, "id")
                                                                                .to_string(),
                                                                            text(&c, "sort_name"),
                                                                        )
                                                                    }),
                                                            )
                                                            .collect()
                                                        }))
                                                        .change(Callback::new(move |v: String| {
                                                            form.update(|f| {
                                                                f["sort_id"] = json!(
                                                                    v.parse::<i64>().unwrap_or(0)
                                                                )
                                                            })
                                                        }))
                                                        .required(true)
                                                        .disabled(Signal::derive(move || {
                                                            categories.value.get().is_none()
                                                        }))
                                                        .build(),
                                                ),
                                            ))
                                            .class("blog-form-field blog-stack"),
                                    )
                                    .into_any()
                                } else {
                                    View::new(
                                        blog_interface::html::label()
                                            .class("blog-form-field blog-stack")
                                            .child((
                                                blog_interface::html::span()
                                                    .class("blog-field-label blog-row")
                                                    .child(t("优先级", "Priority")),
                                                blog_interface::html::input()
                                                    .class("blog-input")
                                                    .node_ref(priority_input)
                                                    .name("priority")
                                                    .r#type("text")
                                                    .inputmode("numeric")
                                                    .prop("value", move || priority.get())
                                                    .on(blog_interface::ev::input, move |ev| {
                                                        priority.set(event_target_value(&ev));
                                                        validation.set(String::new());
                                                    }),
                                            )),
                                    )
                                    .into_any()
                                },
                                blog_interface::html::button()
                                    .child(move || {
                                        if save.busy.get() {
                                            t("保存中…", "Saving…")
                                        } else if editing.get().is_some() {
                                            t("保存修改", "Save changes")
                                        } else if labels {
                                            t("新增标签", "Add tag")
                                        } else {
                                            t("创建分类", "Create category")
                                        }
                                    })
                                    .class("blog-button")
                                    .r#type("submit")
                                    .disabled(move || {
                                        save.busy.get()
                                            || deletion.busy.get()
                                                && deleting.get() == editing.get()
                                    }),
                                move || {
                                    editing.get().map(|_| {
                                        View::new(
                                            blog_interface::html::button()
                                                .child(t("取消编辑", "Cancel editing"))
                                                .class("blog-button")
                                                .r#type("button")
                                                .on(blog_interface::ev::click, move |_| {
                                                    reset.run(())
                                                }),
                                        )
                                    })
                                },
                            ))
                            .class("inline-form blog-frame blog-frame--padded blog-flow blog-form-flow")
                            .on(blog_interface::ev::submit, submit),
                        move || {
                            data.value.get().or_else(|| data.previous.get()).map(|v| {
                                View::new(
                                    blog_interface::html::div()
                                        .child(
                                            blog_interface::html::table()
                                                .child((
                                                    blog_interface::html::thead().child(
                                                        blog_interface::html::tr().child((
                                                            labels.then(|| {
                                                                View::new(
                                                                    blog_interface::html::th()
                                                                        .child("ID"),
                                                                )
                                                            }),
                                                            blog_interface::html::th().child(
                                                                if labels {
                                                                    t("标签", "Tags")
                                                                } else {
                                                                    t("名称", "Name")
                                                                },
                                                            ),
                                                            blog_interface::html::th()
                                                                .child(t("描述", "Description")),
                                                            blog_interface::html::th().child(
                                                                if labels {
                                                                    t("分类", "Category")
                                                                } else {
                                                                    t("排序", "Order")
                                                                },
                                                            ),
                                                            blog_interface::html::th()
                                                                .child(t("操作", "Actions")),
                                                        )),
                                                    ),
                                                    blog_interface::html::tbody().child(
                                                        v.as_array()
                                                            .cloned()
                                                            .unwrap_or_default()
                                                            .into_iter()
                                                            .map(render_row)
                                                            .collect_view(),
                                                    ),
                                                ))
                                                .class("blog-table")
                                                .aria_label(if labels {
                                                    t("标签列表", "Tag list")
                                                } else {
                                                    t("分类列表", "Category list")
                                                }),
                                        )
                                        .class("blog-table-scroll blog-frame")
                                        .tabindex("0")
                                        .role("region")
                                        .aria_label(t("数据表格", "Data table")),
                                )
                            })
                        },
                    ))
                    .class("blog-fieldset blog-stack blog-stack--spaced")
                    .disabled(move || save.busy.get()),
            ))
            .class("blog-content-stack blog-stack"),
    )
}
