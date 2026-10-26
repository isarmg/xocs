use crate::common::{Action, Data, Feedback, list, load, load_typed, num, t, text, tr};
use crate::models::{HomeSection, HomeSource};
use blog_interface::component::component_props_builder;
use blog_interface::components::notify;
use blog_interface::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Clone)]
struct SectionRow {
    id: i64,
    form: ArcRwSignal<SectionForm>,
}

#[derive(Clone)]
struct SectionForm {
    title: String,
    source: SectionSource,
    enabled: bool,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum SectionSource {
    Latest,
    Recommended,
    Category { sort_id: Option<i64> },
}

impl SectionSource {
    fn kind(self) -> &'static str {
        match self {
            Self::Latest => "latest",
            Self::Recommended => "recommended",
            Self::Category { .. } => "category",
        }
    }
}

impl From<HomeSection> for SectionRow {
    fn from(section: HomeSection) -> Self {
        Self {
            id: section.id,
            form: ArcRwSignal::new(SectionForm {
                title: section.title,
                source: match section.source {
                    HomeSource::Latest => SectionSource::Latest,
                    HomeSource::Recommended => SectionSource::Recommended,
                    HomeSource::Category { sort_id } => SectionSource::Category {
                        sort_id: Some(sort_id),
                    },
                },
                enabled: section.enabled,
            }),
        }
    }
}

#[component]
pub(super) fn HomeSections() -> impl IntoView {
    let data =
        load_typed::<Vec<HomeSection>>(Signal::derive(|| "/api/v1/content/home-sections".into()));
    let categories = load(Signal::derive(|| "/api/v1/categories".into()));
    let sections = RwSignal::new(Vec::<SectionRow>::new());
    let loaded = RwSignal::new(false);
    let next_id = RwSignal::new(-1i64);
    let save = Action::new();
    let validation = RwSignal::new(String::new());
    Effect::new(move |_| {
        if let Some(v) = data.value.get()
            && !loaded.get_untracked()
        {
            sections.set(v.into_iter().map(SectionRow::from).collect());
            loaded.set(true);
        }
    });
    let save_sections = Callback::new(move |()| {
        if !loaded.get_untracked() || save.busy.get_untracked() {
            return;
        }
        if sections.with_untracked(|rows| {
            rows.iter().any(|row| {
                row.form.with_untracked(|form| {
                    form.title.trim().is_empty()
                        || matches!(form.source, SectionSource::Category { sort_id: None })
                })
            })
        }) {
            validation.set(t(
                "请填写栏目标题并选择分类",
                "Enter a section title and select a category",
            ));
            return;
        }
        validation.set(String::new());
        let payload: Vec<_> = sections.with_untracked(|rows| {
            rows.iter()
                .enumerate()
                .map(|(index, row)| {
                    row.form.with_untracked(|form| {
                        let sort_id = match form.source {
                            SectionSource::Category { sort_id } => sort_id,
                            SectionSource::Latest | SectionSource::Recommended => None,
                        };
                        json!({
                            "title": form.title.trim(),
                            "kind": form.source.kind(),
                            "sort_id": sort_id,
                            "priority": index * 10,
                            "enabled": form.enabled,
                        })
                    })
                })
                .collect()
        });
        save.run_typed(
            "/api/v1/content/home-sections".into(),
            "PUT",
            Some(json!(payload)),
            move |v: Vec<HomeSection>| {
                sections.update(|rows| {
                    *rows = v
                        .into_iter()
                        .map(|section| {
                            let next = SectionRow::from(section);
                            match rows.iter().find(|row| row.id == next.id) {
                                Some(row) => {
                                    row.form.set(next.form.get_untracked());
                                    row.clone()
                                }
                                None => next,
                            }
                        })
                        .collect();
                });
                notify(t("首页栏目已保存", "Home sections saved"));
            },
        )
    });
    View::new(
        blog_interface::html::section()
            .aria_label(t("首页栏目", "Home sections"))
            .child((
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
                            blog_interface::html::p().class("blog-row blog-danger")
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
                                        let id = next_id.get_untracked();
                                        next_id.update(|id| *id -= 1);
                                        sections.update(|rows| {
                                            rows.push(SectionRow {
                                                id,
                                                form: ArcRwSignal::new(SectionForm {
                                                    title: t("新栏目", "New section"),
                                                    source: SectionSource::Category {
                                                        sort_id: list(categories.value.get_untracked())
                                                            .first().map(|category| num(category, "id")),
                                                    },
                                                    enabled: true,
                                                }),
                                            })
                                        })
                                    }),
                            )
                            .class("blog-actions blog-flow"),
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
                                                .key(|row| row.id)
                                                .children(move |item| {
                                                    View::new(HomeRow(
                                                        component_props_builder(&HomeRow)
                                                            .item(item)
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
                    .class("blog-fieldset blog-frame blog-frame--padded blog-stack")
                    .disabled(move || save.busy.get() || !loaded.get()),
            ))
            .class("blog-content-stack blog-stack"),
    )
}

#[component]
fn HomeRow(
    item: SectionRow,
    sections: RwSignal<Vec<SectionRow>>,
    categories: Data,
) -> impl IntoView {
    let id = item.id;
    let current = RwSignal::from(item.form);
    let index = Signal::derive(move || {
        // A removed keyed row can be read before its owner is disposed.
        sections
            .with(|rows| rows.iter().position(|row| row.id == id))
            .unwrap_or(0)
    });
    View::new(
        blog_interface::html::tr().child((
            blog_interface::html::td().child(move || index.get() + 1),
            blog_interface::html::td().class("blog-title-cell").child(
                blog_interface::html::label()
                    .child((
                        blog_interface::html::span()
                            .class("blog-visually-hidden")
                            .child(move || {
                                format!(
                                    "{} {} {}",
                                    t("栏目", "Section"),
                                    index.get() + 1,
                                    t("标题", "title")
                                )
                            }),
                        blog_interface::html::input()
                            .class("blog-input blog-input--compact")
                            .aria_label(move || {
                                format!(
                                    "{} {} {}",
                                    t("栏目", "Section"),
                                    index.get() + 1,
                                    t("标题", "title")
                                )
                            })
                            .maxlength("32")
                            .prop("value", move || current.with(|form| form.title.clone()))
                            .on(blog_interface::ev::input, move |ev| {
                                current.update(|form| form.title = event_target_value(&ev))
                            }),
                    ))
                    .class("blog-form-field blog-stack"),
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
                    .value(Signal::derive(move || {
                        current.with(|form| form.source.kind().to_string())
                    }))
                    .options(Signal::derive(|| {
                        vec![
                            ("latest".into(), t("最新文章", "Latest articles")),
                            ("recommended".into(), t("推荐文章", "Recommended articles")),
                            ("category".into(), t("指定分类", "Selected category")),
                        ]
                    }))
                    .change(Callback::new(move |kind: String| {
                        let source = serde_json::from_value::<SectionSource>(json!({"kind": kind}))
                            .expect("invariant: source selector emits its declared options");
                        let source = match source {
                            SectionSource::Category { .. } => SectionSource::Category {
                                sort_id: list(categories.value.get_untracked())
                                    .first()
                                    .map(|category| num(category, "id")),
                            },
                            SectionSource::Latest | SectionSource::Recommended => source,
                        };
                        current.update(|form| form.source = source);
                    }))
                    .build(),
            )),
            blog_interface::html::td().child(move || {
                matches!(
                    current.with(|form| form.source),
                    SectionSource::Category { .. }
                )
                .then(|| {
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
                                match current.with(|form| form.source) {
                                    SectionSource::Category { sort_id } => {
                                        sort_id.map_or_else(|| "0".into(), |id| id.to_string())
                                    }
                                    SectionSource::Latest | SectionSource::Recommended => {
                                        "0".into()
                                    }
                                }
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
                                let id = v
                                    .parse::<i64>()
                                    .expect("invariant: category selector values are integer IDs");
                                current.update(|form| {
                                    form.source = SectionSource::Category {
                                        sort_id: (id != 0).then_some(id),
                                    };
                                });
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
                            .prop("checked", move || current.with(|form| form.enabled))
                            .on(blog_interface::ev::change, move |ev| {
                                current.update(|form| form.enabled = event_target_checked(&ev))
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
                                sections.update(|r| r.retain(|row| row.id != id))
                            }),
                    ))
                    .class("blog-actions blog-flow"),
            ),
        )),
    )
}
