use crate::common::{Action, Feedback, Pager, load_typed, navigate, t};
use crate::models::{ArticleSummary, Page};
use blog_interface::component::component_props_builder;
use blog_interface::components::{HeaderActions, notify};
use blog_interface::prelude::*;

#[component]
pub(super) fn Articles() -> impl IntoView {
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
        blog_interface::html::section()
            .child((
                HeaderActions(
                    component_props_builder(&HeaderActions)
                        .children(::blog_interface::children::ToChildren::to_children(
                            move || {
                                (
                                    blog_interface::html::button()
                                        .child("+")
                                        .class("blog-button")
                                        .aria_label(t("新增文章", "Add article"))
                                        .title(t("新增文章", "Add article"))
                                        .on(blog_interface::ev::click, move |_| navigate("new")),
                                    blog_interface::html::button()
                                        .child("↻")
                                        .class("blog-button")
                                        .aria_label(t("刷新", "Refresh"))
                                        .title(t("刷新", "Refresh"))
                                        .disabled(move || data.busy.get())
                                        .on(blog_interface::ev::click, move |_| data.reload()),
                                )
                            },
                        ))
                        .build(),
                ),
                blog_interface::html::h1()
                    .class("blog-row")
                    .child(t("文章管理", "Articles")),
                blog_interface::html::p().class("blog-row").child(t(
                    "管理公开文章与草稿。",
                    "Manage published articles and drafts.",
                )),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .fallback(t("文章加载失败", "Unable to load articles"))
                        .retry(Some(Callback::new(move |()| data.reload())))
                        .build(),
                ),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(action.error)
                        .fallback(t(
                            "删除失败，请刷新后重试",
                            "Delete failed. Refresh and try again.",
                        ))
                        .build(),
                ),
                move || {
                    data.value.get().map(|v| {
                        View::new(
                            blog_interface::html::div()
                                .child(
                                    blog_interface::html::table()
                                        .child((
                                            blog_interface::html::thead().child(
                                                blog_interface::html::tr().child(
                                                    [
                                                        t("标题", "Title"),
                                                        t("状态", "Status"),
                                                        t("浏览", "Views"),
                                                        t("创建时间", "Created at"),
                                                        t("操作", "Actions"),
                                                    ]
                                                    .into_iter()
                                                    .map(|l| {
                                                        View::new(
                                                            blog_interface::html::th().child(l),
                                                        )
                                                    })
                                                    .collect_view(),
                                                ),
                                            ),
                                            blog_interface::html::tbody().child(
                                                v.items
                                                    .into_iter()
                                                    .map(|a| {
                                                        let id = a.id;
                                                        View::new(
    blog_interface::html::tr()
        .child(
            (
                    blog_interface::html::td().class("blog-title-cell")
                    .child((blog_interface::html::strong()
                    .child( a.article_title),
                    blog_interface::html::small()
                    .child( format!("#{id}")).class("muted"),)),
                    blog_interface::html::td()
                    .child( blog_interface::html::span()
                    .child( if a.view_status {
                    t("公开", "Public") } else { t("加密", "Password protected")
                    }).class(if a.view_status { "badge success" } else { "badge"
                    })),
                    blog_interface::html::td()
                    .child( a.view_count),
                    blog_interface::html::td()
                    .child( crate
                    ::dates::format(& a.create_time.unwrap_or_default())),
                    blog_interface::html::td()
                    .child( blog_interface::html::div()
                    .child((blog_interface::html::button()
                    .child( t("编辑", "Edit")).class("blog-button")
                    .on(blog_interface::ev::click, move | _ | navigate(&
                    format!("edit/{id}"))),
                    blog_interface::html::button()
                    .child( t("删除",
                    "Delete")).class("blog-button blog-danger")
                    .disabled(move
                    | | action.busy.get()).on(blog_interface::ev::click,
                    move | _ | if web_sys::window().unwrap().confirm_with_message(&
                    t("确定删除这篇文章？", "Delete this article?"))
                    .unwrap_or(false) { action
                    .run(format!("/api/v1/content/articles/{id}"), "DELETE", None,
                    move | _ | { notify(t("文章已删除", "Article deleted"));
                    data.reload(); }); }),)).class("blog-actions blog-flow"))
                ,),
        ),
)
                                                    })
                                                    .collect_view(),
                                            ),
                                        ))
                                        .class("blog-table")
                                        .aria_label(t("文章列表", "Article list")),
                                )
                                .class("blog-table-scroll blog-frame")
                                .tabindex("0")
                                .role("region")
                                .aria_label(t("数据表格", "Data table")),
                        )
                    })
                },
                Pager(
                    component_props_builder(&Pager)
                        .data(data)
                        .page(page)
                        .build(),
                ),
            ))
            .class("blog-content-stack blog-stack"),
    )
}
