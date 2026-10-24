use crate::common::{
    Action, Check, Feedback, Field, ListState, Pager, enc, flag, load, num, rows, t, text,
};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::{Value, json};

fn empty_link() -> Value {
    json!({"title":"","classify":"","cover":"","url":"","introduction":"","link_type":"favorites","status":true})
}

#[component]
pub(super) fn Links() -> impl IntoView {
    let page = RwSignal::new(1u64);
    let kind = RwSignal::new(String::new());
    let status = RwSignal::new(-1i64);
    let data = load(Signal::derive(move || {
        format!(
            "/api/v1/content/links?page={}&size=20{}{}",
            page.get(),
            if kind.get().is_empty() {
                String::new()
            } else {
                format!("&kind={}", enc(&kind.get()))
            },
            if status.get() < 0 {
                String::new()
            } else {
                format!("&status={}", status.get())
            }
        )
    }));
    let form = RwSignal::new(empty_link());
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    let remove = Action::new();
    let removing = RwSignal::new(None::<i64>);
    let reset = Callback::new(move |()| {
        form.set(empty_link());
        editing.set(None)
    });
    let options = || {
        vec![
            ("favorites", t("收藏", "Favorites")),
            ("friendUrl", t("友链", "Friend links")),
            ("lovePhoto", t("照片", "Photos")),
            ("funny", t("音乐与趣味", "Music and fun")),
        ]
    };
    let render_row = move |item: Value| {
        let id = num(&item, "id");
        let edit = item.clone();
        View::new(
            blog_interface::html::tr().child((
                blog_interface::html::td()
                    .class("blog-title-cell")
                    .child(text(&item, "title")),
                blog_interface::html::td().child(text(&item, "link_type")),
                blog_interface::html::td().child(if flag(&item, "status") {
                    t("公开", "Public")
                } else {
                    t("待审核/隐藏", "Pending review / hidden")
                }),
                blog_interface::html::td().child(
                    blog_interface::html::div()
                        .child((
                            blog_interface::html::button()
                                .child(t("编辑", "Edit"))
                                .class("blog-button")
                                .disabled(move || remove.busy.get() && removing.get() == Some(id))
                                .on(blog_interface::ev::click, move |_| {
                                    if save.busy.get_untracked()
                                        || remove.busy.get_untracked()
                                            && removing.get_untracked() == Some(id)
                                    {
                                        return;
                                    }
                                    editing.set(Some(id));
                                    let mut f = empty_link();
                                    for k in [
                                        "title",
                                        "classify",
                                        "cover",
                                        "url",
                                        "introduction",
                                        "link_type",
                                    ] {
                                        f[k] = json!(text(&edit, k));
                                    }
                                    f["status"] = json!(flag(&edit, "status"));
                                    form.set(f);
                                }),
                            (text(&item, "link_type") == "friendUrl" && !flag(&item, "status"))
                                .then(|| {
                                    View::new(
                                        blog_interface::html::button()
                                            .child(t("审核通过", "Approve"))
                                            .class("blog-button")
                                            .disabled(move || remove.busy.get())
                                            .on(blog_interface::ev::click, move |_| {
                                                if save.busy.get_untracked()
                                                    || remove.busy.get_untracked()
                                                {
                                                    return;
                                                }
                                                removing.set(Some(id));
                                                remove.run(
                                                    format!("/api/v1/content/links/{id}/status"),
                                                    "PUT",
                                                    Some(json!({ "active" : true })),
                                                    move |_| {
                                                        if editing.get_untracked() == Some(id) {
                                                            form.update(|form| {
                                                                form["status"] = json!(true)
                                                            });
                                                        }
                                                        data.reload();
                                                    },
                                                )
                                            }),
                                    )
                                }),
                            blog_interface::html::button()
                                .child(t("删除", "Delete"))
                                .class("blog-button blog-danger")
                                .disabled(move || remove.busy.get())
                                .on(blog_interface::ev::click, move |_| {
                                    if save.busy.get_untracked() || remove.busy.get_untracked() {
                                        return;
                                    }
                                    if web_sys::window()
                                        .unwrap()
                                        .confirm_with_message(&t(
                                            "确定删除这条资源？",
                                            "Delete this resource?",
                                        ))
                                        .unwrap_or(false)
                                    {
                                        removing.set(Some(id));
                                        remove.run(
                                            format!("/api/v1/content/links/{id}"),
                                            "DELETE",
                                            None,
                                            move |_| {
                                                if editing.get_untracked() == Some(id) {
                                                    reset.run(());
                                                }
                                                data.reload();
                                            },
                                        );
                                    }
                                }),
                        ))
                        .class("blog-actions"),
                ),
            )),
        )
    };
    View::new(
        blog_interface::html::section()
            .child((
                blog_interface::html::h1()
                    .class("blog-row")
                    .child(t("内容资源", "Content resources")),
                blog_interface::html::p().class("blog-row").child(t(
                    "维护友链、收藏、照片和音乐条目。",
                    "Manage friend links, favorites, photos, and music.",
                )),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .retry(Some(Callback::new(move |()| data.reload())))
                        .build(),
                ),
                Feedback(component_props_builder(&Feedback).error(save.error).build()),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(remove.error)
                        .build(),
                ),
                blog_interface::html::div()
                    .child((
                        blog_interface::html::label()
                            .child((
                                blog_interface::html::span()
                                    .class("blog-field-label blog-row")
                                    .child(t("类型筛选", "Filter by type")),
                                crate::select::Select(
                                    component_props_builder(&crate::select::Select)
                                        .label(Signal::derive(|| t("类型筛选", "Filter by type")))
                                        .value(Signal::derive(move || kind.get()))
                                        .options(Signal::derive(move || {
                                            std::iter::once((String::new(), t("全部", "All")))
                                                .chain(
                                                    options()
                                                        .into_iter()
                                                        .map(|(k, l)| (k.to_string(), l)),
                                                )
                                                .collect()
                                        }))
                                        .change(Callback::new(move |v: String| {
                                            kind.set(v);
                                            page.set(1);
                                        }))
                                        .build(),
                                ),
                            ))
                            .class("blog-form-field"),
                        blog_interface::html::label()
                            .child((
                                blog_interface::html::span()
                                    .class("blog-field-label blog-row")
                                    .child(t("状态筛选", "Filter by status")),
                                crate::select::Select(
                                    component_props_builder(&crate::select::Select)
                                        .label(Signal::derive(|| t("状态筛选", "Filter by status")))
                                        .value(Signal::derive(move || status.get().to_string()))
                                        .options(Signal::derive(|| {
                                            vec![
                                                ("-1".into(), t("全部", "All")),
                                                (
                                                    "0".into(),
                                                    t("待审核/隐藏", "Pending review / hidden"),
                                                ),
                                                ("1".into(), t("已公开", "Published")),
                                            ]
                                        }))
                                        .change(Callback::new(move |v: String| {
                                            status.set(v.parse().unwrap_or(-1));
                                            page.set(1);
                                        }))
                                        .build(),
                                ),
                            ))
                            .class("blog-form-field"),
                    ))
                    .class("inline-form"),
                blog_interface::html::fieldset()
                    .child((
                        blog_interface::html::form()
                            .child(
                                blog_interface::html::fieldset()
                                    .child((
                                        blog_interface::html::h2().class("blog-row").child(
                                            move || {
                                                if editing.get().is_some() {
                                                    t("编辑资源", "Edit resource")
                                                } else {
                                                    t("新增资源", "Add resource")
                                                }
                                            },
                                        ),
                                        blog_interface::html::div()
                                            .child((
                                                blog_interface::html::label()
                                                    .child((
                                                        blog_interface::html::span()
                                                            .class("blog-field-label blog-row")
                                                            .child(t("类型", "Type")),
                                                        crate::select::Select(
                                                            component_props_builder(
                                                                &crate::select::Select,
                                                            )
                                                            .label(Signal::derive(|| {
                                                                t("类型", "Type")
                                                            }))
                                                            .value(Signal::derive(move || {
                                                                text(&form.get(), "link_type")
                                                            }))
                                                            .options(Signal::derive(move || {
                                                                options()
                                                                    .into_iter()
                                                                    .map(|(k, l)| {
                                                                        (k.to_string(), l)
                                                                    })
                                                                    .collect()
                                                            }))
                                                            .change(Callback::new(
                                                                move |v: String| {
                                                                    form.update(|f| {
                                                                        f["link_type"] = json!(v)
                                                                    })
                                                                },
                                                            ))
                                                            .build(),
                                                        ),
                                                    ))
                                                    .class("blog-form-field"),
                                                Field(
                                                    component_props_builder(&Field)
                                                        .label(t("分类", "Category"))
                                                        .key("classify")
                                                        .form(form)
                                                        .max(32)
                                                        .build(),
                                                ),
                                            ))
                                            .class("form-grid"),
                                        Field(
                                            component_props_builder(&Field)
                                                .label(t("标题", "Title"))
                                                .key("title")
                                                .form(form)
                                                .required(true)
                                                .max(64)
                                                .build(),
                                        ),
                                        Field(
                                            component_props_builder(&Field)
                                                .label(t("图片地址", "Image URL"))
                                                .key("cover")
                                                .form(form)
                                                .build(),
                                        ),
                                        Field(
                                            component_props_builder(&Field)
                                                .label(t("目标地址", "Target URL"))
                                                .key("url")
                                                .form(form)
                                                .build(),
                                        ),
                                        Field(
                                            component_props_builder(&Field)
                                                .label(t("简介", "Introduction"))
                                                .key("introduction")
                                                .form(form)
                                                .build(),
                                        ),
                                        Check(
                                            component_props_builder(&Check)
                                                .label(t(" 公开显示", " Show publicly"))
                                                .key("status")
                                                .form(form)
                                                .build(),
                                        ),
                                        blog_interface::html::div()
                                            .child((
                                                blog_interface::html::button()
                                                    .child(move || {
                                                        if editing.get().is_some() {
                                                            t("保存修改", "Save changes")
                                                        } else {
                                                            t("新增资源", "Add resource")
                                                        }
                                                    })
                                                    .class("blog-button")
                                                    .r#type("submit")
                                                    .disabled(move || remove.busy.get()),
                                                move || {
                                                    editing.get().map(|_| {
                                                        View::new(
                                                            blog_interface::html::button()
                                                                .child(t(
                                                                    "取消编辑",
                                                                    "Cancel editing",
                                                                ))
                                                                .class("blog-button")
                                                                .r#type("button")
                                                                .on(
                                                                    blog_interface::ev::click,
                                                                    move |_| reset.run(()),
                                                                ),
                                                        )
                                                    })
                                                },
                                            ))
                                            .class("blog-actions"),
                                    ))
                                    .class("blog-fieldset")
                                    .disabled(move || {
                                        remove.busy.get() && removing.get() == editing.get()
                                    }),
                            )
                            .class("editor-form")
                            .on(blog_interface::ev::submit, move |ev| {
                                ev.prevent_default();
                                if save.busy.get_untracked() || remove.busy.get_untracked() {
                                    return;
                                }
                                let mut f = form.get_untracked();
                                for k in ["classify", "cover", "url", "introduction"] {
                                    if text(&f, k).is_empty() {
                                        f[k] = Value::Null;
                                    }
                                }
                                let path = editing
                                    .get_untracked()
                                    .map(|id| format!("/api/v1/content/links/{id}"))
                                    .unwrap_or("/api/v1/content/links".into());
                                save.run(
                                    path,
                                    if editing.get_untracked().is_some() {
                                        "PUT"
                                    } else {
                                        "POST"
                                    },
                                    Some(f),
                                    move |_| {
                                        reset.run(());
                                        data.reload();
                                        notify(t("资源已保存", "Resource saved"));
                                    },
                                );
                            }),
                        move || {
                            data.value.get().map(|v| {
                                View::new(
                                    blog_interface::html::div()
                                        .child(
                                            blog_interface::html::table()
                                                .child((
                                                    blog_interface::html::thead().child(
                                                        blog_interface::html::tr().child((
                                                            blog_interface::html::th()
                                                                .child(t("标题", "Title")),
                                                            blog_interface::html::th()
                                                                .child(t("类型", "Type")),
                                                            blog_interface::html::th()
                                                                .child(t("状态", "Status")),
                                                            blog_interface::html::th()
                                                                .child(t("操作", "Actions")),
                                                        )),
                                                    ),
                                                    blog_interface::html::tbody().child(
                                                        rows(&v, "items")
                                                            .into_iter()
                                                            .map(render_row)
                                                            .collect_view(),
                                                    ),
                                                ))
                                                .class("blog-table")
                                                .aria_label(t(
                                                    "内容资源列表",
                                                    "Content resource list",
                                                )),
                                        )
                                        .class("blog-table-scroll")
                                        .tabindex("0")
                                        .role("region")
                                        .aria_label(t("数据表格", "Data table")),
                                )
                            })
                        },
                        ListState(
                            component_props_builder(&ListState)
                                .data(data)
                                .empty(t("暂无内容资源", "No content resources yet"))
                                .build(),
                        ),
                        Pager(
                            component_props_builder(&Pager)
                                .data(data)
                                .page(page)
                                .build(),
                        ),
                    ))
                    .class("blog-fieldset")
                    .disabled(move || save.busy.get()),
            ))
            .class("blog-content-stack"),
    )
}
