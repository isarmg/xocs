use crate::common::Action;
use crate::common::*;
use crate::models::{AdminStats, ArticleSummary, Page};
use leptos::component::{component_props_builder, component_view};
use leptos::prelude::*;
use serde_json::{Value, json};
use xcss_web::components::{AccountPage, AdminShell, HeaderActions, HeaderNavigation, notify};
#[derive(Clone)]
struct Group {
    id: &'static str,
    label: String,
    title: String,
    pages: Vec<(&'static str, String)>,
}
fn groups() -> Vec<Group> {
    vec![
        Group {
            id: "overview",
            label: t("总览", "Overview"),
            title: t("总览", "Overview"),
            pages: vec![("dashboard", t("总览", "Overview"))],
        },
        Group {
            id: "content",
            label: t("内容", "Content"),
            title: t("内容管理", "Content management"),
            pages: vec![
                ("articles", t("文章管理", "Articles")),
                ("categories", t("分类管理", "Categories")),
                ("labels", t("标签管理", "Tags")),
                ("notes", t("随笔管理", "Notes")),
                ("letter", t("信笺管理", "Letter")),
            ],
        },
        Group {
            id: "community",
            label: t("社区", "Community"),
            title: t("社区管理", "Community management"),
            pages: vec![
                ("comments", t("评论管理", "Comments")),
                ("tree-hole", t("留言板管理", "Message board")),
            ],
        },
        Group {
            id: "resources",
            label: t("资源", "Resources"),
            title: t("资源管理", "Resource management"),
            pages: vec![
                ("resources", t("文件管理", "Files")),
                ("links", t("内容资源", "Content resources")),
                ("family", t("恋爱笔记", "Love journal")),
            ],
        },
        Group {
            id: "settings",
            label: t("设置", "Settings"),
            title: t("网站设置", "Site settings"),
            pages: vec![
                ("site", t("网站设置", "Site settings")),
                ("home", t("首页栏目", "Home sections")),
            ],
        },
    ]
}
#[component]
pub fn Admin() -> impl IntoView {
    View::new(
        leptos::html::div()
            .child(
                #[allow(unused_braces)]
                {
                    {
                        #[allow(unreachable_code)]
                        #[allow(unused_mut)]
                        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                        component_view(
                            #[allow(clippy::needless_borrows_for_generic_args)]
                            &AdminShell,
                            {
                                let mut props = component_props_builder(&AdminShell)
                                    .product(
                                        #[allow(unused_braces)]
                                        {
                                            "xocs"
                                        },
                                    )
                                    .landing(
                                        #[allow(unused_braces)]
                                        {
                                            "/admin#articles"
                                        },
                                    )
                                    .children({
                                        ::leptos::children::ToChildren::to_children(move || {
                                            #[allow(unreachable_code)]
                                            #[allow(unused_mut)]
                                            #[allow(
                                                clippy::let_and_return,
                                                clippy::unit_arg,
                                                clippy::let_unit_value
                                            )]
                                            component_view(
                                                #[allow(clippy::needless_borrows_for_generic_args)]
                                                &Workspace,
                                                {
                                                    let mut props =
                                                        component_props_builder(&Workspace).build();
                                                    props
                                                },
                                            )
                                        })
                                    })
                                    .build();
                                props
                            },
                        )
                    }
                },
            )
            .class("xocs-admin-app"),
    )
}
#[component]
fn Workspace() -> impl IntoView {
    let route = hash_signal();
    let active = Signal::derive(move || {
        let r = route.get();
        if r == "new" || r.starts_with("edit/") || r.is_empty() {
            "articles".into()
        } else {
            r
        }
    });
    let group = Signal::derive(move || {
        groups()
            .into_iter()
            .find(|g| g.pages.iter().any(|(p, _)| *p == active.get()))
            .map(|g| g.id)
            .unwrap_or("content")
    });
    View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & HeaderNavigation, { let mut props =
                    component_props_builder(& HeaderNavigation)
                    .label(#[allow(unused_braces)] { t("后台导航",
                    "Administration navigation") }).children({
                    ::leptos::children::ToChildren::to_children(move ||
                    IntoRender::into_render({ groups().into_iter()
                    .map(| g | View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ g.label }) },
        )
        .class("xcss-button")
        .aria_pressed(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                move || {
                    (group.get() == g.id && active.get() != "account").to_string()
                },
            ),
        )
        .on(leptos::ev::click, move |_| navigate(g.pages[0].0)),
)).collect_view() })) }).build(); props })
                    }), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & HeaderActions, { let mut props =
                    component_props_builder(& HeaderActions)
                    .children({ ::leptos::children::ToChildren::to_children(move ||
                    leptos::html::a()
                    .child(#[allow(unused_braces)] { leptos::svg::svg()
                    .child(#[allow(unused_braces)] { leptos::svg::path()
                    .attr("d", "m3 10 9-7 9 7M5 9v11h5v-7h4v7h5V9") })
                    .attr("aria-hidden", "true").attr("width", "1em").attr("height",
                    "1em").attr("viewBox", "0 0 24 24").attr("fill", "none")
                    .attr("stroke", "currentColor").attr("stroke-width", "1.7") })
                    .class("xocs-site-link")
                    .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(href("/")))
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("返回网站",
                    "Back to site")))
                    .title(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("返回网站",
                    "Back to site")))) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | groups().into_iter().find(| g | g.id == group.get() && g
                    .pages.len() > 1 && active.get() != "account").map(| g | View::new(
    leptos::html::nav()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    g.pages
                        .into_iter()
                        .map(|(id, label)| {
                            View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render(label) },
        )
        .class("xcss-button")
        .aria_pressed(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                move || {
                    (active.get() == id).to_string()
                },
            ),
        )
        .on(leptos::ev::click, move |_| navigate(id)),
)
                        })
                        .collect_view()
                })
            },
        )
        .class("xcss-secondary-navigation")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(g.title),
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | { let route = route.get(); match route.as_str() {
                    "account" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &AccountPage,
        {
            let mut props = component_props_builder(
                    &AccountPage,
                )
                .build();
            props
        },
    )
}) .into_any(), "dashboard"
                    => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Dashboard,
        {
            let mut props = component_props_builder(&Dashboard)
                .build();
            props
        },
    )
}) .into_any(), "categories" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Taxonomy,
        {
            let mut props = component_props_builder(&Taxonomy)
                .labels(#[allow(unused_braces)] { false })
                .build();
            props
        },
    )
}) .into_any(), "labels" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Taxonomy,
        {
            let mut props = component_props_builder(&Taxonomy)
                .labels(#[allow(unused_braces)] { true })
                .build();
            props
        },
    )
}) .into_any(), "comments" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Moderation,
        {
            let mut props = component_props_builder(&Moderation)
                .kind(#[allow(unused_braces)] { "comments" })
                .build();
            props
        },
    )
}) .into_any(), "tree-hole" =>
                    View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Moderation,
        {
            let mut props = component_props_builder(&Moderation)
                .kind(#[allow(unused_braces)] { "tree-hole" })
                .build();
            props
        },
    )
}) .into_any(),
                    "resources" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Moderation,
        {
            let mut props = component_props_builder(&Moderation)
                .kind(#[allow(unused_braces)] { "resources" })
                .build();
            props
        },
    )
})
                    .into_any(), "site" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &SiteSettings,
        {
            let mut props = component_props_builder(
                    &SiteSettings,
                )
                .build();
            props
        },
    )
}) .into_any(),
                    "letter" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &LetterSettings,
        {
            let mut props = component_props_builder(
                    &LetterSettings,
                )
                .build();
            props
        },
    )
}) .into_any(), "family"
                    => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &FamilySettings,
        {
            let mut props = component_props_builder(
                    &FamilySettings,
                )
                .build();
            props
        },
    )
}) .into_any(), "home" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &HomeSections,
        {
            let mut props = component_props_builder(
                    &HomeSections,
                )
                .build();
            props
        },
    )
}) .into_any(), "links" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Links,
        {
            let mut props = component_props_builder(&Links)
                .build();
            props
        },
    )
})
                    .into_any(), "notes" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &crate::editor::Notes,
        {
            let mut props = component_props_builder(
                    &crate::editor::Notes,
                )
                .build();
            props
        },
    )
})
                    .into_any(), "new" => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &crate::editor::ArticleEditor,
        {
            let mut props = component_props_builder(
                    &crate::editor::ArticleEditor,
                )
                .build();
            props
        },
    )
}) .into_any(), s if s.starts_with("edit/") => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &crate::editor::ArticleEditor,
        {
            let mut props = component_props_builder(
                    &crate::editor::ArticleEditor,
                )
                .id(
                    #[allow(unused_braces)]
                    { { s[5..].parse::<i64>().unwrap_or(0) } },
                )
                .build();
            props
        },
    )
}) .into_any(), _ => View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Articles,
        {
            let mut props = component_props_builder(&Articles)
                .build();
            props
        },
    )
})
                    .into_any() } } }))
                ,),
        )
        .class("xocs-admin-workspace xcss-content-stack"),
)
}
#[component]
fn Dashboard() -> impl IntoView {
    let data = load_typed::<AdminStats>(Signal::derive(|| "/api/v1/content/statistics".into()));
    View::new(
        leptos::html::section()
            .child((
                IntoMaybeErased::into_maybe_erased(leptos::html::h1().child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({ t("总览", "Overview") })
                    },
                )),
                IntoMaybeErased::into_maybe_erased(leptos::html::p().child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            t("网站内容与社区状态。", "Site content and community status.")
                        })
                    },
                )),
                IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)]
                    #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Feedback,
                        {
                            let mut props = component_props_builder(&Feedback)
                                .error(
                                    #[allow(unused_braces)]
                                    {
                                        data.error
                                    },
                                )
                                .fallback(
                                    #[allow(unused_braces)]
                                    {
                                        t("统计信息加载失败", "Unable to load statistics")
                                    },
                                )
                                .retry(
                                    #[allow(unused_braces)]
                                    {
                                        Some(Callback::new(move |()| data.reload()))
                                    },
                                )
                                .build();
                            props
                        },
                    )
                }),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move || {
                        data.value.get().map(|v| {
                            View::new(
                                leptos::html::div()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            leptos::html::table()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(leptos::html::thead()
                                .child(#[allow(unused_braces)] {
                                leptos::html::tr()
                                .child((IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("指标",
                                "Metric") }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("数量",
                                "Count") }) })),)) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::tbody()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ [(v.articles,
                                t("文章", "Articles")), (v.comments, t("评论",
                                "Comments")), (v.members, t("用户", "Users")), (v
                                .categories, t("分类", "Categories"))].into_iter().map(|
                                (k, l) | View::new(
    leptos::html::tr()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::th()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ l }) })
                    .scope("row")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ k }) }))
                ,),
        ),
)).collect_view() }) }))
                            ,),
                    )
                    .class("xcss-table xcss-statistics-table")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("网站统计", "Site statistics"),
                        ),
                    )
                                        },
                                    )
                                    .class("xcss-table-scroll")
                                    .tabindex("0")
                                    .role("region")
                                    .aria_label(
                                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                            t("数据表格", "Data table"),
                                        ),
                                    ),
                            )
                        })
                    }
                })),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    groups()
                                        .into_iter()
                                        .filter(|g| g.id != "overview")
                                        .map(|g| {
                                            View::new(
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ g.title.clone() })
                    })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::nav()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ g.pages.into_iter()
                    .map(| (id, l) | View::new(
    leptos::html::a()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ l }) },
        )
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                href(&format!("/admin#{id}")),
            ),
        ),
)).collect_view() }) })
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(g
                    .title)))
                ,),
        )
        .class("xcss-content-panel"),
)
                                        })
                                        .collect_view()
                                })
                            },
                        )
                        .class("xocs-dashboard-sections"),
                ),
            ))
            .class("xcss-content-stack"),
    )
}
#[component]
fn Articles() -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load_typed::<Page<ArticleSummary>>(Signal::derive(move || {
        format!("/api/v1/content/articles?page={}&size=15", page.get())
    }));
    let action = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            let last = ((v.total + v.size - 1) / v.size.max(1)).max(1) as u64;
            if page.get_untracked() > last {
                page.set(last);
            }
        }
    });
    View::new(
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & HeaderActions, { let mut props =
                    component_props_builder(& HeaderActions)
                    .children({ ::leptos::children::ToChildren::to_children(move ||
                    (IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] { "+" }).class("xcss-button")
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("新增文章",
                    "Add article")))
                    .title(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("新增文章",
                    "Add article"))).on(leptos::ev::click, move |
                    _ | navigate("new"))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] { "↻" }).class("xcss-button")
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("刷新",
                    "Refresh")))
                    .title(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("刷新",
                    "Refresh")))
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | data.busy.get())).on(leptos::ev::click,
                    move | _ | data.reload())),)) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("文章管理",
                    "Articles") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("管理公开文章与草稿。",
                    "Manage published articles and drafts.") }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .fallback(#[allow(unused_braces)] { t("文章加载失败",
                    "Unable to load articles") }).retry(#[allow(unused_braces)] {
                    Some(Callback::new(move | () | data.reload())) }).build(); props
                    }) }), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { action.error })
                    .fallback(#[allow(unused_braces)] {
                    t("删除失败，请刷新后重试",
                    "Delete failed. Refresh and try again.") }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.value.get().map(| v | View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::table()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(leptos::html::thead()
                                .child(#[allow(unused_braces)] {
                                leptos::html::tr()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ [t("标题",
                                "Title"), t("状态", "Status"), t("浏览", "Views"),
                                t("创建时间", "Created at"), t("操作", "Actions")]
                                .into_iter().map(| l | View::new(
    leptos::html::th()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ l }) },
        ),
))
                                .collect_view() }) }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::tbody()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ v.items
                                .into_iter().map(| a | { let id = a.id; View::new(
    leptos::html::tr()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ a.article_title })
                    })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ format!("#{id}") })
                    }).class("muted")),))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if a.view_status {
                    t("公开", "Public") } else { t("加密", "Password protected")
                    } }) }).class(if a.view_status { "badge success" } else { "badge"
                    }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ a.view_count }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ crate
                    ::dates::format(& a.create_time.unwrap_or_default()) }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("编辑", "Edit")
                    }) }).class("xcss-button")
                    .on(leptos::ev::click, move | _ | navigate(&
                    format!("edit/{id}")))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("删除",
                    "Delete") }) }).class("xcss-button xcss-danger")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | action.busy.get())).on(leptos::ev::click,
                    move | _ | { if web_sys::window().unwrap().confirm_with_message(&
                    t("确定删除这篇文章？", "Delete this article?"))
                    .unwrap_or(false) { action
                    .run(format!("/api/v1/content/articles/{id}"), "DELETE", None,
                    move | _ | { notify(t("文章已删除", "Article deleted"));
                    data.reload(); }); } })),)).class("xcss-actions") }))
                ,),
        ),
) })
                                .collect_view() }) }))
                            ,),
                    )
                    .class("xcss-table")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("文章列表", "Article list"),
                        ),
                    )
            },
        )
        .class("xcss-table-scroll")
        .tabindex("0")
        .role("region")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("数据表格", "Data table"),
            ),
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Pager, { let mut props =
                    component_props_builder(& Pager)
                    .data(#[allow(unused_braces)] { data })
                    .page(#[allow(unused_braces)] { page }).build(); props }) })
                ,),
        )
        .class("xcss-content-stack"),
)
}
#[component]
fn Taxonomy(labels: bool) -> impl IntoView {
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
    let save = Action::new();
    let deletion = Action::new();
    let deleting = RwSignal::new(None::<i64>);
    let reset = Callback::new(move |()| {
        editing.set(None);
        form.set(json!({"name":"","description":"","priority":0,"sort_id":0}));
    });
    View::new(
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if labels {
                    t("标签管理", "Tags") } else { t("分类管理",
                    "Categories") } }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .retry(#[allow(unused_braces)] { Some(Callback::new(move | () |
                    data.reload())) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { categories.error })
                    .fallback(#[allow(unused_braces)] { t("分类加载失败",
                    "Unable to load categories") }).retry(#[allow(unused_braces)] {
                    Some(Callback::new(move | () | categories.reload())) }).build();
                    props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { save.error }).build(); props })
                    }), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { deletion.error }).build(); props
                    }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::fieldset()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::form()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { if labels { t("标签名称",
                    "Tag name") } else { t("分类名称", "Category name") } })
                    .key(#[allow(unused_braces)] { "name" })
                    .form(#[allow(unused_braces)] { form })
                    .required(#[allow(unused_braces)] { true })
                    .max(#[allow(unused_braces)] { if labels { 32 } else { 64 } })
                    .build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { if labels { t("标签描述",
                    "Tag description") } else { t("分类描述",
                    "Category description") } }).key(#[allow(unused_braces)] {
                    "description" }).form(#[allow(unused_braces)] { form })
                    .max(#[allow(unused_braces)] { 200 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    if labels { View::new(
    leptos::html::label()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("分类", "Category") })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & crate ::select::Select, { let mut props =
                    component_props_builder(& crate
                    ::select::Select).label(#[allow(unused_braces)] {
                    Signal::derive(| | t("分类", "Category")) })
                    .value(#[allow(unused_braces)] { Signal::derive(move | | num(&
                    form.get(), "sort_id").to_string()) })
                    .options(#[allow(unused_braces)] { Signal::derive(move | |
                    std::iter::once(("0".to_string(), t("选择分类",
                    "Select a category"))).chain(list(categories.value.get())
                    .into_iter().map(| c | (num(& c, "id").to_string(), text(& c,
                    "sort_name")))).collect()) }).change(#[allow(unused_braces)] {
                    Callback::new(move | v : String | form.update(| f | f["sort_id"]
                    = json!(v.parse::< i64 > ().unwrap_or(0)))) })
                    .required(#[allow(unused_braces)] { true })
                    .disabled(#[allow(unused_braces)] { Signal::derive(move | |
                    categories.value.get().is_none()) }).build(); props }) })
                ,),
        )
        .class("xcss-form-field"),
) .into_any() } else { View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Field,
        {
            let mut props = component_props_builder(&Field)
                .label(#[allow(unused_braces)] { t("优先级", "Priority") })
                .key(#[allow(unused_braces)] { "priority" })
                .form(#[allow(unused_braces)] { form })
                .kind(#[allow(unused_braces)] { "number" })
                .build();
            props
        },
    )
}) .into_any() } })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if save.busy.get() { t("保存中…", "Saving…") }
                    else if editing.get().is_some() { t("保存修改",
                    "Save changes") } else if labels { t("新增标签", "Add tag") }
                    else { t("创建分类", "Create category") } }) })
                    .class("xcss-button").r#type("submit")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || deletion.busy.get() && deleting.get() ==
                    editing.get()))),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | editing.get().map(| _ | View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("取消编辑", "Cancel editing")
                })
            },
        )
        .class("xcss-button")
        .r#type("button")
        .on(leptos::ev::click, move |_| reset.run(())),
))
                    })),)).class("inline-form")
                    .on(leptos::ev::submit, move | ev | { ev
                    .prevent_default(); let current = form.get_untracked(); if
                    deletion.busy.get_untracked() && deleting.get_untracked() ==
                    editing.get_untracked() { return; }
                    if labels && !
                    list(categories.value.get_untracked()).iter().any(| c | num(c,
                    "id") == num(& current, "sort_id")) { return; } let mut body =
                    json!({ "name" : current["name"], "description" : if text(&
                    current, "description").is_empty() { Value::Null } else {
                    current["description"].clone() } }); let order = if labels {
                    "sort_id" } else { "priority" }; body[order] = current[order]
                    .clone(); let path = editing.get_untracked().map(| id |
                    format!("{endpoint}/{id}")).unwrap_or(endpoint.into()); save
                    .run(path, if editing.get_untracked().is_some() { "PUT" } else {
                    "POST" }, Some(body), move | _ | { reset.run(()); data.reload();
                    notify(if labels { t("标签已保存", "Tag saved") } else {
                    t("分类已保存", "Category saved") }); }); })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.value.get().or_else(| | data.previous.get()).map(|
                    v | View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::table()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(leptos::html::thead()
                                .child(#[allow(unused_braces)] {
                                leptos::html::tr()
                                .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                labels.then(| | View::new(
    leptos::html::th().child(#[allow(unused_braces)] { "ID" }),
)) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ if labels {
                                t("标签", "Tags") } else { t("名称", "Name") } }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("描述",
                                "Description") }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ if labels {
                                t("分类", "Category") } else { t("排序", "Order") } })
                                })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("操作",
                                "Actions") }) })),)) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::tbody()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ v.as_array()
                                .cloned().unwrap_or_default().into_iter().map(| item | { let
                                id = num(& item, "id"); let edit = item.clone(); View::new(
    leptos::html::tr()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    labels.then(| | View::new(
    leptos::html::td()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ id }) },
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& item, if
                    labels { "label_name" } else { "sort_name" }) }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& item, if
                    labels { "label_description" } else { "sort_description" }) })
                    })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if labels {
                    list(categories.value.get()).iter().find(| c | num(c, "id") ==
                    num(& item, "sort_id")).map(| c | text(c, "sort_name"))
                    .unwrap_or_default() } else { num(& item, "priority").to_string()
                    } }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("编辑", "Edit")
                    }) }).class("xcss-button")
                    .on(leptos::ev::click, move | _ | { editing
                    .set(Some(id)); form.set(json!({ "name" : text(& edit, if labels
                    { "label_name" } else { "sort_name" }), "description" : text(&
                    edit, if labels { "label_description" } else { "sort_description"
                    }), "priority" : num(& edit, "priority"), "sort_id" : num(& edit,
                    "sort_id") })); })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("删除",
                    "Delete") }) }).class("xcss-button xcss-danger")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | deletion.busy.get()))
                    .on(leptos::ev::click, move | _ | { if save
                    .busy.get_untracked() || ! web_sys::window().unwrap()
                    .confirm_with_message(& if labels {
                    t("确定删除这个标签？", "Delete this tag?") } else {
                    t("确定删除这个分类？", "Delete this category?") })
                    .unwrap_or(false) { return; } deleting.set(Some(id)); deletion
                    .run(format!("{endpoint}/{id}"), "DELETE", None, move | _ | { if
                    editing.get_untracked() == Some(id) { reset.run(()) } for state
                    in [data.value, data.previous] { state.update(| value | { if let
                    Some(rows) = value.as_mut().and_then(Value::as_array_mut) { rows
                    .retain(| row | num(row, "id") != id); } }); } data.reload();
                    notify(if labels { t("标签已删除", "Tag deleted") } else {
                    t("分类已删除", "Category deleted") }); }); })),))
                    .class("xcss-actions") }))
                ,),
        ),
)
                                }).collect_view() }) }))
                            ,),
                    )
                    .class("xcss-table")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            if labels {
                                t("标签列表", "Tag list")
                            } else {
                                t("分类列表", "Category list")
                            },
                        ),
                    )
            },
        )
        .class("xcss-table-scroll")
        .tabindex("0")
        .role("region")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("数据表格", "Data table"),
            ),
        ),
)) })),))
                    .class("xocs-fieldset")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get())))
                ,),
        )
        .class("xcss-content-stack"),
)
}
fn moderation_author(item: &Value, message: bool) -> String {
    let username = text(item, "username");
    if !username.is_empty() {
        return username;
    }
    let user = num(item, "user_id");
    if user > 0 {
        if message {
            tr("用户 {0}", "User {0}", &[user.to_string()])
        } else {
            user.to_string()
        }
    } else {
        t("匿名访客", "Anonymous guest")
    }
}
#[component]
fn Moderation(kind: &'static str) -> impl IntoView {
    let page = RwSignal::new(1u64);
    let data = load(Signal::derive(move || {
        format!("/api/v1/content/{kind}?page={}&size=20", page.get())
    }));
    let action = Action::new();
    let upload = Action::new();
    let lightbox = RwSignal::new(None::<String>);
    let chosen = StoredValue::new(send_wrapper::SendWrapper::new(None::<web_sys::File>));
    let filename = RwSignal::new(String::new());
    let uploaded = RwSignal::new(String::new());
    let title = match kind {
        "comments" => t("评论管理", "Comments"),
        "tree-hole" => t("留言板管理", "Message board"),
        _ => t("文件管理", "Files"),
    };
    View::new(
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ title }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .retry(#[allow(unused_braces)] { Some(Callback::new(move | () |
                    data.reload())) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { action.error }).build(); props
                    }) }), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { upload.error }).build(); props
                    }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    (kind == "resources").then(| | View::new(
    (
            IntoMaybeErased::into_maybe_erased(leptos::html::fieldset()
            .child(#[allow(unused_braces)] { leptos::html::form()
            .child((IntoMaybeErased::into_maybe_erased({
            #[allow(unreachable_code)] #[allow(unused_mut)]
            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
            & FilePicker, { let mut props =
            component_props_builder(& FilePicker)
            .label(#[allow(unused_braces)] {
            t("上传图片（PNG、JPEG、GIF、WebP）",
            "Upload image (PNG, JPEG, GIF, WebP)") })
            .filename(#[allow(unused_braces)] { Signal::derive(move | | filename
            .get()) }).disabled(#[allow(unused_braces)] { Signal::derive(move | |
            upload.busy.get() || action.busy.get()) }).choose(#[allow(unused_braces)]
            { Callback::new(move | f : send_wrapper::SendWrapper < web_sys::File > |
            { filename.set(f.name()); chosen
            .set_value(send_wrapper::SendWrapper::new(Some((* f).clone()))); }) })
            .build(); props }) }),
            IntoMaybeErased::into_maybe_erased(leptos::html::button()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({
            move | | if upload.busy.get() { t("上传中…", "Uploading…") } else { t("上传", "Upload") } }) }).class("xcss-button").r#type("submit")
            .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
            | | filename.get().is_empty() || upload.busy.get()))),))
            .class("inline-form").on(leptos::ev::submit, move | ev
            | { ev.prevent_default(); let file = chosen.with_value(| f | f.as_ref()
            .cloned()); if let Some(file) = file { upload
            .upload(send_wrapper::SendWrapper::new(file), move | v | { uploaded
            .set(text(& v, "path")); filename.set(String::new()); chosen
            .set_value(send_wrapper::SendWrapper::new(None)); data.reload();
            notify(t("图片已上传", "Image uploaded")); }) } }) })
            .class("xocs-fieldset")
            .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
            | | upload.busy.get() || action.busy.get()))),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            move | | (! uploaded.get().is_empty()).then(| | View::new(
    leptos::html::p()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("新图片地址：", "New image URL: ") })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::code()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ uploaded.get() })
                    }))
                ,),
        ),
)) }))
        ,),
)) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.value.get().map(| v | View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::table()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(leptos::html::thead()
                                .child(#[allow(unused_braces)] {
                                leptos::html::tr()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ (if kind ==
                                "comments" { vec![t("来源", "Source"), t("用户",
                                "Users"), t("内容", "Content"), t("时间", "Time"),
                                t("操作", "Actions")] } else if kind == "tree-hole" {
                                vec!["ID".into(), t("作者", "Author"), t("内容",
                                "Content"), t("图片", "Image"), t("时间", "Time"),
                                t("操作", "Actions")] } else { vec![t("名称", "Name"),
                                t("类型", "Type"), t("大小", "Size"), t("位置",
                                "Location"), t("操作", "Actions")] }).into_iter().map(| v
                                | View::new(
    leptos::html::th()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ v }) },
        ),
)).collect_view() }) }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::tbody()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ rows(& v,
                                "items").into_iter().map(| item | { let id = num(& item,
                                "id"); let path = text(& item, "path"); let path2 = path
                                .clone(); View::new(
    leptos::html::tr()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    if kind == "resources" { View::new(
    (
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ text(& item,
            "original_name") }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ text(& item,
            "resource_type") }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ num(& item, "size") }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ safe_url(& path, false)
            .map(| u | View::new(
    leptos::html::a()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ t("打开", "Open") }) },
        )
        .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .target("_blank")
        .rel("noopener noreferrer"),
)) }) }))
        ,),
) .into_any() } else if kind == "comments" {
                    View::new(
    (
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ match text(& item,
            "comment_type").as_str() { "message" => t("留言板", "Message board")
            .into_any(), "love" => View::new(
    leptos::html::a()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("祝福板", "Wishes")
                })
            },
        )
        .href("/love")
        .target("_blank")
        .rel("noopener noreferrer"),
)
            .into_any(), _ => View::new(
    leptos::html::a()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    format!(
                        "{}{}", t("文章 #", "Article #"), num(& item, "source")
                    )
                })
            },
        )
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                format!("/article/{}", num(& item, "source")),
            ),
        )
        .target("_blank")
        .rel("noopener noreferrer"),
) .into_any() } }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ moderation_author(& item,
            false) }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ text(& item,
            "comment_content") }) }).class("comment-cell")),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ table_time(& text(& item,
            "create_time")) }) }))
        ,),
)
                    .into_any() } else { View::new(
    (
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ id }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ moderation_author(& item,
            true) }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ let message = text(& item,
            "message"); if message.is_empty() { t("图片留言", "Image message") }
            else { message } }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ safe_url(& text(& item,
            "image_path"), true).map(| u | { let click = u.clone(); View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::img()
                    .src(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            u,
                        ),
                    )
                    .alt(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("查看留言图片", "View message image"),
                        ),
                    )
            },
        )
        .class("moderation-photo")
        .r#type("button")
        .on(
            leptos::ev::click,
            move |_| lightbox.set(Some(click.clone())),
        ),
) }) }) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::td()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ table_time(& text(& item,
            "create_time")) }) }))
        ,),
) .into_any() } })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("删除",
                    "Delete") }) }).class("xcss-button xcss-danger")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | action.busy.get() || upload.busy.get() || (kind ==
                    "resources" && path2.is_empty())))
                    .on(leptos::ev::click, move | _ | { if !
                    web_sys::window().unwrap().confirm_with_message(& match kind {
                    "comments" =>
                    t("删除这条评论？如果是顶层评论，会同时删除该楼层的全部回复。",
                    "Delete this comment? If it is a top-level comment, all replies in its thread will also be deleted."),
                    "resources" => t("确定删除这个文件及其图片数据？",
                    "Delete this file and its image data?"), _ =>
                    t("确定删除这条留言？", "Delete this message?") })
                    .unwrap_or(false) { return; } let body = (kind == "resources")
                    .then(| | json!({ "path" : path })); action
                    .run(format!("/api/v1/content/{kind}/{id}"), "DELETE", body, move
                    | _ | data.reload()); }) }))
                ,),
        ),
) }).collect_view() }) }))
                            ,),
                    )
                    .class("xcss-table")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            match kind {
                                "comments" => t("评论列表", "Comment list"),
                                "tree-hole" => t("留言列表", "Message list"),
                                _ => t("文件列表", "File list"),
                            },
                        ),
                    )
            },
        )
        .class("xcss-table-scroll")
        .tabindex("0")
        .role("region")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("数据表格", "Data table"),
            ),
        ),
))
                    })), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & ListState, { let mut props =
                    component_props_builder(& ListState)
                    .data(#[allow(unused_braces)] { data })
                    .empty(#[allow(unused_braces)] { match kind { "comments" =>
                    t("暂无评论", "No comments yet"), "tree-hole" =>
                    t("暂无留言", "No messages yet"), _ => t("暂无文件",
                    "No files yet") } }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Pager, { let mut props =
                    component_props_builder(& Pager)
                    .data(#[allow(unused_braces)] { data })
                    .page(#[allow(unused_braces)] { page }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Lightbox, { let mut props =
                    component_props_builder(& Lightbox)
                    .src(#[allow(unused_braces)] { lightbox }).build(); props }) })
                ,),
        )
        .class("xcss-content-stack"),
)
}
#[component]
fn SiteSettings() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/site".into()));
    let form = RwSignal::new(Value::Null);
    let save = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            form.set(v);
        }
    });
    View::new(
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("网站设置",
                    "Site settings") }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .retry(#[allow(unused_braces)] { Some(Callback::new(move | () |
                    data.reload())) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { save.error }).build(); props })
                    }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! form.get().is_null()).then(| | View::new(
    leptos::html::form()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::fieldset()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::section()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("基本信息",
                    "Basic information") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("网站名称", "Site name")
                    }).key(#[allow(unused_braces)] { "web_name" })
                    .form(#[allow(unused_braces)] { form })
                    .required(#[allow(unused_braces)] { true }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("网站标题", "Site title")
                    }).key(#[allow(unused_braces)] { "web_title" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("站长显示名",
                    "Site owner display name") }).key(#[allow(unused_braces)] {
                    "author_name" }).form(#[allow(unused_braces)] { form })
                    .required(#[allow(unused_braces)] { true })
                    .max(#[allow(unused_braces)] { 200 }).build(); props }) }),))
                    .class("form-grid")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("用于文章和随笔署名；管理员登录名与密码在右上角头像的账号设置中修改。",
                    "Used as the author of articles and notes. Change administrator sign-in credentials through the account settings avatar.")
                    }) }).class("xocs-field-help")),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("公告", "Notices") })
                    .key(#[allow(unused_braces)] { "notices" })
                    .form(#[allow(unused_braces)] { form })
                    .textarea(#[allow(unused_braces)] { true }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("使用 JSON 字符串数组或每行一条公告。",
                    "Use a JSON string array or one notice per line.") }) })
                    .class("xocs-field-help")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::details()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::summary()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("推送格式",
                    "Promotion format") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::code()
                    .child(#[allow(unused_braces)] { "推送标题：" })),
                    IntoMaybeErased::into_maybe_erased(" "),
                    IntoMaybeErased::into_maybe_erased(leptos::html::code()
                    .child(#[allow(unused_braces)] { "推送封面：" })),
                    IntoMaybeErased::into_maybe_erased(" "),
                    IntoMaybeErased::into_maybe_erased(leptos::html::code()
                    .child(#[allow(unused_braces)] { "推送链接：" })),))),))),))
                    .class("xcss-content-panel")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::section()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("外观",
                    "Appearance") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("背景图片",
                    "Background image") }).key(#[allow(unused_braces)] {
                    "background_image" }).form(#[allow(unused_braces)] { form })
                    .build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("站长头像",
                    "Site owner avatar") }).key(#[allow(unused_braces)] { "avatar" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),))
                    .class("form-grid")),)).class("xcss-content-panel")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::section()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("随机封面",
                    "Random covers") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("使用 JSON 图片地址数组或每行一个图片地址。",
                    "Use a JSON array of image URLs or one image URL per line.") })
                    }).class("xocs-field-help")),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("随机封面",
                    "Random covers") }).key(#[allow(unused_braces)] { "random_cover"
                    }).form(#[allow(unused_braces)] { form })
                    .textarea(#[allow(unused_braces)] { true }).build(); props })
                    }),)).class("xcss-content-panel")),)).class("xocs-fieldset")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get()))
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("网站设置",
                    "Site settings")))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child(#[allow(unused_braces)] {
                    leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("保存设置",
                    "Save settings") }) }).class("xcss-button").r#type("submit")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get())) }).class("xcss-actions"))
                ,),
        )
        .class("xocs-settings-form")
        .on(
            leptos::ev::submit,
            move |ev| {
                ev.prevent_default();
                let mut f = form.get_untracked();
                let name = text(&f, "author_name").trim().to_string();
                let len = name.chars().count();
                if !(1..=100).contains(&len) {
                    return;
                }
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
            },
        ),
)) }))
                ,),
        )
        .class("xcss-content-stack"),
)
}
#[component]
fn LetterSettings() -> impl IntoView {
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
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("信笺管理",
                    "Letter") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("维护公开信笺的收信人、署名和正文。",
                    "Manage the recipient, sender, and content of the public letter.")
                    }) })), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .retry(#[allow(unused_braces)] { Some(Callback::new(move | () |
                    data.reload())) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { save.error }).build(); props })
                    }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! form.get().is_null()).then(| | View::new(
    leptos::html::form()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::fieldset()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased({
                                #[allow(unreachable_code)] #[allow(unused_mut)]
                                #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                & Field, { let mut props =
                                component_props_builder(& Field)
                                .label(#[allow(unused_braces)] { t("收信人", "Recipient")
                                }).key(#[allow(unused_braces)] { "recipient" })
                                .form(#[allow(unused_braces)] { form })
                                .required(#[allow(unused_braces)] { true })
                                .max(#[allow(unused_braces)] { 200 }).build(); props }) }),
                                IntoMaybeErased::into_maybe_erased({
                                #[allow(unreachable_code)] #[allow(unused_mut)]
                                #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                & Field, { let mut props =
                                component_props_builder(& Field)
                                .label(#[allow(unused_braces)] { t("署名", "Sender") })
                                .key(#[allow(unused_braces)] { "sender" })
                                .form(#[allow(unused_braces)] { form })
                                .required(#[allow(unused_braces)] { true })
                                .max(#[allow(unused_braces)] { 200 }).build(); props }) }),
                                IntoMaybeErased::into_maybe_erased({
                                #[allow(unreachable_code)] #[allow(unused_mut)]
                                #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                & Field, { let mut props =
                                component_props_builder(& Field)
                                .label(#[allow(unused_braces)] { t("信笺正文",
                                "Letter content") }).key(#[allow(unused_braces)] { "content"
                                }).form(#[allow(unused_braces)] { form })
                                .required(#[allow(unused_braces)] { true })
                                .textarea(#[allow(unused_braces)] { true })
                                .max(#[allow(unused_braces)] { 32768 }).build(); props })
                                }), IntoMaybeErased::into_maybe_erased({
                                #[allow(unreachable_code)] #[allow(unused_mut)]
                                #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                & Check, { let mut props =
                                component_props_builder(& Check)
                                .label(#[allow(unused_braces)] { t(" 公开显示",
                                " Show publicly") }).key(#[allow(unused_braces)] {
                                "is_public" }).form(#[allow(unused_braces)] { form })
                                .build(); props }) }),
                                IntoMaybeErased::into_maybe_erased(leptos::html::div()
                                .child((IntoMaybeErased::into_maybe_erased(leptos::html::button()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({
                                t("保存信笺", "Save letter") }) }).class("xcss-button")
                                .r#type("submit")
                                .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                                | | save.busy.get()))),
                                IntoMaybeErased::into_maybe_erased(leptos::html::button()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({
                                t("撤销修改", "Reset changes") }) })
                                .class("xcss-button").r#type("button")
                                .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                                | | save.busy.get() || form.get() == original.get()))
                                .on(leptos::ev::click, move | _ | { form
                                .set(original.get_untracked()); save.error.set(None); })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::a()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({
                                t("查看公开页面", "View public page") }) })
                                .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(href("/letter")))
                                .target("_blank").rel("noopener noreferrer")),))
                                .class("xcss-actions"))
                            ,),
                    )
                    .class("xocs-fieldset")
                    .disabled(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || save.busy.get(),
                        ),
                    )
            },
        )
        .class("editor-form")
        .on(
            leptos::ev::submit,
            move |ev| {
                ev.prevent_default();
                let f = form.get_untracked();
                let valid = [("recipient", 100), ("sender", 100), ("content", 16384)]
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
            },
        ),
))
                    }))
                ,),
        )
        .class("xcss-content-stack"),
)
}
fn empty_family() -> Value {
    json!({"bg_cover":"","man_cover":"","woman_cover":"","man_name":"","woman_name":"","timing":"","countdown_title":"","countdown_time":"","family_info":"","status":true})
}
fn family_form(item: &Value) -> Value {
    let mut form = empty_family();
    for key in [
        "bg_cover",
        "man_cover",
        "woman_cover",
        "man_name",
        "woman_name",
        "timing",
        "countdown_title",
        "countdown_time",
        "family_info",
    ] {
        form[key] = json!(text(item, key));
    }
    form["status"] = json!(flag(item, "status"));
    form
}
#[component]
fn FamilySettings() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/content/family".into()));
    let form = RwSignal::new(empty_family());
    let editing = RwSignal::new(None::<i64>);
    let save = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            if let Some(item) = v.as_array().and_then(|a| a.first()) {
                editing.set(Some(num(item, "id")));
                form.set(family_form(item));
            } else {
                editing.set(None);
                form.set(empty_family());
            }
        }
    });
    View::new(
        leptos::html::section()
            .child((
                IntoMaybeErased::into_maybe_erased(leptos::html::h1().child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({ t("恋爱笔记", "Love journal") })
                    },
                )),
                IntoMaybeErased::into_maybe_erased(leptos::html::p().child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            t(
                                "这里仅由管理员创建并维护一份家的资料。",
                                "The administrator creates and maintains one home here.",
                            )
                        })
                    },
                )),
                IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)]
                    #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Feedback,
                        {
                            let mut props = component_props_builder(&Feedback)
                                .error(
                                    #[allow(unused_braces)]
                                    {
                                        data.error
                                    },
                                )
                                .retry(
                                    #[allow(unused_braces)]
                                    {
                                        Some(Callback::new(move |()| data.reload()))
                                    },
                                )
                                .build();
                            props
                        },
                    )
                }),
                IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)]
                    #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Feedback,
                        {
                            let mut props = component_props_builder(&Feedback)
                                .error(
                                    #[allow(unused_braces)]
                                    {
                                        save.error
                                    },
                                )
                                .build();
                            props
                        },
                    )
                }),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::fieldset()
                        .child(
                            #[allow(unused_braces)]
                            {
                                leptos::html::form()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if editing.get().is_some() { t("编辑家的资料",
                    "Edit home details") } else { t("创建我的家",
                    "Create my home") } }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("男生昵称", "His name") })
                    .key(#[allow(unused_braces)] { "man_name" })
                    .form(#[allow(unused_braces)] { form })
                    .required(#[allow(unused_braces)] { true })
                    .max(#[allow(unused_braces)] { 32 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("女生昵称", "Her name") })
                    .key(#[allow(unused_braces)] { "woman_name" })
                    .form(#[allow(unused_braces)] { form })
                    .required(#[allow(unused_braces)] { true })
                    .max(#[allow(unused_braces)] { 32 }).build(); props }) }),))
                    .class("form-grid")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("起始时间", "Start date")
                    }).key(#[allow(unused_braces)] { "timing" })
                    .form(#[allow(unused_braces)] { form })
                    .max(#[allow(unused_braces)] { 32 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("倒计时标题",
                    "Countdown title") }).key(#[allow(unused_braces)] {
                    "countdown_title" }).form(#[allow(unused_braces)] { form })
                    .max(#[allow(unused_braces)] { 32 }).build(); props }) }),))
                    .class("form-grid")),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("倒计时时间",
                    "Countdown date") }).key(#[allow(unused_braces)] {
                    "countdown_time" }).form(#[allow(unused_braces)] { form })
                    .max(#[allow(unused_braces)] { 32 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("背景图片",
                    "Background image") }).key(#[allow(unused_braces)] { "bg_cover"
                    }).form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("男生头像", "His avatar")
                    }).key(#[allow(unused_braces)] { "man_cover" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("女生头像", "Her avatar")
                    }).key(#[allow(unused_braces)] { "woman_cover" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),))
                    .class("form-grid")),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("简介", "Introduction") })
                    .key(#[allow(unused_braces)] { "family_info" })
                    .form(#[allow(unused_braces)] { form })
                    .textarea(#[allow(unused_braces)] { true })
                    .max(#[allow(unused_braces)] { 1024 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Check, { let mut props =
                    component_props_builder(& Check)
                    .label(#[allow(unused_braces)] { t(" 公开显示",
                    " Show publicly") }).key(#[allow(unused_braces)] { "status" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("保存", "Save")
                    }) }).class("xcss-button").r#type("submit")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | editing.get().map(| id | View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("删除家的资料", "Delete home details")
                })
            },
        )
        .class("xcss-button xcss-danger")
        .r#type("button")
        .on(
            leptos::ev::click,
            move |_| {
                if web_sys::window()
                    .unwrap()
                    .confirm_with_message(
                        &t(
                            "确定删除家的资料？",
                            "Delete your home details?",
                        ),
                    )
                    .unwrap_or(false)
                {
                    save.run(
                        format!("/api/v1/content/family/{id}"),
                        "DELETE",
                        None,
                        move |_| data.reload(),
                    );
                }
            },
        ),
)) })),))
                    .class("xcss-actions")),)).class("editor-form")
                    .on(leptos::ev::submit, move | ev | { ev
                    .prevent_default(); if data.value.get_untracked().is_none() {
                    return; } let mut payload = form.get_untracked(); if let Some(o)
                    = payload.as_object_mut() { o.remove("id"); } for k in
                    ["bg_cover", "man_cover", "woman_cover", "countdown_title",
                    "countdown_time", "family_info"] { if text(& payload, k)
                    .is_empty() { payload[k] = Value::Null; } } let path = editing
                    .get_untracked().map(| id |
                    format!("/api/v1/content/family/{id}"))
                    .unwrap_or("/api/v1/content/family".into()); save.run(path, if
                    editing.get_untracked().is_some() { "PUT" } else { "POST" },
                    Some(payload), move | v | { editing.set(Some(num(& v, "id")));
                    form.set(family_form(& v)); notify(t("家的资料已保存",
                    "Home details saved")); }); })
                            },
                        )
                        .class("xocs-fieldset")
                        .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || save.busy.get() || data.value.get().is_none(),
                        )),
                ),
            ))
            .class("xcss-content-stack"),
    )
}
#[component]
fn HomeSections() -> impl IntoView {
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
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("首页栏目",
                    "Home sections") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("设置栏目、顺序和展示内容。推荐栏目使用文章的“推荐”状态。",
                    "Set sections, their order, and content. Recommended sections use articles marked as recommended.")
                    }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("保存栏目",
                    "Save sections") }) }).class("xcss-button")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || ! loaded.get()))
                    .on(leptos::ev::click, move | _ | {
                    save_sections.run(()) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .fallback(#[allow(unused_braces)] { t("首页栏目加载失败",
                    "Unable to load home sections") }).retry(#[allow(unused_braces)]
                    { Some(Callback::new(move | () | data.reload())) }).build();
                    props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { categories.error })
                    .fallback(#[allow(unused_braces)] { t("分类加载失败",
                    "Unable to load categories") }).retry(#[allow(unused_braces)] {
                    Some(Callback::new(move | () | categories.reload())) }).build();
                    props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { save.error })
                    .fallback(#[allow(unused_braces)] { t("保存失败",
                    "Unable to save") }).retry(#[allow(unused_braces)] {
                    Some(save_sections) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! validation.get().is_empty()).then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ validation.get() }) },
        )
        .role("alert"),
)) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::fieldset()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child(#[allow(unused_braces)] {
                    leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("新增分类栏目", "Add category section") }) })
                    .class("xcss-button")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value({
                    move | | sections.get().len() >= 12 }))
                    .on(leptos::ev::click, move | _ | sections
                    .update(| rows | rows.push(json!({ "id":- (js_sys::Date::now() as
                    i64), "title" : t("新栏目", "New section"), "kind" :
                    "category", "sort_id" : list(categories.value.get_untracked())
                    .first().map(| v | v["id"].clone()).unwrap_or(Value::Null),
                    "priority" : rows.len(), "enabled" : true })))) })
                    .class("xcss-actions")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child(#[allow(unused_braces)] {
                    leptos::html::table()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::thead()
                    .child(#[allow(unused_braces)] {
                    leptos::html::tr()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ [t("顺序",
                    "Order"), t("标题", "Title"), t("内容来源",
                    "Content source"), t("分类", "Category"), t("显示", "Show"),
                    t("操作", "Actions")].into_iter().map(| l | View::new(
    leptos::html::th()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ l }) },
        ),
)).collect_view() }) }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::tbody()
                    .child(#[allow(unused_braces)] { { #[allow(unreachable_code)]
                    #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & For, { let mut props =
                    component_props_builder(& For)
                    .each(#[allow(unused_braces)] { move | | sections.get() })
                    .key(#[allow(unused_braces)] { | v | num(v, "id") })
                    .children(#[allow(unused_braces)] { move | item | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &HomeRow,
        {
            let mut props = component_props_builder(&HomeRow)
                .id(#[allow(unused_braces)] { num(&item, "id") })
                .sections(#[allow(unused_braces)] { sections })
                .categories(#[allow(unused_braces)] { categories })
                .build();
            props
        },
    )
}) })
                    .build(); props }) } })),)).class("xcss-table")
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("首页栏目列表",
                    "Home section list"))) }).class("xcss-table-scroll")
                    .tabindex("0").role("region")
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("数据表格",
                    "Data table")))),)).class("xocs-fieldset")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get() || ! loaded.get())))
                ,),
        )
        .class("xcss-content-stack"),
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
    View::new(leptos::html::tr().child((
        IntoMaybeErased::into_maybe_erased(leptos::html::td().child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({ move || index.get() + 1 })
            },
        )),
        IntoMaybeErased::into_maybe_erased(leptos::html::td().child(
            #[allow(unused_braces)]
            {
                leptos::html::label()
                    .child((
                        IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move || {
                                format!(
                                    "{} {} {}",
                                    t("栏目", "Section"),
                                    index.get() + 1,
                                    t("标题", "title")
                                )
                            }
                        })),
                        IntoMaybeErased::into_maybe_erased(
                            leptos::html::input()
                                .class("xcss-input")
                                .maxlength("32")
                                .prop("value", move || text(&current.get(), "title"))
                                .on(leptos::ev::input, move |ev| {
                                    change.run(("title", json!(event_target_value(&ev))))
                                }),
                        ),
                    ))
                    .class("xcss-form-field")
            },
        )),
        IntoMaybeErased::into_maybe_erased(leptos::html::td().child(
            #[allow(unused_braces)]
            {
                {
                    #[allow(unreachable_code)]
                    #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &crate::select::Select,
                        {
                            let mut props = component_props_builder(&crate::select::Select)
                                .label(
                                    #[allow(unused_braces)]
                                    {
                                        Signal::derive(move || {
                                            tr(
                                                "栏目 {0} 内容来源",
                                                "Section {0} content source",
                                                &[(index.get() + 1).to_string()],
                                            )
                                        })
                                    },
                                )
                                .value(
                                    #[allow(unused_braces)]
                                    {
                                        Signal::derive(move || text(&current.get(), "kind"))
                                    },
                                )
                                .options(
                                    #[allow(unused_braces)]
                                    {
                                        Signal::derive(|| {
                                            vec![
                                                ("latest".into(), t("最新文章", "Latest articles")),
                                                (
                                                    "recommended".into(),
                                                    t("推荐文章", "Recommended articles"),
                                                ),
                                                (
                                                    "category".into(),
                                                    t("指定分类", "Selected category"),
                                                ),
                                            ]
                                        })
                                    },
                                )
                                .change(
                                    #[allow(unused_braces)]
                                    {
                                        Callback::new(move |kind: String| {
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
                                        })
                                    },
                                )
                                .build();
                            props
                        },
                    )
                }
            },
        )),
        IntoMaybeErased::into_maybe_erased(leptos::html::td().child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    move || {
                        (text(&current.get(), "kind") == "category").then(|| {
                            View::new({
                                #[allow(unreachable_code)]
                                #[allow(unused_mut)]
                                #[allow(
                                    clippy::let_and_return,
                                    clippy::unit_arg,
                                    clippy::let_unit_value
                                )]
                                component_view(
                                    #[allow(clippy::needless_borrows_for_generic_args)]
                                    &crate::select::Select,
                                    {
                                        let mut props =
                                            component_props_builder(&crate::select::Select)
                                                .label(
                                                    #[allow(unused_braces)]
                                                    {
                                                        Signal::derive(move || {
                                                            tr(
                                                                "栏目 {0} 分类",
                                                                "Section {0} category",
                                                                &[(index.get() + 1).to_string()],
                                                            )
                                                        })
                                                    },
                                                )
                                                .value(
                                                    #[allow(unused_braces)]
                                                    {
                                                        Signal::derive(move || {
                                                            num(&current.get(), "sort_id")
                                                                .to_string()
                                                        })
                                                    },
                                                )
                                                .options(
                                                    #[allow(unused_braces)]
                                                    {
                                                        Signal::derive(move || {
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
                                                        })
                                                    },
                                                )
                                                .change(
                                                    #[allow(unused_braces)]
                                                    {
                                                        Callback::new(move |v: String| {
                                                            change.run((
                                                                "sort_id",
                                                                json!(
                                                                    v.parse::<i64>().unwrap_or(0)
                                                                ),
                                                            ))
                                                        })
                                                    },
                                                )
                                                .build();
                                        props
                                    },
                                )
                            })
                        })
                    }
                })
            },
        )),
        IntoMaybeErased::into_maybe_erased(leptos::html::td().child(
            #[allow(unused_braces)]
            {
                leptos::html::label().child((
                    IntoMaybeErased::into_maybe_erased(
                        leptos::html::input()
                            .r#type("checkbox")
                            .prop("checked", move || flag(&current.get(), "enabled"))
                            .on(leptos::ev::change, move |ev| {
                                change.run(("enabled", json!(event_target_checked(&ev))))
                            }),
                    ),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                        t(" 启用", " Enable")
                    })),
                ))
            },
        )),
        IntoMaybeErased::into_maybe_erased(leptos::html::td().child(
            #[allow(unused_braces)]
            {
                leptos::html::div()
                    .child((
                        IntoMaybeErased::into_maybe_erased(
                            leptos::html::button()
                                .child(
                                    #[allow(unused_braces)]
                                    {
                                        "↑"
                                    },
                                )
                                .class("xcss-button")
                                .aria_label(
                                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                        move || {
                                            format!(
                                                "{}{}",
                                                t("上移栏目 ", "Move section up "),
                                                index.get() + 1
                                            )
                                        },
                                    ),
                                )
                                .disabled(
                                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                        move || index.get() == 0,
                                    ),
                                )
                                .on(leptos::ev::click, move |_| {
                                    let i = index.get_untracked();
                                    sections.update(|r| {
                                        if i > 0 {
                                            r.swap(i, i - 1)
                                        }
                                    })
                                }),
                        ),
                        IntoMaybeErased::into_maybe_erased(
                            leptos::html::button()
                                .child(
                                    #[allow(unused_braces)]
                                    {
                                        "↓"
                                    },
                                )
                                .class("xcss-button")
                                .aria_label(
                                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                        move || {
                                            format!(
                                                "{}{}",
                                                t("下移栏目 ", "Move section down "),
                                                index.get() + 1
                                            )
                                        },
                                    ),
                                )
                                .disabled(
                                    ::leptos::prelude::IntoAttributeValue::into_attribute_value({
                                        move || index.get() + 1 >= sections.get().len()
                                    }),
                                )
                                .on(leptos::ev::click, move |_| {
                                    let i = index.get_untracked();
                                    sections.update(|r| {
                                        if i + 1 < r.len() {
                                            r.swap(i, i + 1)
                                        }
                                    })
                                }),
                        ),
                        IntoMaybeErased::into_maybe_erased(
                            leptos::html::button()
                                .child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({ t("删除", "Delete") })
                                    },
                                )
                                .class("xcss-button xcss-danger")
                                .on(leptos::ev::click, move |_| {
                                    sections.update(|r| r.retain(|v| num(v, "id") != id))
                                }),
                        ),
                    ))
                    .class("xcss-actions")
            },
        )),
    )))
}
fn empty_link() -> Value {
    json!({"title":"","classify":"","cover":"","url":"","introduction":"","link_type":"favorites","status":true})
}
#[component]
fn Links() -> impl IntoView {
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
    View::new(
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("内容资源",
                    "Content resources") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("维护友链、收藏、照片和音乐条目。",
                    "Manage friend links, favorites, photos, and music.") }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .retry(#[allow(unused_braces)] { Some(Callback::new(move | () |
                    data.reload())) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { save.error }).build(); props })
                    }), IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { remove.error }).build(); props
                    }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("类型筛选", "Filter by type") })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & crate ::select::Select, { let mut props =
                    component_props_builder(& crate
                    ::select::Select).label(#[allow(unused_braces)] {
                    Signal::derive(| | t("类型筛选", "Filter by type")) })
                    .value(#[allow(unused_braces)] { Signal::derive(move | | kind
                    .get()) }).options(#[allow(unused_braces)] { Signal::derive(move
                    | | std::iter::once((String::new(), t("全部", "All")))
                    .chain(options().into_iter().map(| (k, l) | (k.to_string(), l)))
                    .collect()) }).change(#[allow(unused_braces)] {
                    Callback::new(move | v : String | { kind.set(v); page.set(1); })
                    }).build(); props }) }),)).class("xcss-form-field")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("状态筛选", "Filter by status") })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & crate ::select::Select, { let mut props =
                    component_props_builder(& crate
                    ::select::Select).label(#[allow(unused_braces)] {
                    Signal::derive(| | t("状态筛选", "Filter by status")) })
                    .value(#[allow(unused_braces)] { Signal::derive(move | | status
                    .get().to_string()) }).options(#[allow(unused_braces)] {
                    Signal::derive(| | vec![("-1".into(), t("全部", "All")), ("0"
                    .into(), t("待审核/隐藏", "Pending review / hidden")), ("1"
                    .into(), t("已公开", "Published"))]) })
                    .change(#[allow(unused_braces)] { Callback::new(move | v : String
                    | { status.set(v.parse().unwrap_or(- 1)); page.set(1); }) })
                    .build(); props }) }),)).class("xcss-form-field")),))
                    .class("inline-form")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::fieldset()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::form()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if editing.get().is_some() { t("编辑资源",
                    "Edit resource") } else { t("新增资源", "Add resource") } })
                    })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("类型", "Type") })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & crate ::select::Select, { let mut props =
                    component_props_builder(& crate
                    ::select::Select).label(#[allow(unused_braces)] {
                    Signal::derive(| | t("类型", "Type")) })
                    .value(#[allow(unused_braces)] { Signal::derive(move | | text(&
                    form.get(), "link_type")) }).options(#[allow(unused_braces)] {
                    Signal::derive(move | | options().into_iter().map(| (k, l) | (k
                    .to_string(), l)).collect()) }).change(#[allow(unused_braces)] {
                    Callback::new(move | v : String | form.update(| f |
                    f["link_type"] = json!(v))) }).build(); props }) }),))
                    .class("xcss-form-field")),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("分类", "Category") })
                    .key(#[allow(unused_braces)] { "classify" })
                    .form(#[allow(unused_braces)] { form })
                    .max(#[allow(unused_braces)] { 32 }).build(); props }) }),))
                    .class("form-grid")),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("标题", "Title") })
                    .key(#[allow(unused_braces)] { "title" })
                    .form(#[allow(unused_braces)] { form })
                    .required(#[allow(unused_braces)] { true })
                    .max(#[allow(unused_braces)] { 64 }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("图片地址", "Image URL")
                    }).key(#[allow(unused_braces)] { "cover" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("目标地址", "Target URL")
                    }).key(#[allow(unused_braces)] { "url" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Field, { let mut props =
                    component_props_builder(& Field)
                    .label(#[allow(unused_braces)] { t("简介", "Introduction") })
                    .key(#[allow(unused_braces)] { "introduction" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Check, { let mut props =
                    component_props_builder(& Check)
                    .label(#[allow(unused_braces)] { t(" 公开显示",
                    " Show publicly") }).key(#[allow(unused_braces)] { "status" })
                    .form(#[allow(unused_braces)] { form }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if editing.get().is_some() { t("保存修改",
                    "Save changes") } else { t("新增资源", "Add resource") } })
                    }).class("xcss-button").r#type("submit")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | editing.get().map(| _ | View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("取消编辑", "Cancel editing")
                })
            },
        )
        .class("xcss-button")
        .r#type("button")
        .on(leptos::ev::click, move |_| reset.run(())),
))
                    })),)).class("xcss-actions")),)).class("editor-form")
                    .on(leptos::ev::submit, move | ev | { ev
                    .prevent_default(); let mut f = form.get_untracked(); for k in
                    ["classify", "cover", "url", "introduction"] { if text(& f, k)
                    .is_empty() { f[k] = Value::Null; } } let path = editing
                    .get_untracked().map(| id |
                    format!("/api/v1/content/links/{id}"))
                    .unwrap_or("/api/v1/content/links".into()); save.run(path, if
                    editing.get_untracked().is_some() { "PUT" } else { "POST" },
                    Some(f), move | _ | { reset.run(()); data.reload();
                    notify(t("资源已保存", "Resource saved")); }); })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.value.get().map(| v | View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::table()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(leptos::html::thead()
                                .child(#[allow(unused_braces)] {
                                leptos::html::tr()
                                .child((IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("标题",
                                "Title") }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("类型",
                                "Type") }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("状态",
                                "Status") }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::th()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ t("操作",
                                "Actions") }) })),)) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::tbody()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ rows(& v,
                                "items").into_iter().map(| item | { let id = num(& item,
                                "id"); let edit = item.clone(); View::new(
    leptos::html::tr()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& item,
                    "title") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& item,
                    "link_type") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if flag(& item,
                    "status") { t("公开", "Public") } else { t("待审核/隐藏",
                    "Pending review / hidden") } }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::td()
                    .child(#[allow(unused_braces)] {
                    leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("编辑", "Edit")
                    }) }).class("xcss-button")
                    .on(leptos::ev::click, move | _ | { editing
                    .set(Some(id)); let mut f = empty_link(); for k in ["title",
                    "classify", "cover", "url", "introduction", "link_type"] { f[k] =
                    json!(text(& edit, k)); } f["status"] = json!(flag(& edit,
                    "status")); form.set(f); })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    (text(& item, "link_type") == "friendUrl" && ! flag(& item,
                    "status")).then(| | View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("审核通过", "Approve")
                })
            },
        )
        .class("xcss-button")
        .on(
            leptos::ev::click,
            move |_| {
                remove
                    .run(
                        format!("/api/v1/content/links/{id}/status"),
                        "PUT",
                        Some(json!({ "active" : true })),
                        move |_| data.reload(),
                    )
            },
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("删除",
                    "Delete") }) }).class("xcss-button xcss-danger")
                    .on(leptos::ev::click, move | _ | { if
                    web_sys::window().unwrap().confirm_with_message(&
                    t("确定删除这条资源？", "Delete this resource?"))
                    .unwrap_or(false) { remove
                    .run(format!("/api/v1/content/links/{id}"), "DELETE", None, move
                    | _ | data.reload()); } })),)).class("xcss-actions") }))
                ,),
        ),
) }).collect_view() }) }))
                            ,),
                    )
                    .class("xcss-table")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("内容资源列表", "Content resource list"),
                        ),
                    )
            },
        )
        .class("xcss-table-scroll")
        .tabindex("0")
        .role("region")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("数据表格", "Data table"),
            ),
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & ListState, { let mut props =
                    component_props_builder(& ListState)
                    .data(#[allow(unused_braces)] { data })
                    .empty(#[allow(unused_braces)] { t("暂无内容资源",
                    "No content resources yet") }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Pager, { let mut props =
                    component_props_builder(& Pager)
                    .data(#[allow(unused_braces)] { data })
                    .page(#[allow(unused_braces)] { page }).build(); props }) }),))
                    .class("xocs-fieldset")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | save.busy.get())))
                ,),
        )
        .class("xcss-content-stack"),
)
}
