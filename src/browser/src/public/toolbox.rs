//! Favorite links, friend links and music browsing.
use super::appearance::{background_style, site_media_url};
use super::shell::{Chrome, site_data};
use crate::common::{
    BrowsePager, Feedback, browse, load, page_number, param, rows, safe_url, search_signal, t, text,
};
use blog_interface::children::ToChildren;
use blog_interface::component::component_props_builder;
use blog_interface::ev;
use blog_interface::html;
use blog_interface::prelude::*;
use serde_json::Value;

#[component]
pub(super) fn Toolbox() -> impl IntoView {
    let state = search_signal();
    let tab = Signal::derive(move || {
        let v = param(state, "tab");
        if v == "friends" || v == "music" {
            v
        } else {
            "favorites".into()
        }
    });
    let data = load(Signal::derive(move || {
        format!(
            "/api/v1/links/page?kind={}&page={}&size=24",
            match tab.get().as_str() {
                "friends" => "friendUrl",
                "music" => "funny",
                _ => "favorites",
            },
            page_number(state)
        )
    }));
    let site = site_data();
    let reduced_motion = web_sys::window()
        .and_then(|window| {
            window
                .match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|query| query.matches());
    View::new(Chrome(
        component_props_builder(&Chrome)
            .plain(true)
            .title(Signal::derive(|| t("百宝箱", "Toolbox")))
            .children(ToChildren::to_children(move || {
                html::div()
                    .child((
                        html::section()
                            .child((
                                move || {
                                    site_media_url(site, |site| site.media.toolbox_video.as_deref())
                                        .map(|url| {
                                            html::video()
                                                .autoplay(!reduced_motion)
                                                .muted(true)
                                                .r#loop(!reduced_motion)
                                                .playsinline(true)
                                                .controls(reduced_motion)
                                                .preload(if reduced_motion {
                                                    "none"
                                                } else {
                                                    "metadata"
                                                })
                                                .poster(site_media_url(site, |site| {
                                                    site.media.toolbox_background.as_deref()
                                                }))
                                                .src(url)
                                        })
                                },
                                html::div()
                                    .child((
                                        html::small().child(t("记录", "Articles")),
                                        html::h1().child(t("百宝箱", "Toolbox")),
                                        html::div()
                                            .child(
                                                [
                                                    (
                                                        "favorites",
                                                        t("收藏夹", "Favorites"),
                                                        t(
                                                            "将本网站添加到您的收藏夹吧",
                                                            "Add this site to your favorites",
                                                        ),
                                                    ),
                                                    (
                                                        "music",
                                                        t("音乐", "Music"),
                                                        t(
                                                            "一曲肝肠断，天涯何处觅知音",
                                                            "Music brings kindred spirits together",
                                                        ),
                                                    ),
                                                    (
                                                        "friends",
                                                        t("友人帐", "Friends"),
                                                        t(
                                                            "看看朋友们的网站",
                                                            "Explore our friends’ websites",
                                                        ),
                                                    ),
                                                ]
                                                .into_iter()
                                                .map(|(id, l, desc)| {
                                                    View::new(
                                                        html::button()
                                                            .child((
                                                                html::strong().child(l),
                                                                html::span().child(desc),
                                                            ))
                                                            .class(move || {
                                                                if tab.get() == id {
                                                                    "active"
                                                                } else {
                                                                    ""
                                                                }
                                                            })
                                                            .aria_pressed(move || {
                                                                (tab.get() == id).to_string()
                                                            })
                                                            .on(ev::click, move |_| {
                                                                browse(
                                                                    state,
                                                                    &[
                                                                        ("tab", Some(id.into())),
                                                                        ("page", None),
                                                                    ],
                                                                )
                                                            }),
                                                    )
                                                })
                                                .collect_view(),
                                            )
                                            .class("favorite-feature-links"),
                                    ))
                                    .class("favorite-banner-overlay"),
                            ))
                            .style(move || {
                                background_style(site_media_url(site, |site| {
                                    site.media.toolbox_background.as_deref()
                                }))
                            })
                            .class("favorite-banner"),
                        html::div()
                            .child((
                                Feedback(
                                    component_props_builder(&Feedback)
                                        .error(data.error)
                                        .public(true)
                                        .fallback(t("内容加载失败", "Unable to load links"))
                                        .retry(Some(Callback::new(move |()| data.reload())))
                                        .build(),
                                ),
                                move || {
                                    if tab.get() == "music" {
                                        data.value
                                            .get()
                                            .map(|v| {
                                                View::new(Music(
                                                    component_props_builder(&Music)
                                                        .items(rows(&v, "items"))
                                                        .build(),
                                                ))
                                            })
                                            .into_any()
                                    } else {
                                        let items = data
                                            .value
                                            .get()
                                            .map(|v| rows(&v, "items"))
                                            .unwrap_or_default();
                                        let mut grouped =
                                            std::collections::BTreeMap::<String, Vec<Value>>::new();
                                        for item in items {
                                            let name = text(&item, "classify");
                                            let name = if name.is_empty() {
                                                if tab.get() == "friends" {
                                                    t("🥇友情链接", "🥇 Friend links")
                                                } else {
                                                    t("我的收藏", "My favorites")
                                                }
                                            } else {
                                                name
                                            };
                                            grouped.entry(name).or_default().push(item);
                                        }
                                        if tab.get() == "friends" {
                                            let mut order = vec![
                                                t("♥️青出于蓝", "♥️ Inspiration"),
                                                t("🥇友情链接", "🥇 Friend links"),
                                            ];
                                            order.extend(
                                                grouped
                                                    .keys()
                                                    .filter(|n| !order.contains(n))
                                                    .cloned()
                                                    .collect::<Vec<_>>(),
                                            );
                                            View::new(
    html::div()
        .child(
            (
                    html::section()
                    .child((html::h2()
                    .child( t("🌸 本站信息", "🌸 Site information")),
                    html::div()
                    .child((html::p()
                    .child((html::b()
                    .child( t("网站名称",
                    "Site name")),
                    html::span()
                    .child( site.value.get()
                    .map(| v | v.web_name.clone().unwrap_or_default())
                    .unwrap_or("blog".into())),)),
                    html::p()
                    .child((html::b()
                    .child( t("网址", "URL")),
                    html::a()
                    .child( web_sys::window()
                    .unwrap().location().origin().unwrap_or_default())
                    .href(web_sys::window()
                    .unwrap().location().origin().unwrap_or_default()),)),
                    site.value.get().and_then(| v | safe_url(& v.avatar.clone()
                    .unwrap_or_default(), false)).map(| u | View::new(
    html::p()
        .child(
            (
                    html::b()
                    .child( t("头像",
                    "Avatar")),
                    html::a()
                    .child( t("查看头像",
                    "View avatar"))
                    .href(u)
                    .target("_blank").rel("noopener noreferrer")
                ,),
        ),
)),
                    html::p()
                    .child((html::b()
                    .child( t("描述",
                    "Description")),
                    html::span()
                    .child( site.value.get()
                    .map(| v | v.web_title.clone().unwrap_or_default())
                    .unwrap_or(t("有诗意地记录生活",
                    "Record life through poetry"))),)),)),))
                    .class("friend-site-info"),
                    order.into_iter().map(| name | { let items = grouped.remove(&
                    name).unwrap_or_default(); View::new(FavoriteGroup(component_props_builder(
                    &FavoriteGroup,
                )
                .name( name)
                .items( items)
                .build())) }).collect_view()
                ,),
        )
        .class("friend-page"),
)
                                        .into_any()
                                        } else {
                                            (
                                                grouped
                                                    .into_iter()
                                                    .map(|(name, items)| {
                                                        View::new(FavoriteGroup(
                                                            component_props_builder(&FavoriteGroup)
                                                                .name(name)
                                                                .items(items)
                                                                .build(),
                                                        ))
                                                    })
                                                    .collect_view(),
                                                (data.value.get().is_some()
                                                    && data.value.get().is_some_and(|v| {
                                                        rows(&v, "items").is_empty()
                                                    }))
                                                .then(|| {
                                                    View::new(
                                                        html::p()
                                                            .child(t(
                                                                "还没有收藏的内容。",
                                                                "No favorites yet.",
                                                            ))
                                                            .class("favorite-empty"),
                                                    )
                                                }),
                                            )
                                                .into_any()
                                        }
                                    }
                                },
                                BrowsePager(
                                    component_props_builder(&BrowsePager)
                                        .data(data)
                                        .state(state)
                                        .size(24)
                                        .label(t("百宝箱分页", "Toolbox pages"))
                                        .build(),
                                ),
                            ))
                            .class("favorite-groups"),
                    ))
                    .class("favorite-page")
            }))
            .build(),
    ))
}

#[component]
fn FavoriteGroup(name: String, items: Vec<Value>) -> impl IntoView {
    let empty = items.is_empty();
    View::new(
        html::section()
            .child((
                html::h2().child(name),
                html::div()
                    .child(
                        items
                            .into_iter()
                            .map(|item| {
                                let href = safe_url(&text(&item, "url"), false)
                                    .map(|url| crate::common::href(&url));
                                View::new(
                                    html::a()
                                        .child((
                                            if let Some(u) = safe_url(&text(&item, "cover"), false)
                                            {
                                                View::new(
                                                    html::img().src(u).alt("").loading("lazy"),
                                                )
                                                .into_any()
                                            } else {
                                                View::new(
                                                    html::span().class("favorite-link-placeholder"),
                                                )
                                                .into_any()
                                            },
                                            html::span().child((
                                                html::strong().child({
                                                    let n = text(&item, "title");
                                                    if n.is_empty() {
                                                        t("未命名", "Untitled")
                                                    } else {
                                                        n
                                                    }
                                                }),
                                                html::small().child(text(&item, "introduction")),
                                            )),
                                        ))
                                        .class("favorite-link-card")
                                        .href(href.clone())
                                        .target(href.is_some().then_some("_blank"))
                                        .rel("noopener noreferrer"),
                                )
                            })
                            .collect_view(),
                    )
                    .class("favorite-link-grid"),
                empty.then(|| {
                    View::new(
                        html::p()
                            .child(t("这里还没有友链。", "No friend links yet."))
                            .class("favorite-group-empty"),
                    )
                }),
            ))
            .class("favorite-link-group"),
    )
}

#[component]
fn Music(items: Vec<Value>) -> impl IntoView {
    let items = StoredValue::new(
        items
            .into_iter()
            .filter(|i| safe_url(&text(i, "url"), false).is_some())
            .collect::<Vec<_>>(),
    );
    let selected = RwSignal::new(0usize);
    View::new(
        html::div()
            .child((
                html::div()
                    .child((
                        items.get_value().is_empty().then(|| {
                            View::new(html::h2().child(t("选择一首曲目", "Select a track")))
                        }),
                        move || {
                            items.get_value().get(selected.get()).cloned().map(|item| {
                                View::new((
                                    html::h2().child(text(&item, "title")),
                                    safe_url(&text(&item, "cover"), false).map(|u| {
                                        View::new(html::img().class("music-cover").src(u).alt(""))
                                    }),
                                    html::audio()
                                        .controls(true)
                                        .src(safe_url(&text(&item, "url"), false))
                                        .on(ev::ended, move |_| {
                                            selected.update(|i| {
                                                *i = (*i + 1)
                                                    .min(items.get_value().len().saturating_sub(1))
                                            })
                                        }),
                                ))
                            })
                        },
                    ))
                    .class("wall-card"),
                html::div()
                    .child((
                        html::h2().child(t("播放列表", "Playlist")),
                        items
                            .get_value()
                            .into_iter()
                            .enumerate()
                            .map(|(i, item)| {
                                View::new(
                                    html::button()
                                        .child((
                                            text(&item, "title"),
                                            html::small().child(text(&item, "classify")),
                                        ))
                                        .class(
                                            move || {
                                                if selected.get() == i { "selected" } else { "" }
                                            },
                                        )
                                        .on(ev::click, move |_| selected.set(i)),
                                )
                            })
                            .collect_view(),
                        items
                            .get_value()
                            .is_empty()
                            .then(|| View::new(html::p().child(t("暂无曲目。", "No tracks yet.")))),
                    ))
                    .class("wall-card"),
            ))
            .class("music-layout"),
    )
}
