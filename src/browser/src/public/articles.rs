//! Article cards, search, browsing, access and reading interactions.
use super::shell::Chrome;
use crate::comments::Comments;
use crate::common::{
    Action, BrowsePager, Feedback, Lightbox, Sliding, browse, enc, href, load_typed, page_number,
    param, safe_url, search_signal, t,
};
use crate::models::{
    Article as ArticleModel, ArticleAccess, ArticleSummary, Category, Label, News as NewsModel,
    Page,
};
use blog_interface::children::ToChildren;
use blog_interface::component::component_props_builder;
use blog_interface::ev;
use blog_interface::html;
use blog_interface::modal::ModalSurface;
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use serde_json::json;
use wasm_bindgen::JsCast;

fn article_excerpt(value: &str, limit: usize) -> String {
    let mut plain = value.to_string();
    // These fixed expressions are shared across cards, rather than compiled
    // four times for every article on every reactive render.
    static RULES: std::sync::LazyLock<[(regex::Regex, &str); 4]> = std::sync::LazyLock::new(|| {
        [
            (r"!\[[^\]]*\]\([^)]*\)", " "),
            (r"\[([^\]]+)\]\([^)]*\)", "$1"),
            (r"<[^>]+>", " "),
            (r"[#>*_`~|]", " "),
        ]
        .map(|(pattern, replacement)| {
            (
                regex::Regex::new(pattern).expect("fixed excerpt pattern"),
                replacement,
            )
        })
    });
    for (pattern, replacement) in RULES.iter() {
        plain = pattern.replace_all(&plain, *replacement).into_owned();
    }
    plain = plain.split_whitespace().collect::<Vec<_>>().join(" ");
    if plain.chars().count() > limit {
        format!(
            "{}…",
            plain.chars().take(limit).collect::<String>().trim_end()
        )
    } else {
        plain
    }
}

#[component]
pub(super) fn Card(item: ArticleSummary, #[prop(default = false)] compact: bool) -> impl IntoView {
    let title = item.article_title.clone();
    View::new(
        html::a()
            .child((
                html::div()
                    .child(
                        if let Some(u) =
                            safe_url(&item.article_cover.clone().unwrap_or_default(), false)
                        {
                            View::new(html::img().src(u).alt("").loading("lazy")).into_any()
                        } else {
                            View::new(
                                html::span().child(t(
                                    "遇事不决，可问春风",
                                    "Let the spring breeze guide you",
                                )),
                            )
                            .into_any()
                        },
                    )
                    .class(if compact {
                        "original-list-cover"
                    } else {
                        "article-card-cover"
                    }),
                html::div()
                    .child((
                        html::small().child((
                            t("📅 发布于 ", "📅 Published "),
                            crate::dates::format(&item.create_time.clone().unwrap_or_default()),
                        )),
                        html::h3().child(Sliding(
                            component_props_builder(&Sliding).title(title).build(),
                        )),
                        html::p()
                            .child(format!(
                                "🔥 {}{}{}{}{}{}",
                                item.view_count,
                                t(" 热度 💬 ", " views 💬 "),
                                item.comment_count,
                                t(" 评论 🧡 ", " comments 🧡 "),
                                item.like_count,
                                t(" 点赞", " likes")
                            ))
                            .class(if compact {
                                "original-list-stats"
                            } else {
                                "article-card-stats"
                            }),
                        compact.then(|| {
                            View::new(
                                html::p()
                                    .child({
                                        let s = item.search_snippet.clone().unwrap_or_default();
                                        article_excerpt(
                                            &if s.is_empty() {
                                                item.excerpt.clone().unwrap_or_default()
                                            } else {
                                                s
                                            },
                                            140,
                                        )
                                    })
                                    .class("original-list-excerpt"),
                            )
                        }),
                        html::span()
                            .child((
                                html::span().child(format!(
                                    "📁 {}",
                                    item.sort_name.clone().unwrap_or_default()
                                )),
                                html::span().child(format!(
                                    "🏷 {}",
                                    item.label_name.clone().unwrap_or_default()
                                )),
                                (!item.view_status).then(|| {
                                    View::new(
                                        html::span()
                                            .child(t("🔒 密码文章", "🔒 Password protected")),
                                    )
                                }),
                            ))
                            .class(if compact {
                                "original-list-tags"
                            } else {
                                "article-card-tags"
                            }),
                    ))
                    .class(if compact {
                        "original-list-body"
                    } else {
                        "article-card-body"
                    }),
            ))
            .class(if compact {
                "original-list-card"
            } else {
                "article-card"
            })
            .href(href(&format!("/article/{}", item.id))),
    )
}

#[component]
pub(super) fn Article(id: i64) -> impl IntoView {
    let data = load_typed::<ArticleModel>(Signal::derive(move || format!("/api/v1/articles/{id}")));
    let access = load_typed::<ArticleAccess>(Signal::derive(move || {
        if data.error.get().is_some_and(|error| error.status == 404) {
            format!("/api/v1/articles/{id}/access")
        } else {
            String::new()
        }
    }));
    let unlocked = RwSignal::new(None::<ArticleModel>);
    let current = Signal::derive(move || unlocked.get().or(data.value.get()));
    let password = RwSignal::new(String::new());
    let action = Action::new();
    let retry = Callback::new(move |()| data.reload());
    View::new(Chrome(
        component_props_builder(&Chrome)
            .title(Signal::derive(move || {
                current
                    .get()
                    .map(|v| v.article_title.clone())
                    .unwrap_or(t("文章", "Articles"))
            }))
            .cover(Signal::derive(move || {
                current
                    .get()
                    .and_then(|v| v.article_cover.clone())
                    .unwrap_or_default()
            }))
            .metadata(current)
            .children(ToChildren::to_children(move || {
                (
                    For(component_props_builder(&For)
                        .each(move || current.get().into_iter().collect::<Vec<_>>())
                        .key(|a| (a.id, a.article_content.clone()))
                        .children(move |article| {
                            Reading(component_props_builder(&Reading).article(article).build())
                        })
                        .build()),
                    move || {
                        if current.get().is_some() {
                            return ().into_any();
                        }
                        if data.busy.get() || access.busy.get() || data.error.get().is_none() {
                            return View::new(
                                html::p()
                                    .child(t("正在加载文章…", "Loading article…"))
                                    .role("status"),
                            )
                            .into_any();
                        }
                        if data.error.get().is_some_and(|error| error.status != 404) {
                            return Feedback(
                                component_props_builder(&Feedback)
                                    .error(data.error)
                                    .public(true)
                                    .retry(Some(retry))
                                    .fallback(t("文章加载失败", "Unable to load the article"))
                                    .build(),
                            )
                            .into_any();
                        }
                        if let Some(error) = access.error.get() {
                            return if error.status == 404 {
                                View::new(
                                    html::section().role("alert").child((
                                        html::p().child(t("文章不存在", "Article not found")),
                                        html::button()
                                            .r#type("button")
                                            .child(t("重试", "Try again"))
                                            .on(ev::click, move |_| retry.run(())),
                                    )),
                                )
                                .into_any()
                            } else {
                                Feedback(
                                    component_props_builder(&Feedback)
                                        .error(access.error)
                                        .public(true)
                                        .retry(Some(Callback::new(move |()| access.reload())))
                                        .fallback(t(
                                            "文章访问状态加载失败",
                                            "Unable to check article access",
                                        ))
                                        .build(),
                                )
                                .into_any()
                            };
                        }
                        let Some(info) = access.value.get() else {
                            return View::new(
                                html::p()
                                    .child(t("正在加载文章…", "Loading article…"))
                                    .role("status"),
                            )
                            .into_any();
                        };
                        if !info.password_required {
                            // Visibility may have changed between the detail and access
                            // requests. Retry the detail; a completed probe is never loading.
                            return View::new(
                                html::section().role("alert").child((
                                    html::p().child(t(
                                        "文章加载失败，请重试。",
                                        "Unable to load the article. Please try again.",
                                    )),
                                    html::button()
                                        .r#type("button")
                                        .child(t("重试", "Try again"))
                                        .on(ev::click, move |_| retry.run(())),
                                )),
                            )
                            .into_any();
                        }
                        View::new(
                            html::form()
                                .class("reading-card password-form")
                                .child((
                                    html::h2().child(t(
                                        "这篇文章需要访问密码",
                                        "This article requires an access password",
                                    )),
                                    html::p().child(info.tips.unwrap_or_default()),
                                    Feedback(
                                        component_props_builder(&Feedback)
                                            .error(action.error)
                                            .public(true)
                                            .fallback(t(
                                                "访问密码错误或文章不可用",
                                                "Incorrect password or article unavailable",
                                            ))
                                            .build(),
                                    ),
                                    html::label().child((
                                        t("访问密码", "Access password"),
                                        html::input()
                                            .r#type("password")
                                            .required(true)
                                            .prop("value", move || password.get())
                                            .on(ev::input, move |ev| {
                                                password.set(event_target_value(&ev))
                                            }),
                                    )),
                                    html::button()
                                        .r#type("submit")
                                        .child(t("阅读文章", "Read article"))
                                        .disabled(move || action.busy.get()),
                                ))
                                .on(ev::submit, move |ev| {
                                    ev.prevent_default();
                                    action.run_typed(
                                        format!("/api/v1/articles/{id}/unlock"),
                                        "POST",
                                        Some(json!({ "password": password.get_untracked() })),
                                        move |value| {
                                            unlocked.set(Some(value));
                                            password.set(String::new());
                                        },
                                    );
                                }),
                        )
                        .into_any()
                    },
                )
            }))
            .build(),
    ))
}

fn reading_action(
    target: Option<web_sys::Element>,
    lightbox: RwSignal<Option<String>>,
    copy_status: RwSignal<String>,
) {
    let Some(target) = target else { return };
    if let Ok(Some(button)) = target.closest(".article-copy-code") {
        let content = button
            .parent_element()
            .and_then(|e| e.query_selector("code").ok().flatten())
            .and_then(|c| c.text_content())
            .unwrap_or_default();
        blog_interface::task::spawn_local(async move {
            let result = wasm_bindgen_futures::JsFuture::from(
                web_sys::window()
                    .unwrap()
                    .navigator()
                    .clipboard()
                    .write_text(&content),
            )
            .await;
            copy_status.set(if result.is_ok() {
                t("代码已复制", "Code copied")
            } else {
                t(
                    "复制失败，请手动选中代码",
                    "Copy failed. Select the code manually.",
                )
            });
            blog_interface::timers::callback::Timeout::new(2000, move || {
                let _ = copy_status.try_set(String::new());
            })
            .forget();
        });
        return;
    }
    if let Ok(Some(img)) = target.closest("img") {
        lightbox.set(img.get_attribute("src").and_then(|s| safe_url(&s, false)));
    }
}

#[component]
fn Reading(article: ArticleModel) -> impl IntoView {
    let rendered = crate::browser::reading_markup(
        &article.article_content,
        &t("复制代码", "Copy code"),
        &t("查看图片：", "View image: "),
        &t("文章插图", "Article illustration"),
    );
    let headings = StoredValue::new(rendered.headings);
    let node = NodeRef::<html::Article>::new();
    let progress = RwSignal::new(0i64);
    let active = RwSignal::new(String::new());
    let lightbox = RwSignal::new(None::<String>);
    let copyright = RwSignal::new(false);
    let copied = RwSignal::new(String::new());
    let update = move || {
        let Some(card) = node.get() else { return };
        let win = web_sys::window().unwrap();
        let scroll = win.scroll_y().unwrap_or(0.);
        let start = card.get_bounding_client_rect().top() + scroll;
        let end = (start + card.offset_height() as f64
            - win
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(1.))
        .max(start + 1.);
        progress.set(
            ((scroll - start) / (end - start) * 100.)
                .clamp(0., 100.)
                .round() as i64,
        );
        let doc = win.document().unwrap();
        let current = headings
            .get_value()
            .into_iter()
            .rfind(|h| {
                doc.get_element_by_id(&h.id)
                    .is_some_and(|e| e.get_bounding_client_rect().top() <= 150.)
            })
            .map(|h| h.id)
            .unwrap_or_default();
        active.set(current);
    };
    let scroll = SendWrapper::new(blog_interface::events::EventListener::new(
        &web_sys::window().unwrap(),
        "scroll",
        move |_| update(),
    ));
    let resize = SendWrapper::new(blog_interface::events::EventListener::new(
        &web_sys::window().unwrap(),
        "resize",
        move |_| update(),
    ));
    on_cleanup(move || {
        drop(scroll);
        drop(resize)
    });
    let outline = move || {
        View::new(
            html::ol().child(
                headings
                    .get_value()
                    .into_iter()
                    .map(|h| {
                        let id = h.id;
                        let id2 = id.clone();
                        View::new(
                            html::li()
                                .child(
                                    html::a()
                                        .child(h.title)
                                        .class(
                                            move || if active.get() == id2 { "active" } else { "" },
                                        )
                                        .href(format!("#{id}")),
                                )
                                .class(format!("toc-level-{}", h.level)),
                        )
                    })
                    .collect_view(),
            ),
        )
    };
    let id = article.id;
    View::new((
        IntoMaybeErased::into_maybe_erased(
            html::div()
                .child(html::span().style(move || format!("width:{}%", progress.get())))
                .class("reading-progress")
                .role("progressbar")
                .aria_label(t("阅读进度", "Reading progress"))
                .aria_valuemin("0")
                .aria_valuemax("100")
                .aria_valuenow(move || progress.get()),
        ),
        IntoMaybeErased::into_maybe_erased(
            html::div()
                .child((
                    html::article()
                        .child((
                            html::a()
                                .child(t("← 返回文章", "← Back to articles"))
                                .href(href("/")),
                            safe_url(&article.video_url.clone().unwrap_or_default(), false).map(
                                |u| {
                                    View::new(
                                        html::video()
                                            .class("article-video")
                                            .controls(true)
                                            .preload("metadata")
                                            .poster(safe_url(
                                                &article.article_cover.clone().unwrap_or_default(),
                                                false,
                                            ))
                                            .src(u),
                                    )
                                },
                            ),
                            (!article.password_required).then(|| {
                                View::new(News(component_props_builder(&News).id(id).build()))
                            }),
                            html::div()
                                .class("article-markdown")
                                .inner_html(rendered.html)
                                .on(ev::click, move |ev| {
                                    reading_action(
                                        ev.target()
                                            .and_then(|t| t.dyn_into::<web_sys::Element>().ok()),
                                        lightbox,
                                        copied,
                                    )
                                })
                                .on(ev::keydown, move |ev| {
                                    if ["Enter", " "].contains(&ev.key().as_str())
                                        && ev.target().is_some_and(|e| {
                                            e.is_instance_of::<web_sys::HtmlImageElement>()
                                        })
                                    {
                                        ev.prevent_default();
                                        reading_action(
                                            ev.target().and_then(|t| {
                                                t.dyn_into::<web_sys::Element>().ok()
                                            }),
                                            lightbox,
                                            copied,
                                        )
                                    }
                                }),
                            move || {
                                (!copied.get().is_empty()).then(|| {
                                    View::new(
                                        html::span()
                                            .child(copied.get())
                                            .class("article-copy-status")
                                            .role("status"),
                                    )
                                })
                            },
                            html::div()
                                .child((
                                    html::p().child((
                                        t("文章最后更新于 ", "Last updated "),
                                        crate::dates::format(&{
                                            let u = article.update_time.clone().unwrap_or_default();
                                            if u.is_empty() {
                                                article.create_time.clone().unwrap_or_default()
                                            } else {
                                                u
                                            }
                                        }),
                                    )),
                                    html::a()
                                        .child(format!(
                                            "{} ▶ {}",
                                            article.sort_name.clone().unwrap_or_default(),
                                            article.label_name.clone().unwrap_or_default()
                                        ))
                                        .href(href(&format!(
                                            "/sort?sort_id={}&label_id={}",
                                            article.sort_id, article.label_id
                                        ))),
                                    html::blockquote().child((
                                        t("作者：", "Author: "),
                                        article.username.clone().unwrap_or_default(),
                                        html::br(),
                                        t("版权与许可请详阅 ", "For copyright and licensing, see "),
                                        html::button()
                                            .child(t("版权声明", "Copyright notice"))
                                            .r#type("button")
                                            .on(ev::click, move |_| copyright.set(true)),
                                    )),
                                ))
                                .class("article-end-info"),
                            (article.comment_status && !article.password_required).then(|| {
                                View::new(Comments(
                                    component_props_builder(&Comments).article_id(id).build(),
                                ))
                            }),
                        ))
                        .class("reading-card reading-article")
                        .node_ref(node),
                    (!headings.get_value().is_empty()).then(|| {
                        View::new(
                            html::aside()
                                .child((
                                    html::nav().child((
                                        html::h3().child(t("目录", "Contents")),
                                        outline(),
                                    )),
                                    html::details().child((
                                        html::summary().child((
                                            t("文章目录 · ", "Contents · "),
                                            move || progress.get(),
                                            "%",
                                        )),
                                        outline(),
                                    )),
                                ))
                                .class("reading-toc")
                                .aria_label(t("文章目录", "Article contents")),
                        )
                    }),
                ))
                .class("reading-layout"),
        ),
        IntoMaybeErased::into_maybe_erased(move || {
            copyright.get().then(|| {
                let close = Callback::new(move |()| copyright.set(false));
                html::div()
                    .class("article-copyright-backdrop")
                    .role("presentation")
                    .child(ModalSurface(
                        component_props_builder(&ModalSurface)
                            .class("article-copyright-dialog")
                            .label(t("版权声明", "Copyright notice"))
                            .close(close)
                            .dismiss_on_backdrop(true)
                            .children(ToChildren::to_children(move || (
                                html::button()
                                    .child("×")
                                    .r#type("button")
                                    .attr("data-blog-initial-focus", "")
                                    .aria_label(t("关闭版权声明", "Close copyright notice"))
                                    .on(ev::click, move |_| close.run(())),
                                html::h2().child(t("版权声明", "Copyright notice")),
                                html::p().child(t(
                                    "文章、图片和视频的著作权归各自原作者所有。转载或使用内容时，请遵守原作者的许可要求；本站原创内容采用署名－非商业性使用－相同方式共享 4.0 国际许可协议。",
                                    "Articles, images, and videos belong to their respective authors. Follow their license terms when using or republishing content. Original content on this site is licensed under CC BY-NC-SA 4.0 International.",
                                )),
                                html::p().child(html::a()
                                    .child(t("查看许可协议", "View license"))
                                    .href("https://creativecommons.org/licenses/by-nc-sa/4.0/")
                                    .target("_blank")
                                    .rel("noopener noreferrer")),
                            )))
                            .build(),
                    ))
                    .on(ev::click, move |event| {
                        if event.target() == event.current_target() {
                            close.run(());
                        }
                    })
            })
        }),
        IntoMaybeErased::into_maybe_erased(Lightbox(
            component_props_builder(&Lightbox).src(lightbox).build(),
        )),
    ))
}

#[component]
fn News(id: i64) -> impl IntoView {
    let data = load_typed::<Vec<NewsModel>>(Signal::derive(move || {
        format!("/api/v1/articles/{id}/news")
    }));
    View::new((
        IntoMaybeErased::into_maybe_erased(Feedback(
            component_props_builder(&Feedback)
                .error(data.error)
                .public(true)
                .fallback(t("最新进展加载失败", "Unable to load recent updates"))
                .build(),
        )),
        IntoMaybeErased::into_maybe_erased(move || {
            data.value.get().filter(|v| !v.is_empty()).map(|v| {
                View::new(
                    html::section()
                        .child((
                            html::h2().child(t("最新进展", "Recent updates")),
                            html::ol().child(
                                v.into_iter()
                                    .map(|n| {
                                        View::new(html::li().child((
                                            html::time().child(crate::dates::format(
                                                &n.create_time.clone().unwrap_or_default(),
                                            )),
                                            html::p().child(n.content.clone()),
                                        )))
                                    })
                                    .collect_view(),
                            ),
                        ))
                        .class("article-news"),
                )
            })
        }),
    ))
}

#[component]
fn Highlight(value: String, term: String) -> impl IntoView {
    if term.is_empty() {
        return value.into_any();
    }
    let Ok(re) = regex::RegexBuilder::new(&regex::escape(&term))
        .case_insensitive(true)
        .build()
    else {
        return value.into_any();
    };
    let mut views = Vec::<AnyView>::new();
    let mut previous = 0;
    for m in re.find_iter(&value) {
        views.push(value[previous..m.start()].to_string().into_any());
        views.push(View::new(html::mark().child(value[m.start()..m.end()].to_string())).into_any());
        previous = m.end();
    }
    views.push(value[previous..].to_string().into_any());
    views.collect_view().into_any()
}

#[component]
pub(super) fn Browse(search: bool) -> impl IntoView {
    let state = search_signal();
    let page = Signal::derive(move || page_number(state));
    let selected = Signal::derive(move || {
        if param(state, "recommended") == "1" {
            -1
        } else {
            param(state, "sort_id").parse::<i64>().unwrap_or(0)
        }
    });
    let label = Signal::derive(move || param(state, "label_id").parse::<i64>().unwrap_or(0));
    let term = Signal::derive(move || param(state, "q").trim().to_string());
    let draft = RwSignal::new(term.get_untracked());
    Effect::new(move |_| draft.set(term.get()));
    let categories = load_typed::<Vec<Category>>(Signal::derive(move || {
        if search {
            String::new()
        } else {
            "/api/v1/categories".into()
        }
    }));
    let labels = load_typed::<Vec<Label>>(Signal::derive(move || {
        if !search && selected.get() > 0 {
            format!("/api/v1/labels?sort_id={}", selected.get())
        } else {
            String::new()
        }
    }));
    let data = load_typed::<Page<ArticleSummary>>(Signal::derive(move || {
        if search {
            let t = term.get();
            if t.is_empty() {
                return String::new();
            }
            format!(
                "/api/v1/articles?search={}&page={}&size=12",
                enc(&t),
                page.get()
            )
        } else {
            let filter = if selected.get() == -1 {
                "&recommended=true".into()
            } else if selected.get() > 0 {
                format!(
                    "&sort_id={}{}",
                    selected.get(),
                    if label.get() > 0 {
                        format!("&label_id={}", label.get())
                    } else {
                        String::new()
                    }
                )
            } else {
                String::new()
            };
            format!("/api/v1/articles?page={}&size=12{filter}", page.get())
        }
    }));
    let title = Signal::derive(move || {
        if search {
            t("搜索文章", "Search articles")
        } else {
            categories
                .value
                .get()
                .unwrap_or_default()
                .iter()
                .find(|c| c.id == selected.get())
                .map(|c| c.sort_name.clone())
                .unwrap_or(t("文章分类", "Article categories"))
        }
    });
    let choose = Callback::new(move |id: i64| {
        browse(
            state,
            &[
                ("sort_id", (id > 0).then(|| id.to_string())),
                ("recommended", (id == -1).then(|| "1".into())),
                ("label_id", None),
                ("page", None),
            ],
        )
    });
    View::new(Chrome(component_props_builder(&Chrome)
                .title( title)
                .subtitle(

                    Signal::derive(move || if search {
                                t(
                                    "在文字里寻找熟悉的故事。",
                                    "Find familiar stories in these words.",
                                )
                            } else {
                                t(
                                    "就算风吹散了冰雪，想念也会留下来。",
                                    "Memories remain after the snow has melted.",
                                )
                            }),
                )
                .children(ToChildren::to_children(move || (
                            if search { View::new(
    html::form()
        .child(
            (
                    html::input()
                    .aria_label(t("搜索标题或正文",
                    "Search titles or content")).prop("value",
                    move | | draft.get())
                    .maxlength("200")
                    .placeholder(t("搜索标题或正文",
                    "Search titles or content"))
                    .on(ev::input, move | ev | draft
                    .set(event_target_value(& ev))),
                    html::button()
                    .child( t("搜索",
                    "Search")).r#type("submit")
                ,),
        )
        .class("search-form")
        .on(
            ev::submit,
            move |ev| {
                ev.prevent_default();
                let next = draft.get_untracked().trim().to_string();
                if next == term.get_untracked() && page.get_untracked() == 1 {
                    data.reload();
                }
                browse(
                    state,
                    &[("q", (!next.is_empty()).then_some(next)), ("page", None)],
                );
            },
        ),
) .into_any()
                            } else { View::new(
    (
            html::div()
            .child((html::button()
            .child((t("全部 ", "All "),
            html::small()
            .child( move | | categories.value.get().unwrap_or_default().iter().map(| c | c
            .article_count).sum:: < i64 > ()),))
            .class(move | | if selected.get() == 0
            { "active" } else { "" }).on(ev::click, move
            | _ | choose.run(0)),
            html::button()
            .child( t("推荐", "Recommended")).class(move | | if selected.get()
            == - 1 { "active" } else { "" })
            .on(ev::click, move | _ | choose.run(- 1)),
            move | | categories.value.get().unwrap_or_default().into_iter().map(| c |
            { let id = c.id; View::new(
    html::button()
        .child(
            (
                    c.sort_name.clone(),
                    " ",
                    html::small()
                    .child( c.article_count)
                ,),
        )
        .class(
            move || if selected.get() == id { "active" } else { "" },
        )
        .on(ev::click, move |_| choose.run(id)),
) }).collect_view(),))
            .class("original-tag-panel original-category-panel"),
            move | | (! labels.value.get().unwrap_or_default().is_empty()).then(| |
            View::new(
    html::div()
        .child(
            (
                    html::button()
                    .child( t("全部标签 ",
                    "All tags ")).class(move |
                    | if label.get() == 0 { "active" } else { "" })
                    .on(ev::click, move | _ |
                    browse(state, & [("label_id", None), ("page", None)])),
                    move | | labels.value.get().unwrap_or_default().into_iter().map(|
                    l | { let id = l.id; View::new(
    html::button()
        .child(
            (
                    l.label_name.clone(),
                    " ",
                    html::small()
                    .child( l.article_count
                    .unwrap_or(0))
                ,),
        )
        .class(
            move || if label.get() == id { "active" } else { "" },
        )
        .on(
            ev::click,
            move |_| browse(
                state,
                &[("label_id", Some(id.to_string())), ("page", None)],
            ),
        ),
) })
                    .collect_view()
                ,),
        )
        .class("original-tag-panel original-label-panel"),
))
        ,),
)
                            .into_any() },
                            Feedback(component_props_builder(& Feedback)
                            .error( categories.error)
                            .public( true)
                            .retry( Some(Callback::new(move |
                            () | categories.reload()))).build()),
                            Feedback(component_props_builder(& Feedback)
                            .error( labels.error)
                            .public( true)
                            .retry( Some(Callback::new(move |
                            () | labels.reload()))).build()),
                            Feedback(component_props_builder(& Feedback)
                            .error( data.error)
                            .public( true)
                            .fallback( if search {
                            t("搜索失败", "Search failed") } else {
                            t("文章加载失败", "Unable to load articles") })
                            .retry( Some(Callback::new(move |
                            () | data.reload()))).build()),
                            move | | data.busy.get().then(| | View::new(
    html::p()
        .child(

            if search {
                        t("正在搜索…", "Searching…")
                    } else {
                        t("正在加载文章…", "Loading articles…")
                    },
        )
        .role("status"),
)),
                            move | | (search && ! term.get().is_empty()).then(| | data
                            .value.get().map(| v | View::new(
    html::p()
        .child(

            format!(
                        "“{}{}{}{}", term.get(), t("” 找到 ", "” found "), v
                        .total, t(" 篇文章；标题匹配优先。",
                        " articles; title matches come first.")
                    ),
        )
        .class("original-search-count"),
))),
                            html::div()
                            .child((html::h2()
                            .child( t("🍃 发现", "🍃 Discover")),
                            move | | data.value.get().map(| v | v.items)
                            .unwrap_or_default().into_iter().map(| item | if search {
                            View::new(
    html::a()
        .child(
            (
                    html::div()
                    .child( safe_url(& item
                    .article_cover.clone().unwrap_or_default(), false).map(| u |
                    View::new(
    html::img()
        .src(u)
        .alt("")
        .loading("lazy"),
)))
                    .class("original-list-cover"),
                    html::div()
                    .child((html::small()
                    .child((if item.search_snippet.clone().unwrap_or_default().is_empty() {
                    t("标题匹配", "Title match") } else { t("正文匹配",
                    "Content match") },
                    " · ",
                    crate ::dates::format(& item.create_time.clone()
                    .unwrap_or_default()),)),
                    html::h3()
                    .child( Highlight(component_props_builder(& Highlight)
                    .value( item.article_title.clone())
                    .term( term.get()).build())),
                    (! item.search_snippet.clone().unwrap_or_default().is_empty())
                    .then(| | View::new(
    html::p()
        .child(

            Highlight(component_props_builder(
                                    &Highlight,
                                )
                                .value(

                                    item.search_snippet.clone().unwrap_or_default(),
                                )
                                .term( term.get())
                                .build()),
        )
        .class("search-snippet"),
)),
                    html::span()
                    .child( if item.view_status
                    { t("公开", "Public") } else { t("需要访问密码",
                    "Access password required") })
                    .class("original-list-tag"),)).class("original-list-body")
                ,),
        )
        .class("original-list-card")
        .href(href(&format!("/article/{}", item.id))),
)
                            .into_any() } else { View::new(Card(component_props_builder(&Card)
                .item( item)
                .compact( true)
                .build()))
                            .into_any() }).collect_view(),
                            move | | ((search && term.get().is_empty()) || data.value
                            .get().is_some_and(| v | v.items.is_empty())).then(| | View::new(
    html::div()
        .child(
            (
                    html::span()
                    .child( "⌕").aria_hidden("true"),
                    html::h3()
                    .child( if search { if term
                    .get().is_empty() { t("输入关键词开始搜索",
                    "Enter a keyword to start searching") } else {
                    t("没有找到匹配的文章", "No matching articles") } } else { t("还没有文章", "No articles yet") }),
                    html::p()
                    .child( if search { if term
                    .get().is_empty() {
                    t("可以搜索文章标题，也可以寻找正文里的一句话。",
                    "Search for an article title or a phrase from its content.") }
                    else { t("试试其它关键词，或浏览全部文章。",
                    "Try another keyword, or browse all articles.") } } else {
                    t("第一篇记录正在路上。",
                    "The first story is on its way.") }),
                    html::a()
                    .child((t("浏览全部文章", "Browse all articles"),
                    html::span()
                    .child( "↗").aria_hidden("true"),))
                    .href(href("/sort"))
                ,),
        )
        .class("public-empty-state search-empty-state"),
)),
                            BrowsePager(component_props_builder(& BrowsePager)
                            .data( data)
                            .state( state)
                            .size( 12)
                            .always( true)
                            .empty( search).build()),)).class("original-article-list")
                        ,)))
                .build()))
}
