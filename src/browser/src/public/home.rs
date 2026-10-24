//! Homepage aggregation and configured article sections.
use super::appearance::{Avatar, site_name};
use super::articles::Card;
use super::shell::{Chrome, notices, site_data};
use crate::common::{Feedback, list, load, load_typed, query, safe_url, t, text};
use crate::models::{
    ArticleSummary, Category, HomeKind, HomeSection as HomeSectionModel, Label, Page, SiteStats,
};
use blog_interface::children::ToChildren;
use blog_interface::component::component_props_builder;
use blog_interface::ev;
use blog_interface::html;
use blog_interface::prelude::*;
use serde_json::{Value, json};

#[component]
pub(super) fn Home() -> impl IntoView {
    let site = site_data();
    let stats = load_typed::<SiteStats>(Signal::derive(|| "/api/v1/site/stats".into()));
    let sections =
        load_typed::<Vec<HomeSectionModel>>(Signal::derive(|| "/api/v1/home-sections".into()));
    let categories = load_typed::<Vec<Category>>(Signal::derive(|| "/api/v1/categories".into()));
    let recommended = load_typed::<Page<ArticleSummary>>(Signal::derive(|| {
        "/api/v1/articles?size=5&recommended=true".into()
    }));
    let labels = load_typed::<Vec<Label>>(Signal::derive(|| "/api/v1/labels".into()));
    let messages = load(Signal::derive(|| "/api/v1/tree-hole".into()));
    let push = RwSignal::new(None::<Value>);
    Effect::new(move |_| {
        let items = site
            .value
            .get()
            .map(|v| notices(&v.notices.clone().unwrap_or_default()))
            .unwrap_or_default();
        let get = |prefix: &str| {
            items
                .iter()
                .find_map(|s| s.strip_prefix(prefix).map(str::to_string))
                .unwrap_or_default()
        };
        let title = get("推送标题：");
        let link = get("推送链接：");
        if title.is_empty() || safe_url(&link, false).is_none() {
            return;
        }
        let key = format!("blog-push:{link}");
        let store = web_sys::window().unwrap().local_storage().ok().flatten();
        if store
            .as_ref()
            .is_some_and(|s| s.get_item(&key).ok().flatten().as_deref() == Some("seen"))
        {
            return;
        }
        let value = json!({"title":title,"link":link,"cover":get("推送封面：")});
        let timer = blog_interface::thread_bound::SendWrapper::new(
            blog_interface::timers::callback::Timeout::new(2000, move || {
                push.set(Some(value));
                if let Some(s) = store {
                    let _ = s.set_item(&key, "seen");
                }
            }),
        );
        on_cleanup(move || drop(timer));
    });
    View::new(Chrome(
        component_props_builder(&Chrome)
            .home(true)
            .children(ToChildren::to_children(move || {
                (
                    Feedback(
                        component_props_builder(&Feedback)
                            .error(site.error)
                            .public(true)
                            .fallback(t("网站信息加载失败", "Unable to load site information"))
                            .retry(Some(Callback::new(move |()| site.reload())))
                            .build(),
                    ),
                    Feedback(
                        component_props_builder(&Feedback)
                            .error(sections.error)
                            .public(true)
                            .fallback(t("首页栏目加载失败", "Unable to load home sections"))
                            .retry(Some(Callback::new(move |()| sections.reload())))
                            .build(),
                    ),
                    html::div()
                        .child((
                            html::aside()
                                .child((
                                    html::section()
                                        .child((
                                            html::div()
                                                .child(move || {
                                                    Avatar(
                                                        component_props_builder(&Avatar)
                                                            .name(site_name(site))
                                                            .src(
                                                                site.value
                                                                    .get()
                                                                    .and_then(|site| site.avatar)
                                                                    .unwrap_or_default(),
                                                            )
                                                            .class("aside-avatar")
                                                            .alt(t("站长头像", "Site owner avatar"))
                                                            .build(),
                                                    )
                                                })
                                                .class("aside-avatar-wrap"),
                                            html::h2().child(move || {
                                                site.value
                                                    .get()
                                                    .map(|v| v.web_name.clone().unwrap_or_default())
                                                    .filter(|s| !s.is_empty())
                                                    .unwrap_or("blog".into())
                                            }),
                                            html::div()
                                                .child((
                                                    html::span().child((
                                                        t("📖 文章", "📖 Articles"),
                                                        html::strong().child(move || {
                                                            stats
                                                                .value
                                                                .get()
                                                                .map(|v| v.article_count)
                                                                .unwrap_or(0)
                                                        }),
                                                    )),
                                                    html::span().child((
                                                        t("📒 分类", "📒 Categories"),
                                                        html::strong().child(move || {
                                                            categories
                                                                .value
                                                                .get()
                                                                .unwrap_or_default()
                                                                .len()
                                                        }),
                                                    )),
                                                    html::span().child((
                                                        t("🔥 访问量", "🔥 Views"),
                                                        html::strong().child(move || {
                                                            stats
                                                                .value
                                                                .get()
                                                                .map(|v| v.view_count)
                                                                .unwrap_or(0)
                                                        }),
                                                    )),
                                                ))
                                                .class("aside-stats"),
                                        ))
                                        .class("aside-profile"),
                                    html::form()
                                        .child((
                                            (!query("lang").is_empty()).then(|| {
                                                View::new(
                                                    html::input()
                                                        .r#type("hidden")
                                                        .name("lang")
                                                        .value(query("lang")),
                                                )
                                            }),
                                            html::label()
                                                .child(t("搜索", "Search"))
                                                .r#for("home-search"),
                                            html::div().child((
                                                html::input()
                                                    .id("home-search")
                                                    .name("q")
                                                    .maxlength("200")
                                                    .placeholder(t("搜索文章", "Search articles")),
                                                html::button()
                                                    .child("⌕")
                                                    .r#type("submit")
                                                    .aria_label(t("搜索", "Search")),
                                            )),
                                        ))
                                        .class("aside-search")
                                        .action("/search")
                                        .method("get"),
                                    move || {
                                        recommended.value.get().filter(|v| !v.items.is_empty()).map(
                                            |v| {
                                                View::new((
                                                    html::section()
                                                        .child((
                                                            html::h2()
                                                                .child(t("✧ 推荐位", "✧ Featured")),
                                                            v.items
                                                                .iter()
                                                                .take(2)
                                                                .map(|a| {
                                                                    View::new(
    html::a()
        .child(
            (
                    safe_url(& a.article_cover.clone().unwrap_or_default(), false)
                    .map(| u | View::new(
    html::img()
        .src(u)
        .alt("")
        .loading("lazy"),
)),
                    html::strong()
                    .child( a.article_title
                    .clone())
                ,),
        )
        .href(
            format!("/article/{}", a.id),
        ),
)
                                                                })
                                                                .collect_view(),
                                                        ))
                                                        .class("aside-featured"),
                                                    html::section()
                                                        .child((
                                                            html::h2().child(t(
                                                                "🔥 推荐文章",
                                                                "🔥 Recommended articles",
                                                            )),
                                                            v.items
                                                                .into_iter()
                                                                .map(|a| {
                                                                    View::new(
    html::a()
        .child(
            (
                    safe_url(& a.article_cover.clone().unwrap_or_default(), false)
                    .map(| u | View::new(
    html::img()
        .src(u)
        .alt("")
        .loading("lazy"),
)),
                    html::span()
                    .child((a.article_title.clone(),
                    html::small()
                    .child( format!("◷ {}",
                    crate ::dates::format(& a.create_time.clone()
                    .unwrap_or_default()))),))
                ,),
        )
        .href(
            format!("/article/{}", a.id),
        ),
)
                                                                })
                                                                .collect_view(),
                                                        ))
                                                        .class("aside-recommended"),
                                                ))
                                            },
                                        )
                                    },
                                    html::section()
                                        .child((
                                            html::h2().child(t("🏷 标签", "🏷 Tags")),
                                            html::div().child(move || {
                                                labels
                                                    .value
                                                    .get()
                                                    .unwrap_or_default()
                                                    .into_iter()
                                                    .take(28)
                                                    .map(|l| {
                                                        View::new(
                                                            html::a()
                                                                .child(l.label_name.clone())
                                                                .href(format!(
                                                                    "/sort?sort_id={}&label_id={}",
                                                                    l.sort_id, l.id
                                                                )),
                                                        )
                                                    })
                                                    .collect_view()
                                            }),
                                        ))
                                        .class("aside-labels"),
                                    html::section()
                                        .child((
                                            html::h2()
                                                .child(t("💬 最新弹幕", "💬 Recent messages")),
                                            html::div().child(move || {
                                                list(messages.value.get())
                                                    .into_iter()
                                                    .take(9)
                                                    .map(|m| {
                                                        View::new(
                                                            html::p().child(text(&m, "message")),
                                                        )
                                                    })
                                                    .collect_view()
                                            }),
                                        ))
                                        .class("aside-latest"),
                                    html::section()
                                        .child(move || {
                                            categories
                                                .value
                                                .get()
                                                .unwrap_or_default()
                                                .into_iter()
                                                .enumerate()
                                                .map(|(i, c)| {
                                                    View::new(
    html::a()
        .child(
            (
                    html::small()
                    .child( t("速览",
                    "Overview")),
                    html::strong()
                    .child( c.sort_name.clone()),
                    html::span()
                    .child( c.sort_description
                    .clone().unwrap_or_default())
                ,),
        )
        .style(
            format!(
                "background:{}", ["linear-gradient(110deg,#358bff,#15c6ff)",
                "linear-gradient(110deg,#18c9a7,#1eebeb)",
                "linear-gradient(110deg,#ff6655,#ffbf37)",
                "linear-gradient(110deg,#8c72e9,#d3a4f4)"] [i % 4]
            ),
        )
        .href(
            format!("/sort?sort_id={}", c.id),
        ),
)
                                                })
                                                .collect_view()
                                        })
                                        .class("aside-categories")
                                        .aria_label(t("分类速览", "Category overview")),
                                ))
                                .class("home-aside")
                                .aria_label(t("网站侧栏", "Site sidebar")),
                            html::div()
                                .child((
                                    html::div()
                                        .child((
                                            html::span().child("🔊").aria_hidden("true"),
                                            html::p().child(move || {
                                                site.value
                                                    .get()
                                                    .map(|v| {
                                                        notices(
                                                            &v.notices.clone().unwrap_or_default(),
                                                        )
                                                        .into_iter()
                                                        .filter(|s| {
                                                            ![
                                                                "推送标题：",
                                                                "推送封面：",
                                                                "推送链接：",
                                                            ]
                                                            .iter()
                                                            .any(|p| s.starts_with(p))
                                                        })
                                                        .collect::<Vec<_>>()
                                                        .join(" · ")
                                                    })
                                                    .filter(|s| !s.is_empty())
                                                    .unwrap_or(t("欢迎光临", "Welcome"))
                                            }),
                                        ))
                                        .class("announcement"),
                                    For(component_props_builder(&For)
                                        .each(move || sections.value.get().unwrap_or_default())
                                        .key(|s| s.id)
                                        .children(move |section| {
                                            View::new(HomeSection(
                                                component_props_builder(&HomeSection)
                                                    .section(section)
                                                    .build(),
                                            ))
                                        })
                                        .build()),
                                ))
                                .class("home-content"),
                        ))
                        .class("home-layout"),
                    move || {
                        push.get().map(|p| {
                            View::new(
                                html::div()
                                    .child(
                                        html::section()
                                            .child((
                                                html::button()
                                                    .child("×")
                                                    .class("push-close")
                                                    .r#type("button")
                                                    .aria_label(t(
                                                        "关闭每日推荐",
                                                        "Close daily recommendation",
                                                    ))
                                                    .on(ev::click, move |_| push.set(None)),
                                                html::h2()
                                                    .child(t("每日推荐", "Daily recommendation")),
                                                html::h3().child(text(&p, "title")),
                                                safe_url(&text(&p, "cover"), false).map(|u| {
                                                    View::new(
                                                        html::img().src(u).alt(t(
                                                            "推荐封面",
                                                            "Recommended cover",
                                                        )),
                                                    )
                                                }),
                                                html::a()
                                                    .child(t("立即前往 →", "Visit now →"))
                                                    .href(safe_url(&text(&p, "link"), false))
                                                    .target("_blank")
                                                    .rel("noopener noreferrer")
                                                    .on(ev::click, move |_| push.set(None)),
                                            ))
                                            .class("push-dialog")
                                            .role("dialog")
                                            .aria_modal("true")
                                            .aria_label(t("每日推荐", "Daily recommendation"))
                                            .on(ev::click, move |ev| ev.stop_propagation()),
                                    )
                                    .class("push-backdrop")
                                    .role("presentation")
                                    .on(ev::click, move |_| push.set(None)),
                            )
                        })
                    },
                )
            }))
            .build(),
    ))
}

#[component]
fn HomeSection(section: HomeSectionModel) -> impl IntoView {
    let kind = section.kind;
    let filter = match kind {
        HomeKind::Recommended => "&recommended=true".into(),
        HomeKind::Category => format!("&sort_id={}", section.sort_id.unwrap_or(0)),
        _ => String::new(),
    };
    let path = format!("/api/v1/articles?size=6{filter}");
    let data = load_typed::<Page<ArticleSummary>>(Signal::derive(move || path.clone()));
    let more = match kind {
        HomeKind::Recommended => "/sort?recommended=1".into(),
        HomeKind::Category => format!("/sort?sort_id={}", section.sort_id.unwrap_or(0)),
        _ => "/sort".into(),
    };
    View::new(
        html::section()
            .child((
                html::div()
                    .child((
                        html::h2().child((
                            html::span().class("section-squares").aria_hidden("true"),
                            section.title.clone(),
                        )),
                        html::a()
                            .child((
                                html::span()
                                    .child("❯❯")
                                    .class("more-chevrons")
                                    .aria_hidden("true"),
                                " ",
                                t("更多", "More"),
                            ))
                            .href(more),
                    ))
                    .class("section-heading"),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .public(true)
                        .retry(Some(Callback::new(move |()| data.reload())))
                        .build(),
                ),
                html::div()
                    .child(move || {
                        data.value
                            .get()
                            .map(|v| v.items)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|item| {
                                View::new(Card(component_props_builder(&Card).item(item).build()))
                            })
                            .collect_view()
                    })
                    .class("article-grid"),
                move || {
                    data.value.get().filter(|v| v.items.is_empty()).map(|_| {
                        View::new(
    html::div()
        .child(
            (
                    html::span()
                    .child( "✎").aria_hidden("true"),
                    html::h3()
                    .child( t("这个栏目还没有文章。",
                    "No articles in this section yet.")),
                    html::p()
                    .child( t("浏览其它分类，或到留言页分享你的想法。",
                    "Browse other categories, or leave a thought on the message wall.")),
                    html::div()
                    .child((html::a()
                    .child((t("浏览文章", "Browse articles"),
                    html::span()
                    .child( "↗").aria_hidden("true"),))
                    .href("/sort"),
                    html::a()
                    .child((t("给我留言", "Leave a message"),
                    html::span()
                    .child( "↗").aria_hidden("true"),))
                    .href("/message"),))
                ,),
        )
        .class("public-empty-state home-empty-section"),
)
                    })
                },
            ))
            .class("home-section")
            .aria_label(section.title.clone()),
    )
}
