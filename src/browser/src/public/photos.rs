//! Photo browsing, category filters and paged collection state.
use super::shell::Chrome;
use crate::common::{Feedback, Lightbox, enc, list, load, num, rows, safe_url, t, text};
use blog_interface::children::ToChildren;
use blog_interface::component::component_props_builder;
use blog_interface::ev;
use blog_interface::html;
use blog_interface::prelude::*;
use serde_json::Value;

#[component]
fn PhotoGrid(items: Signal<Vec<Value>>, lightbox: RwSignal<Option<String>>) -> impl IntoView {
    View::new(
        html::div()
            .child((
                move || {
                    items
                        .get()
                        .into_iter()
                        .map(|item| {
                            let cover = safe_url(&text(&item, "cover"), true);
                            let target = cover.clone();
                            let title = {
                                let n = text(&item, "title");
                                if n.is_empty() {
                                    let c = text(&item, "classify");
                                    if c.is_empty() {
                                        t("相册照片", "Album photo")
                                    } else {
                                        c
                                    }
                                } else {
                                    n
                                }
                            };
                            View::new(
                                html::button()
                                    .child((
                                        html::span()
                                            .child(if let Some(url) = cover.clone() {
                                                View::new(
                                                    html::img()
                                                        .src(url)
                                                        .alt("")
                                                        .loading("lazy")
                                                        .decoding("async"),
                                                )
                                                .into_any()
                                            } else {
                                                View::new(html::span().child(t(
                                                    "记录美好瞬间",
                                                    "Capture beautiful moments",
                                                )))
                                                .into_any()
                                            })
                                            .class("travel-photo-image"),
                                        html::span()
                                            .child((
                                                html::strong().child(title.clone()),
                                                html::small().child((
                                                    t("日期（UTC）：", "Date (UTC): "),
                                                    text(&item, "create_time")
                                                        .get(..10)
                                                        .unwrap_or_default()
                                                        .to_string(),
                                                )),
                                            ))
                                            .class("travel-photo-body"),
                                    ))
                                    .class("travel-photo-card")
                                    .r#type("button")
                                    .disabled(cover.is_none())
                                    .aria_label(format!(
                                        "{}{}",
                                        t("查看照片：", "View photo: "),
                                        title
                                    ))
                                    .on(ev::click, move |_| lightbox.set(target.clone())),
                            )
                        })
                        .collect_view()
                },
                move || {
                    items.get().is_empty().then(|| {
                        View::new(
                            html::div()
                                .child((
                                    html::span().child("▧").aria_hidden("true"),
                                    html::h3().child(t(
                                        "这一组还没有照片",
                                        "No photos in this collection yet",
                                    )),
                                    html::p().child(t(
                                        "切换其它相册，或稍后回来看看新的回忆。",
                                        "Try another collection, or come back for new memories.",
                                    )),
                                ))
                                .class("travel-photo-empty public-empty-state"),
                        )
                    })
                },
            ))
            .class("travel-grid"),
    )
}

#[component]
pub(super) fn Photos() -> impl IntoView {
    View::new(Chrome(
        component_props_builder(&Chrome)
            .plain(true)
            .title(Signal::derive(|| t("时光相册", "Photo album")))
            .children(ToChildren::to_children(move || {
                html::div()
                    .child((
                        html::section()
                            .child(html::div().child((
                                html::h1().child(t("时光相册", "Photo album")),
                                html::h2().child(t(
                                    "每一张照片都是一次美好的记忆",
                                    "Every photo holds a memory",
                                )),
                            )))
                            .class("travel-banner"),
                        html::div()
                            .child(Collection(
                                component_props_builder(&Collection)
                                    .love(false)
                                    .active(Signal::derive(|| true))
                                    .build(),
                            ))
                            .class("travel-content"),
                    ))
                    .class("travel-page")
            }))
            .build(),
    ))
}

#[component]
pub(super) fn Collection(love: bool, active: Signal<bool>) -> impl IntoView {
    let classes = load(Signal::derive(|| {
        "/api/v1/links/classes?kind=lovePhoto".into()
    }));
    let selected = RwSignal::new(String::new());
    let page = RwSignal::new(1u64);
    let loaded = RwSignal::new(0u64);
    let items = RwSignal::new(Vec::<Value>::new());
    let total = RwSignal::new(None::<i64>);
    let all_total = RwSignal::new(None::<i64>);
    let lightbox = RwSignal::new(None::<String>);
    Effect::new(move |_| {
        if !love && let Some(v) = classes.value.get() {
            let first = v
                .as_array()
                .and_then(|a| a.first())
                .map(|v| text(v, "classify"))
                .unwrap_or_default();
            if selected.get_untracked().is_empty() {
                selected.set(first)
            }
        }
    });
    Effect::new(move |_| {
        selected.get();
        page.set(1);
        loaded.set(0);
        items.set(vec![]);
        total.set(None);
    });
    let data = load(Signal::derive(move || {
        if active.get() {
            format!(
                "/api/v1/links/page?kind=lovePhoto&page={}&size=12{}",
                page.get(),
                if selected.get().is_empty() {
                    String::new()
                } else {
                    format!("&classify={}", enc(&selected.get()))
                }
            )
        } else {
            String::new()
        }
    }));
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            let next = rows(&v, "items");
            total.set(Some(num(&v, "total")));
            if selected.get_untracked().is_empty() {
                all_total.set(Some(num(&v, "total")));
            }
            loaded.set(page.get_untracked());
            items.update(|items| {
                if page.get_untracked() == 1 {
                    *items = next
                } else {
                    for n in next {
                        if !items.iter().any(|v| v["id"] == n["id"]) {
                            items.push(n)
                        }
                    }
                }
            });
        }
    });
    View::new(
        html::div()
            .child((
                html::div()
                    .child((
                        html::button()
                            .child((
                                if love {
                                    t("全部", "All")
                                } else {
                                    t("全部照片", "All photos")
                                },
                                " ",
                                html::small().child(move || {
                                    all_total.get().map(|n| n.to_string()).unwrap_or_default()
                                }),
                            ))
                            .class(move || {
                                if selected.get().is_empty() {
                                    "active"
                                } else {
                                    ""
                                }
                            })
                            .r#type("button")
                            .aria_pressed(move || selected.get().is_empty().to_string())
                            .on(ev::click, move |_| selected.set(String::new())),
                        move || {
                            list(classes.value.get())
                                .into_iter()
                                .map(|c| {
                                    let title = text(&c, "classify");
                                    let checked = title.clone();
                                    let click = title.clone();
                                    View::new(
                                        html::button()
                                            .child((
                                                title.clone(),
                                                " ",
                                                html::small().child(num(&c, "count")),
                                            ))
                                            .class(move || {
                                                if selected.get() == checked {
                                                    "active"
                                                } else {
                                                    ""
                                                }
                                            })
                                            .r#type("button")
                                            .aria_pressed({
                                                let checked = title.clone();
                                                move || (selected.get() == checked).to_string()
                                            })
                                            .on(ev::click, move |_| selected.set(click.clone())),
                                    )
                                })
                                .collect_view()
                        },
                    ))
                    .class(if love {
                        "original-tag-panel love-photo-filters"
                    } else {
                        "original-tag-panel"
                    })
                    .aria_label(t("相册分类", "Photo categories")),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(classes.error)
                        .public(true)
                        .fallback(t("相册分类加载失败", "Unable to load photo categories"))
                        .retry(Some(Callback::new(move |()| classes.reload())))
                        .build(),
                ),
                html::div()
                    .child((
                        html::div().child((
                            html::h2().child(move || {
                                let v = selected.get();
                                if v.is_empty() {
                                    if love {
                                        t("时光相册", "Photo album")
                                    } else {
                                        t("全部照片", "All photos")
                                    }
                                } else {
                                    v
                                }
                            }),
                            love.then(|| {
                                View::new(html::p().child(t(
                                    "收藏一起看过的风景。",
                                    "Keep the places and moments we have shared.",
                                )))
                            }),
                        )),
                        move || {
                            love.then(|| {
                                total.get().map(|n| {
                                    View::new(
                                        html::span()
                                            .child(format!("{n} {}", t("张照片", "photos"))),
                                    )
                                })
                            })
                        },
                    ))
                    .class("love-section-heading"),
                Feedback(
                    component_props_builder(&Feedback)
                        .error(data.error)
                        .public(true)
                        .fallback(t("相册加载失败", "Unable to load photos"))
                        .retry(Some(Callback::new(move |()| {
                            page.set(loaded.get_untracked() + 1);
                            data.reload();
                        })))
                        .build(),
                ),
                move || {
                    (data.busy.get() && items.get().is_empty()).then(|| {
                        View::new(
                            html::p()
                                .child(t("正在加载照片…", "Loading photos…"))
                                .class("public-loading")
                                .role("status"),
                        )
                    })
                },
                Show(
                    component_props_builder(&Show)
                        .when(move || {
                            !items.get().is_empty()
                                || (!data.busy.get() && data.error.get().is_none())
                        })
                        .children(ToChildren::to_children(move || {
                            PhotoGrid(
                                component_props_builder(&PhotoGrid)
                                    .items(Signal::derive(move || items.get()))
                                    .lightbox(lightbox)
                                    .build(),
                            )
                        }))
                        .build(),
                ),
                move || {
                    total.get().map(|total| {
                        if (items.get().len() as i64) < total {
                            View::new(
                                html::button()
                                    .child(if love {
                                        t("加载更多照片", "Load more photos")
                                    } else {
                                        t("下一页", "Next page")
                                    })
                                    .class("travel-more")
                                    .r#type("button")
                                    .disabled(move || data.busy.get())
                                    .on(ev::click, move |_| {
                                        page.set(loaded.get_untracked() + 1);
                                        data.reload();
                                    }),
                            )
                            .into_any()
                        } else if !love {
                            View::new(
                                html::p()
                                    .child(t("~~到底啦~~", "~~You have reached the end~~"))
                                    .class("travel-end"),
                            )
                            .into_any()
                        } else {
                            ().into_any()
                        }
                    })
                },
                Lightbox(component_props_builder(&Lightbox).src(lightbox).build()),
            ))
            .class(if love {
                "love-photo-collection"
            } else {
                "travel-photo-collection"
            }),
    )
}
