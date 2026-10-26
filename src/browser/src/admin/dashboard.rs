use super::Group;
use crate::common::{Feedback, href, load_typed, t};
use crate::models::AdminStats;
use blog_interface::component::component_props_builder;
use blog_interface::prelude::*;

#[component]
pub(super) fn Dashboard() -> impl IntoView {
    let data = load_typed::<AdminStats>(Signal::derive(|| "/api/v1/content/statistics".into()));
    View::new(
        blog_interface::html::section()
            .class("blog-content-stack blog-stack")
            .aria_label(t("总览", "Overview"))
            .child((
                blog_interface::html::p().class("blog-row").child(t(
                    "网站内容与社区状态。",
                    "Site content and community status.",
                )),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .fallback(t("统计信息加载失败", "Unable to load statistics"))
                        .retry(Some(Callback::new(move |()| data.reload())))
                        .build(),
                ),
                move || {
                    data.value.get().map(|stats| {
                        View::new(
                            blog_interface::html::div()
                                .class("blog-table-scroll blog-frame")
                                .tabindex("0")
                                .role("region")
                                .aria_label(t("数据表格", "Data table"))
                                .child(
                                    blog_interface::html::table()
                                        .class("blog-table blog-statistics-table")
                                        .aria_label(t("网站统计", "Site statistics"))
                                        .child((
                                            blog_interface::html::thead().child(
                                                blog_interface::html::tr().child((
                                                    blog_interface::html::th()
                                                        .child(t("指标", "Metric")),
                                                    blog_interface::html::th()
                                                        .child(t("数量", "Count")),
                                                )),
                                            ),
                                            blog_interface::html::tbody().child(
                                                [
                                                    (stats.articles, t("文章", "Articles")),
                                                    (stats.comments, t("评论", "Comments")),
                                                    (stats.members, t("用户", "Users")),
                                                    (stats.categories, t("分类", "Categories")),
                                                ]
                                                .into_iter()
                                                .map(|(count, label)| {
                                                    View::new(
                                                        blog_interface::html::tr().child((
                                                            blog_interface::html::th()
                                                                .scope("row")
                                                                .child(label),
                                                            blog_interface::html::td().child(count),
                                                        )),
                                                    )
                                                })
                                                .collect_view(),
                                            ),
                                        )),
                                ),
                        )
                    })
                },
                blog_interface::html::div()
                    .class("blog-dashboard-sections")
                    .child(
                        Group::ALL
                            .into_iter()
                            .filter(|group| *group != Group::Overview)
                            .map(|group| {
                                View::new(
                                    blog_interface::html::section()
                                        .class("blog-content-panel blog-frame blog-stack")
                                        .child((
                                            blog_interface::html::h2()
                                                .class("blog-row")
                                                .child(group.title()),
                                            blog_interface::html::nav()
                                                .aria_label(group.title())
                                                .child(
                                                    group
                                                        .pages()
                                                        .iter()
                                                        .copied()
                                                        .map(|page| {
                                                            View::new(
                                                                blog_interface::html::a()
                                                                    .class("blog-button")
                                                                    .child(page.label())
                                                                    .href(href(&format!(
                                                                        "/admin#{}",
                                                                        page.hash()
                                                                    ))),
                                                            )
                                                        })
                                                        .collect_view(),
                                                ),
                                        )),
                                )
                            })
                            .collect_view(),
                    ),
            )),
    )
}
