use crate::common::{Action, Data, Feedback, flag, list, load, num, t, text, tr};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde_json::{Value, json};

#[component]
pub(super) fn HomeSections() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/home-sections".into()));
    let categories = load(Signal::derive(|| "/api/v1/categories".into()));
    let sections = RwSignal::new(Vec::<Value>::new());
    let loaded = RwSignal::new(false);
    let save = Action::new();
    let validation = RwSignal::new(String::new());
    Effect::new(move |_| {
        if let Some(v) = data.value.get()
            && !loaded.get_untracked()
        {
            sections.set(v.as_array().cloned().unwrap_or_default());
            loaded.set(true);
        }
    });
    let save_sections = Callback::new(move |()| {
        if !loaded.get_untracked() || save.busy.get_untracked() {
            return;
        }
        let payload:Vec<Value>=sections.get_untracked().into_iter().enumerate().map(|(i,v)|json!({"title":text(&v,"title").trim(),"kind":v["kind"],"sort_id":if v["kind"]=="category"{v["sort_id"].clone()}else{Value::Null},"priority":i*10,"enabled":flag(&v,"enabled")})).collect();
        if payload.iter().any(|v| {
            text(v, "title").is_empty() || v["kind"] == "category" && num(v, "sort_id") == 0
        }) {
            validation.set(t(
                "请填写栏目标题并选择分类",
                "Enter a section title and select a category",
            ));
            return;
        }
        validation.set(String::new());
        save.run(
            "/api/v1/content/home-sections".into(),
            "PUT",
            Some(json!(payload)),
            move |v| {
                sections.set(v.as_array().cloned().unwrap_or_default());
                notify(t("首页栏目已保存", "Home sections saved"));
            },
        )
    });
    View::new(
        blog_interface::html::section()
            .child((
                blog_interface::html::h1().class("blog-row").child(t("首页栏目", "Home sections")),
                blog_interface::html::p().class("blog-row").child(t("设置栏目、顺序和展示内容。推荐栏目使用文章的“推荐”状态。", "Set sections, their order, and content. Recommended sections use articles marked as recommended.")),
                blog_interface::html::button()
                    .child(t("保存栏目", "Save sections"))
                    .class("blog-button")
                    .disabled(move || save.busy.get() || !loaded.get())
                    .on(blog_interface::ev::click, move |_| save_sections.run(())),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .fallback(t("首页栏目加载失败", "Unable to load home sections"))
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
                Feedback(
                    component_props_builder(&Feedback)
                        .error(save.error)
                        .fallback(t("保存失败", "Unable to save"))
                        .retry(Some(save_sections))
                        .build(),
                ),
                move || {
                    (!validation.get().is_empty()).then(|| {
                        View::new(
                            blog_interface::html::p().class("blog-row")
                                .child(validation.get())
                                .role("alert"),
                        )
                    })
                },
                blog_interface::html::fieldset()
                    .child((
                        blog_interface::html::div()
                            .child(
                                blog_interface::html::button()
                                    .child(t("新增分类栏目", "Add category section"))
                                    .class("blog-button")
                                    .disabled(move || sections.get().len() >= 12)
                                    .on(blog_interface::ev::click, move |_| {
                                        sections.update(|rows| {
                                            rows.push(json!({ "id":- (js_sys::Date::now() as
                    i64), "title" : t("新栏目", "New section"), "kind" :
                    "category", "sort_id" : list(categories.value.get_untracked())
                    .first().map(| v | v["id"].clone()).unwrap_or(Value::Null),
                    "priority" : rows.len(), "enabled" : true }))
                                        })
                                    }),
                            )
                            .class("blog-actions"),
                        blog_interface::html::div()
                            .child(
                                blog_interface::html::table()
                                    .child((
                                        blog_interface::html::thead().child(
                                            blog_interface::html::tr().child(
                                                [
                                                    t("顺序", "Order"),
                                                    t("标题", "Title"),
                                                    t("内容来源", "Content source"),
                                                    t("分类", "Category"),
                                                    t("显示", "Show"),
                                                    t("操作", "Actions"),
                                                ]
                                                .into_iter()
                                                .map(|l| {
                                                    View::new(blog_interface::html::th().child(l))
                                                })
                                                .collect_view(),
                                            ),
                                        ),
                                        blog_interface::html::tbody().child(For(
                                            component_props_builder(&For)
                                                .each(move || sections.get())
                                                .key(|v| num(v, "id"))
                                                .children(move |item| {
                                                    View::new(HomeRow(
                                                        component_props_builder(&HomeRow)
                                                            .id(num(&item, "id"))
                                                            .sections(sections)
                                                            .categories(categories)
                                                            .build(),
                                                    ))
                                                })
                                                .build(),
                                        )),
                                    ))
                                    .class("blog-table")
                                    .aria_label(t("首页栏目列表", "Home section list")),
                            )
                            .class("blog-table-scroll")
                            .tabindex("0")
                            .role("region")
                            .aria_label(t("数据表格", "Data table")),
                    ))
                    .class("blog-fieldset")
                    .disabled(move || save.busy.get() || !loaded.get()),
            ))
            .class("blog-content-stack"),
    )
}

#[component]
fn HomeRow(id: i64, sections: RwSignal<Vec<Value>>, categories: Data) -> impl IntoView {
    let index = Signal::derive(move || {
        sections
            .get()
            .iter()
            .position(|v| num(v, "id") == id)
            .unwrap_or(0)
    });
    let current = Signal::derive(move || {
        sections
            .get()
            .into_iter()
            .find(|v| num(v, "id") == id)
            .unwrap_or(Value::Null)
    });
    let change = Callback::new(move |(k, v): (&'static str, Value)| {
        sections.update(|rows| {
            if let Some(row) = rows.iter_mut().find(|v| num(v, "id") == id) {
                row[k] = v
            }
        })
    });
    View::new(
        blog_interface::html::tr().child((
            blog_interface::html::td().child(move || index.get() + 1),
            blog_interface::html::td().class("blog-title-cell").child(
                blog_interface::html::label()
                    .child((
                        blog_interface::html::span()
                            .class("blog-field-label blog-row")
                            .child(move || {
                                format!(
                                    "{} {} {}",
                                    t("栏目", "Section"),
                                    index.get() + 1,
                                    t("标题", "title")
                                )
                            }),
                        blog_interface::html::input()
                            .class("blog-input")
                            .maxlength("32")
                            .prop("value", move || text(&current.get(), "title"))
                            .on(blog_interface::ev::input, move |ev| {
                                change.run(("title", json!(event_target_value(&ev))))
                            }),
                    ))
                    .class("blog-form-field"),
            ),
            blog_interface::html::td().child(crate::select::Select(
                component_props_builder(&crate::select::Select)
                    .label(Signal::derive(move || {
                        tr(
                            "栏目 {0} 内容来源",
                            "Section {0} content source",
                            &[(index.get() + 1).to_string()],
                        )
                    }))
                    .value(Signal::derive(move || text(&current.get(), "kind")))
                    .options(Signal::derive(|| {
                        vec![
                            ("latest".into(), t("最新文章", "Latest articles")),
                            ("recommended".into(), t("推荐文章", "Recommended articles")),
                            ("category".into(), t("指定分类", "Selected category")),
                        ]
                    }))
                    .change(Callback::new(move |kind: String| {
                        change.run((
                            "sort_id",
                            if kind == "category" {
                                list(categories.value.get_untracked())
                                    .first()
                                    .map(|v| v["id"].clone())
                                    .unwrap_or(Value::Null)
                            } else {
                                Value::Null
                            },
                        ));
                        change.run(("kind", json!(kind)));
                    }))
                    .build(),
            )),
            blog_interface::html::td().child(move || {
                (text(&current.get(), "kind") == "category").then(|| {
                    View::new(crate::select::Select(
                        component_props_builder(&crate::select::Select)
                            .label(Signal::derive(move || {
                                tr(
                                    "栏目 {0} 分类",
                                    "Section {0} category",
                                    &[(index.get() + 1).to_string()],
                                )
                            }))
                            .value(Signal::derive(move || {
                                num(&current.get(), "sort_id").to_string()
                            }))
                            .options(Signal::derive(move || {
                                std::iter::once((
                                    "0".to_string(),
                                    t("选择分类", "Select a category"),
                                ))
                                .chain(
                                    list(categories.value.get()).into_iter().map(|c| {
                                        (num(&c, "id").to_string(), text(&c, "sort_name"))
                                    }),
                                )
                                .collect()
                            }))
                            .change(Callback::new(move |v: String| {
                                change.run(("sort_id", json!(v.parse::<i64>().unwrap_or(0))))
                            }))
                            .build(),
                    ))
                })
            }),
            blog_interface::html::td().child(
                blog_interface::html::label()
                    .class("blog-check blog-row blog-flow")
                    .child((
                        blog_interface::html::input()
                            .r#type("checkbox")
                            .prop("checked", move || flag(&current.get(), "enabled"))
                            .on(blog_interface::ev::change, move |ev| {
                                change.run(("enabled", json!(event_target_checked(&ev))))
                            }),
                        t(" 启用", " Enable"),
                    )),
            ),
            blog_interface::html::td().child(
                blog_interface::html::div()
                    .child((
                        blog_interface::html::button()
                            .child("↑")
                            .class("blog-button")
                            .aria_label(move || {
                                format!("{}{}", t("上移栏目 ", "Move section up "), index.get() + 1)
                            })
                            .disabled(move || index.get() == 0)
                            .on(blog_interface::ev::click, move |_| {
                                let i = index.get_untracked();
                                sections.update(|r| {
                                    if i > 0 {
                                        r.swap(i, i - 1)
                                    }
                                })
                            }),
                        blog_interface::html::button()
                            .child("↓")
                            .class("blog-button")
                            .aria_label(move || {
                                format!(
                                    "{}{}",
                                    t("下移栏目 ", "Move section down "),
                                    index.get() + 1
                                )
                            })
                            .disabled(move || index.get() + 1 >= sections.get().len())
                            .on(blog_interface::ev::click, move |_| {
                                let i = index.get_untracked();
                                sections.update(|r| {
                                    if i + 1 < r.len() {
                                        r.swap(i, i + 1)
                                    }
                                })
                            }),
                        blog_interface::html::button()
                            .child(t("删除", "Delete"))
                            .class("blog-button blog-danger")
                            .on(blog_interface::ev::click, move |_| {
                                sections.update(|r| r.retain(|v| num(v, "id") != id))
                            }),
                    ))
                    .class("blog-actions"),
            ),
        )),
    )
}
