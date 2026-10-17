use crate::common::Action;
use crate::models::{
    Article as ArticleModel, ArticleAccess, ArticleSummary, Category, HomeKind,
    HomeSection as HomeSectionModel, Label, News as NewsModel, Page, Site, SiteStats,
};
use crate::{comments::Comments, common::*};
use leptos::component::{component_props_builder, component_view};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde_json::{Value, json};
use wasm_bindgen::JsCast;
#[component]
pub fn Public() -> impl IntoView {
    let site = load_typed::<Site>(Signal::derive(|| "/api/v1/site".into()));
    provide_context(site);
    match pathname().as_str(){"/"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Home,
        {
            let mut props = component_props_builder(&Home)
                .build();
            props
        },
    )
}).into_any(),"/sort"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Browse,
        {
            let mut props = component_props_builder(&Browse)
                .search(#[allow(unused_braces)] { false })
                .build();
            props
        },
    )
}).into_any(),"/search"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Browse,
        {
            let mut props = component_props_builder(&Browse)
                .search(#[allow(unused_braces)] { true })
                .build();
            props
        },
    )
}).into_any(),"/jotting"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Journal,
        {
            let mut props = component_props_builder(&Journal)
                .build();
            props
        },
    )
}).into_any(),"/menory"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Journey,
        {
            let mut props = component_props_builder(&Journey)
                .build();
            props
        },
    )
}).into_any(),"/message"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Message,
        {
            let mut props = component_props_builder(&Message)
                .build();
            props
        },
    )
}).into_any(),"/travel"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Photos,
        {
            let mut props = component_props_builder(&Photos)
                .love(#[allow(unused_braces)] { false })
                .build();
            props
        },
    )
}).into_any(),"/favorite"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Toolbox,
        {
            let mut props = component_props_builder(&Toolbox)
                .build();
            props
        },
    )
}).into_any(),"/love"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Love,
        {
            let mut props = component_props_builder(&Love)
                .build();
            props
        },
    )
}).into_any(),"/letter"=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Letter,
        {
            let mut props = component_props_builder(&Letter)
                .build();
            props
        },
    )
}).into_any(),path if path.starts_with("/article/")=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Article,
        {
            let mut props = component_props_builder(&Article)
                .id(
                    #[allow(unused_braces)]
                    { { path[9..].parse::<i64>().unwrap_or(0) } },
                )
                .build();
            props
        },
    )
}).into_any(),_=>View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Chrome,
        {
            let mut props = component_props_builder(&Chrome)
                .title(
                    #[allow(unused_braces)]
                    { Signal::derive(|| t("页面不存在", "Page not found")) },
                )
                .children({
                    ::leptos::children::ToChildren::to_children(move || {
                        leptos::html::section()
                            .child(
                                (
                                        IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({
                                        t("页面不存在", "Page not found") }) })),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::a()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({
                                        t("返回首页", "Back to home") }) })
                                        .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/"))))
                                    ,),
                            )
                            .class("public-empty-state")
                    })
                })
                .build();
            props
        },
    )
}).into_any()}
}
fn public_href(path: &str) -> String {
    path.into()
}
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
fn site_data() -> Data<Site> {
    use_context::<Data<Site>>().expect("public site context")
}
#[component]
fn Chrome(
    #[prop(default=Signal::derive(String::new))] title: Signal<String>,
    #[prop(default = false)] home: bool,
    #[prop(default = false)] plain: bool,
    #[prop(default=Signal::derive(String::new))] cover: Signal<String>,
    #[prop(default=Signal::derive(String::new))] subtitle: Signal<String>,
    #[prop(default=Signal::derive(||None))] metadata: Signal<Option<ArticleModel>>,
    children: ChildrenFn,
) -> impl IntoView {
    let site = site_data();
    let random_cover = Memo::new(move |_| {
        let covers = site
            .value
            .get()
            .map(|v| notices(&v.random_cover.clone().unwrap_or_default()))
            .unwrap_or_default();
        covers
            .get((js_sys::Math::random() * covers.len() as f64) as usize)
            .cloned()
            .unwrap_or_default()
    });
    let menu = RwSignal::new(false);
    let dark = RwSignal::new(
        web_sys::window()
            .unwrap()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|s| s.get_item("xocs-theme").ok().flatten())
            .as_deref()
            == Some("dark"),
    );
    let scrolled = RwSignal::new(false);
    let hidden = RwSignal::new(false);
    let previous = StoredValue::new(0.);
    let direction = StoredValue::new(0i32);
    let distance = StoredValue::new(0.);
    let scroll = SendWrapper::new(gloo_events::EventListener::new(
        &web_sys::window().unwrap(),
        "scroll",
        move |_| {
            let win = web_sys::window().unwrap();
            let doc = win.document().unwrap();
            let position = win.scroll_y().unwrap_or(0.).max(0.);
            let delta = position - previous.get_value();
            previous.set_value(position);
            scrolled.set(position > 40.);
            if position <= 48.
                || menu.get_untracked()
                || doc
                    .active_element()
                    .is_some_and(|e| e.matches(".public-header :focus-visible").unwrap_or(false))
            {
                direction.set_value(0);
                distance.set_value(0.);
                hidden.set(false);
                return;
            }
            if delta == 0. {
                return;
            }
            let next = if delta > 0. { 1 } else { -1 };
            let d = if next == direction.get_value() {
                distance.get_value() + delta.abs()
            } else {
                delta.abs()
            };
            direction.set_value(next);
            if d < 8. {
                distance.set_value(d);
                return;
            }
            distance.set_value(0.);
            hidden.set(next > 0);
        },
    ));
    let keys = SendWrapper::new(gloo_events::EventListener::new(
        &web_sys::window().unwrap(),
        "keydown",
        move |ev| {
            if ev
                .dyn_ref::<web_sys::KeyboardEvent>()
                .is_some_and(|e| e.key() == "Escape")
            {
                menu.set(false)
            }
        },
    ));
    on_cleanup(move || {
        drop(scroll);
        drop(keys)
    });
    let nav = [
        ("🏡", t("首页", "Home"), "/"),
        ("💖", t("家", "Home and love"), "/love"),
        ("🌈", t("游记", "Travels"), "/menory"),
        ("🏖️", t("随笔", "Journal"), "/jotting"),
        ("📒", t("记录", "Articles"), "/sort"),
        ("📸", t("相册", "Photos"), "/travel"),
        ("🧰", t("百宝箱", "Toolbox"), "/favorite"),
        ("📪", t("留言", "Messages"), "/message"),
    ];
    View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::header()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::a()
                    .child(#[allow(unused_braces)] { "xocs" }).class("brand")
                    .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/")))
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("xocs 首页",
                    "xocs home")))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::nav()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("欢迎光临",
                    "Welcome") }) }).class("public-mobile-title")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    nav.into_iter().map(| (icon, label, path) | View::new(
    leptos::html::a()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ icon }) })
                    .aria_hidden("true")),
                    IntoMaybeErased::into_maybe_erased(" "),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render(label))
                ,),
        )
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                public_href(path),
            ),
        )
        .aria_current(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                (pathname() == path).then_some("page"),
            ),
        ),
)).collect_view() })),))
                    .class(move | | if menu.get() {
                    "open" } else { "" })
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("主导航",
                    "Main navigation")))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & xcss_web::components::LanguageToggle, { let mut props =
                    component_props_builder(&
                    xcss_web::components::LanguageToggle).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] { leptos::svg::svg()
                    .child(#[allow(unused_braces)] { leptos::svg::path()
                    .attr("d",
                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | if dark.get() {
                    "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8M12 2v2m0 16v2M2 12h2m16 0h2" }
                    else { "M20.9 13A9 9 0 1 1 11 3a7 7 0 0 0 9.9 10Z" })) })
                    .attr("aria-hidden", "true").attr("width", "1em").attr("height",
                    "1em").attr("viewBox", "0 0 24 24").attr("fill", "none")
                    .attr("stroke", "currentColor").attr("stroke-width", "1.7") })
                    .class("theme-toggle").r#type("button")
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | if dark.get() { t("切换到浅色模式",
                    "Switch to light mode") } else { t("切换到深色模式",
                    "Switch to dark mode") }))
                    .title(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | if dark.get() { t("切换到浅色模式",
                    "Switch to light mode") } else { t("切换到深色模式",
                    "Switch to dark mode") }))
                    .aria_pressed(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | dark.get().to_string()))
                    .on(leptos::ev::click, move | _ | { dark
                    .update(| v | * v = ! * v); if let Some(s) = web_sys::window()
                    .unwrap().local_storage().ok().flatten() { let _ = s
                    .set_item("xocs-theme", if dark.get_untracked() { "dark" } else {
                    "light" }); } })),)).class("public-header-controls")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if menu.get() { "✕" } else { "☰" } }) })
                    .class("public-menu-toggle").r#type("button")
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | if menu.get() { t("关闭导航", "Close navigation") } else { t("打开导航", "Open navigation") }))
                    .aria_expanded(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | menu.get().to_string()))
                    .on(leptos::ev::click, move | _ | menu
                    .update(| v | * v = ! * v))),))
                    .class(move | |
                    format!("public-header{}{}{}", if scrolled.get() || menu.get() ||
                    plain { " entered" } else { "" }, if plain { " plain-header" }
                    else { "" }, if hidden.get() && !menu.get() { " header-hidden" }
                    else { "" }))),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    (! plain).then(| | View::new(
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | if home { let value = site.value.get().map(| v | v
                    .web_title.clone().unwrap_or_default()).unwrap_or_default(); if
                    value.is_empty() { t("相信记录的力量",
                    "Believe in the power of stories") } else { value } } else {
                    title.get() } }) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    home.then(| | View::new(
    leptos::html::p()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("相信记录的力量！", "Believe in the power of stories!")
                    })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "|" }).class("cursor"))
                ,),
        )
        .class("hero-poem"),
)) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | (! subtitle.get().is_empty()).then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ subtitle.get() }) },
        )
        .class("hero-subtitle"),
)) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | metadata.get().map(| a | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    format!(
                        "👤 {}　·　📅 {}　·　🔥 {}{}{}{}{}{}", a.username
                        .clone().unwrap_or_default(), crate ::dates::format(& a
                        .create_time.clone().unwrap_or_default()), a.view_count,
                        t(" 热度 · 💬 ", " views · 💬 "), a.comment_count,
                        t(" 评论 · ❤️ ", " comments · ❤️ "), a
                        .like_count, t(" 点赞", " likes")
                    )
                })
            },
        )
        .class("hero-article-meta"),
)) })),))
                    .class(move | | if metadata
                    .get().is_some() { "hero-article-info" } else { "hero-center"
                    })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    home.then(| | View::new(
    (
            IntoMaybeErased::into_maybe_erased(leptos::html::a()
            .child(#[allow(unused_braces)] { "⌄" }).class("hero-down")
            .href("#public-content")
            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("浏览文章",
            "Browse articles")))),
            IntoMaybeErased::into_maybe_erased(leptos::html::div()
            .class("hero-wave hero-wave-back").aria_hidden("true")),
            IntoMaybeErased::into_maybe_erased(leptos::html::div()
            .class("hero-wave hero-wave-front").aria_hidden("true"))
        ,),
)) }))
                ,),
        )
        .style(
            move || {
                let requested = cover.get();
                let info = site.value.get();
                let image = if requested.is_empty() {
                    let background = info
                        .and_then(|info| info.background_image)
                        .unwrap_or_default();
                    if background.is_empty() {
                        random_cover.get()
                    } else {
                        background
                    }
                } else {
                    requested
                };
                let image = safe_url(&image, false)
                    .unwrap_or_else(|| "/live/xocs-current-hero.jpg".into());
                format!(
                    "background-image:linear-gradient(#0b2c3e3d,#0b2c3e52),url({})",
                    serde_json::to_string(& image).unwrap_or_default()
                )
            },
        )
        .class(
            move || {
                format!(
                    "hero{}", if home { " home-hero" } else if metadata.get()
                    .is_some() { " article-hero" } else { " compact-hero" }
                )
            },
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::main()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ children() }) })
                    .class(if home { "public-main" } else { "public-main extra-main"
                    }).id("public-content")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | scrolled.get().then(| | View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::button()
                    .child(
                        #[allow(unused_braces)]
                        {
                            leptos::html::span()
                                .child(#[allow(unused_braces)] { "↑" })
                                .aria_hidden("true")
                        },
                    )
                    .class("back-top")
                    .r#type("button")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("返回顶部", "Back to top"),
                        ),
                    )
                    .on(
                        leptos::ev::click,
                        move |_| {
                            let opts = web_sys::ScrollToOptions::new();
                            opts.set_top(0.);
                            opts.set_behavior(web_sys::ScrollBehavior::Smooth);
                            web_sys::window()
                                .unwrap()
                                .scroll_to_with_scroll_to_options(&opts);
                        },
                    )
            },
        )
        .class("public-tools"),
)) }))
                ,),
        )
        .class(
            move || {
                format!(
                    "public-site original-public page-{}{}", { let p = pathname()
                    .chars().filter(| c | c.is_ascii_alphanumeric() ||* c =='_' ||* c
                    =='-').collect::< String > (); if p.is_empty() { "home"
                    .to_string() } else { p } }, if dark.get() { " theme-dark" } else { "" }
                )
            },
        ),
)
}
#[component]
fn Sliding(title: String) -> impl IntoView {
    let node = NodeRef::<leptos::html::Span>::new();
    Effect::new(move |_| {
        if let Some(node) = node.get() {
            let cleanup = SendWrapper::new(crate::browser::observe_sliding(
                &node.unchecked_into::<web_sys::Element>(),
            ));
            on_cleanup(move || {
                drop(cleanup);
            });
        }
    });
    View::new(
        leptos::html::span()
            .child(
                #[allow(unused_braces)]
                {
                    leptos::html::span()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({ title.clone() })
                            },
                        )
                        .class("sliding-title-text")
                },
            )
            .class("sliding-title")
            .node_ref(node)
            .title(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                title.clone(),
            )),
    )
}
#[component]
fn Card(item: ArticleSummary, #[prop(default = false)] compact: bool) -> impl IntoView {
    let title = item.article_title.clone();
    View::new(
        leptos::html::a()
            .child((
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    if let Some(u) = safe_url(
                                        &item.article_cover.clone().unwrap_or_default(),
                                        false,
                                    ) {
                                        View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt("")
        .loading("lazy"),
)
                    .into_any()
                                    } else {
                                        View::new(leptos::html::span().child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    t(
                                                        "遇事不决，可问春风",
                                                        "Let the spring breeze guide you",
                                                    )
                                                })
                                            },
                                        ))
                                        .into_any()
                                    }
                                })
                            },
                        )
                        .class(if compact {
                            "original-list-cover"
                        } else {
                            "article-card-cover"
                        }),
                ),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child((
                            IntoMaybeErased::into_maybe_erased(leptos::html::small().child((
                                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                    t("📅 发布于 ", "📅 Published ")
                                })),
                                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                    crate::dates::format(
                                        &item.create_time.clone().unwrap_or_default(),
                                    )
                                })),
                            ))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::h3().child(
                                #[allow(unused_braces)]
                                {
                                    {
                                        #[allow(unreachable_code)]
                                        #[allow(unused_mut)]
                                        #[allow(
                                            clippy::let_and_return,
                                            clippy::unit_arg,
                                            clippy::let_unit_value
                                        )]
                                        component_view(
                                            #[allow(clippy::needless_borrows_for_generic_args)]
                                            &Sliding,
                                            {
                                                let mut props = component_props_builder(&Sliding)
                                                    .title(
                                                        #[allow(unused_braces)]
                                                        {
                                                            title
                                                        },
                                                    )
                                                    .build();
                                                props
                                            },
                                        )
                                    }
                                },
                            )),
                            IntoMaybeErased::into_maybe_erased(
                                leptos::html::p()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                format!(
                                                    "🔥 {}{}{}{}{}{}",
                                                    item.view_count,
                                                    t(" 热度 💬 ", " views 💬 "),
                                                    item.comment_count,
                                                    t(" 评论 🧡 ", " comments 🧡 "),
                                                    item.like_count,
                                                    t(" 点赞", " likes")
                                                )
                                            })
                                        },
                                    )
                                    .class(if compact {
                                        "original-list-stats"
                                    } else {
                                        "article-card-stats"
                                    }),
                            ),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                compact.then(|| {
                                    View::new(
                                        leptos::html::p()
                                            .child(
                                                #[allow(unused_braces)]
                                                {
                                                    IntoRender::into_render({
                                                        let s = item
                                                            .search_snippet
                                                            .clone()
                                                            .unwrap_or_default();
                                                        article_excerpt(
                                                            &if s.is_empty() {
                                                                item.excerpt
                                                                    .clone()
                                                                    .unwrap_or_default()
                                                            } else {
                                                                s
                                                            },
                                                            140,
                                                        )
                                                    })
                                                },
                                            )
                                            .class("original-list-excerpt"),
                                    )
                                })
                            })),
                            IntoMaybeErased::into_maybe_erased(
                                leptos::html::span()
                                    .child((
                                        IntoMaybeErased::into_maybe_erased(
                                            leptos::html::span().child(
                                                #[allow(unused_braces)]
                                                {
                                                    IntoRender::into_render({
                                                        format!(
                                                            "📁 {}",
                                                            item.sort_name
                                                                .clone()
                                                                .unwrap_or_default()
                                                        )
                                                    })
                                                },
                                            ),
                                        ),
                                        IntoMaybeErased::into_maybe_erased(
                                            leptos::html::span().child(
                                                #[allow(unused_braces)]
                                                {
                                                    IntoRender::into_render({
                                                        format!(
                                                            "🏷 {}",
                                                            item.label_name
                                                                .clone()
                                                                .unwrap_or_default()
                                                        )
                                                    })
                                                },
                                            ),
                                        ),
                                        IntoMaybeErased::into_maybe_erased(
                                            IntoRender::into_render({
                                                (!item.view_status).then(|| {
                                                    View::new(leptos::html::span().child(
                                                        #[allow(unused_braces)]
                                                        {
                                                            IntoRender::into_render({
                                                                t(
                                                                    "🔒 密码文章",
                                                                    "🔒 Password protected",
                                                                )
                                                            })
                                                        },
                                                    ))
                                                })
                                            }),
                                        ),
                                    ))
                                    .class(if compact {
                                        "original-list-tags"
                                    } else {
                                        "article-card-tags"
                                    }),
                            ),
                        ))
                        .class(if compact {
                            "original-list-body"
                        } else {
                            "article-card-body"
                        }),
                ),
            ))
            .class(if compact {
                "original-list-card"
            } else {
                "article-card"
            })
            .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                public_href(&format!("/article/{}", item.id)),
            )),
    )
}
fn notices(v: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(v).unwrap_or_else(|_| {
        v.lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    })
}
#[component]
fn Home() -> impl IntoView {
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
        let key = format!("xocs-push:{link}");
        let store = web_sys::window().unwrap().local_storage().ok().flatten();
        if store
            .as_ref()
            .is_some_and(|s| s.get_item(&key).ok().flatten().as_deref() == Some("seen"))
        {
            return;
        }
        let value = json!({"title":title,"link":link,"cover":get("推送封面：")});
        let timer =
            send_wrapper::SendWrapper::new(gloo_timers::callback::Timeout::new(2000, move || {
                push.set(Some(value));
                if let Some(s) = store {
                    let _ = s.set_item(&key, "seen");
                }
            }));
        on_cleanup(move || drop(timer));
    });
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                .home(#[allow(unused_braces)] { true })
                .children({
                    ::leptos::children::ToChildren::to_children(move || (
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { site.error })
                            .public(#[allow(unused_braces)] { true })
                            .fallback(#[allow(unused_braces)] {
                            t("网站信息加载失败",
                            "Unable to load site information") })
                            .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                            () | site.reload())) }).build(); props }) }),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { sections.error })
                            .public(#[allow(unused_braces)] { true })
                            .fallback(#[allow(unused_braces)] {
                            t("首页栏目加载失败",
                            "Unable to load home sections") })
                            .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                            () | sections.reload())) }).build(); props }) }),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::aside()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::section()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child(#[allow(unused_braces)] {
                            leptos::html::img()
                            .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                            | | site.value.get().and_then(| v | safe_url(& v.avatar
                            .clone().unwrap_or_default(), false))
                            .unwrap_or("/legacy/avatar.jpg".into())))
                            .alt(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("站长头像",
                            "Site owner avatar"))) }).class("aside-avatar")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | site.value.get().map(| v | v.web_name.clone()
                            .unwrap_or_default()).filter(| s | ! s.is_empty())
                            .unwrap_or("xocs".into()) }) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::span()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            t("📖 文章", "📖 Articles") })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | stats.value.get().map(| v | v.article_count)
                            .unwrap_or(0) }) })),))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::span()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            t("📒 分类", "📒 Categories") })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | categories.value.get().unwrap_or_default().len()
                            }) })),))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::span()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            t("🔥 访问量", "🔥 Views") })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | stats.value.get().map(| v | v.view_count)
                            .unwrap_or(0) }) })),))),)).class("aside-stats")),))
                            .class("aside-profile")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::form()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            (! query("lang").is_empty()).then(| | View::new(
    leptos::html::input()
        .r#type("hidden")
        .name("lang")
        .value(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                query("lang"),
            ),
        ),
)) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::label()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({ t("搜索",
                            "Search") }) }).r#for("home-search")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::input()
                            .id("home-search").name("q").maxlength("200")
                            .placeholder(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("搜索文章",
                            "Search articles")))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::button()
                            .child(#[allow(unused_braces)] { "⌕" }).r#type("submit")
                            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("搜索",
                            "Search")))),))),)).class("aside-search").action("/search")
                            .method("get")),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | recommended.value.get().filter(| v | ! v.items
                            .is_empty()).map(| v | View::new(
    (
            IntoMaybeErased::into_maybe_erased(leptos::html::section()
            .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ t("✧ 推荐位",
            "✧ Featured") }) })),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            v.items.iter().take(2).map(| a | View::new(
    leptos::html::a()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    safe_url(& a.article_cover.clone().unwrap_or_default(), false)
                    .map(| u | View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt("")
        .loading("lazy"),
))
                    })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ a.article_title
                    .clone() }) }))
                ,),
        )
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                public_href(&format!("/article/{}", a.id)),
            ),
        ),
)).collect_view() })),)).class("aside-featured")),
            IntoMaybeErased::into_maybe_erased(leptos::html::section()
            .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ t("🔥 推荐文章",
            "🔥 Recommended articles") }) })),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            v.items.into_iter().map(| a | View::new(
    leptos::html::a()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    safe_url(& a.article_cover.clone().unwrap_or_default(), false)
                    .map(| u | View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt("")
        .loading("lazy"),
))
                    })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    a.article_title.clone() })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ format!("◷ {}",
                    crate ::dates::format(& a.create_time.clone()
                    .unwrap_or_default())) }) })),)))
                ,),
        )
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                public_href(&format!("/article/{}", a.id)),
            ),
        ),
)).collect_view()
            })),)).class("aside-recommended"))
        ,),
)) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::section()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("🏷 标签", "🏷 Tags") }) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | labels.value.get().unwrap_or_default().into_iter()
                            .take(28).map(| l | View::new(
    leptos::html::a()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ l.label_name.clone() }) },
        )
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                public_href(
                    &format!("/sort?sort_id={}&label_id={}", l.sort_id, l.id),
                ),
            ),
        ),
)).collect_view() }) })),))
                            .class("aside-labels")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::section()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("💬 最新弹幕", "💬 Recent messages") }) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | list(messages.value.get()).into_iter().take(9)
                            .map(| m | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ text(&m, "message") }) },
        ),
))
                            .collect_view() }) })),)).class("aside-latest")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::section()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | categories.value.get().unwrap_or_default()
                            .into_iter().enumerate().map(| (i, c) | View::new(
    leptos::html::a()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("速览",
                    "Overview") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ c.sort_name.clone()
                    }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ c.sort_description
                    .clone().unwrap_or_default() }) }))
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
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                public_href(&format!("/sort?sort_id={}", c.id)),
            ),
        ),
)).collect_view() }) }).class("aside-categories")
                            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("分类速览",
                            "Category overview")))),)).class("home-aside")
                            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("网站侧栏",
                            "Site sidebar")))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::span()
                            .child(#[allow(unused_braces)] { "🔊" })
                            .aria_hidden("true")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::p()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | site.value.get().map(| v | notices(& v.notices
                            .clone().unwrap_or_default()).into_iter().filter(| s | !
                            ["推送标题：", "推送封面：", "推送链接："]
                            .iter().any(| p | s.starts_with(p))).collect:: < Vec < _ > >
                            ().join(" · ")).filter(| s | ! s.is_empty())
                            .unwrap_or(t("欢迎光临", "Welcome")) }) })),))
                            .class("announcement")),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & For, { let mut props =
                            component_props_builder(& For)
                            .each(#[allow(unused_braces)] { move | | sections.value
                            .get().unwrap_or_default() }).key(#[allow(unused_braces)] {
                            | s | s.id }).children(#[allow(unused_braces)] { move |
                            section | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &HomeSection,
        {
            let mut props = component_props_builder(
                    &HomeSection,
                )
                .section(#[allow(unused_braces)] { section })
                .build();
            props
        },
    )
}) }).build();
                            props }) }),)).class("home-content")),))
                            .class("home-layout")),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | push.get().map(| p | View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::section()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(leptos::html::button()
                                .child(#[allow(unused_braces)] { "×" }).class("push-close")
                                .r#type("button")
                                .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("关闭每日推荐",
                                "Close daily recommendation")))
                                .on(leptos::ev::click, move | _ | push
                                .set(None))),
                                IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({
                                t("每日推荐", "Daily recommendation") }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ text(& p,
                                "title") }) })),
                                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                safe_url(& text(& p, "cover"), false).map(| u | View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("推荐封面", "Recommended cover"),
            ),
        ),
)) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::a()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({
                                t("立即前往 →", "Visit now →") }) })
                                .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(safe_url(&
                                text(& p, "link"), false))).target("_blank")
                                .rel("noopener noreferrer")
                                .on(leptos::ev::click, move | _ | push
                                .set(None)))
                            ,),
                    )
                    .class("push-dialog")
                    .role("dialog")
                    .aria_modal("true")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("每日推荐", "Daily recommendation"),
                        ),
                    )
                    .on(
                        leptos::ev::click,
                        move |ev| ev.stop_propagation(),
                    )
            },
        )
        .class("push-backdrop")
        .role("presentation")
        .on(leptos::ev::click, move |_| push.set(None)),
)) }))
                        ,))
                })
                .build();
                props
            },
        )
    })
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
    leptos::html::section()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .class("section-squares").aria_hidden("true")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    section.title.clone() })),))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::a()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "❯❯" })
                    .class("more-chevrons").aria_hidden("true")),
                    IntoMaybeErased::into_maybe_erased(" "),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("更多", "More") })),))
                    .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href(&
                    more)))),)).class("section-heading")),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { data.error })
                    .public(#[allow(unused_braces)] { true })
                    .retry(#[allow(unused_braces)] { Some(Callback::new(move | () |
                    data.reload())) }).build(); props }) }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    move | | data.value.get().map(| v | v.items).unwrap_or_default()
                    .into_iter().map(| item | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Card,
        {
            let mut props = component_props_builder(&Card)
                .item(#[allow(unused_braces)] { item })
                .build();
            props
        },
    )
}))
                    .collect_view() }) }).class("article-grid")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | data.value.get().filter(| v | v.items.is_empty()).map(|
                    _ | View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "✎" }).aria_hidden("true")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("这个栏目还没有文章。",
                    "No articles in this section yet.") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("浏览其它分类，或到留言页分享你的想法。",
                    "Browse other categories, or leave a thought on the message wall.")
                    }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::a()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("浏览文章", "Browse articles") })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "↗" }).aria_hidden("true")),))
                    .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/sort")))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::a()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("给我留言", "Leave a message") })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "↗" }).aria_hidden("true")),))
                    .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/message")))),)))
                ,),
        )
        .class("public-empty-state home-empty-section"),
)) }))
                ,),
        )
        .class("home-section")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                section.title.clone(),
            ),
        ),
)
}
#[component]
fn Article(id: i64) -> impl IntoView {
    let data = load_typed::<ArticleModel>(Signal::derive(move || format!("/api/v1/articles/{id}")));
    let access = load_typed::<ArticleAccess>(Signal::derive(move || {
        if data.error.get().is_some() {
            format!("/api/v1/articles/{id}/access")
        } else {
            String::new()
        }
    }));
    let unlocked = RwSignal::new(None::<ArticleModel>);
    let current = Signal::derive(move || unlocked.get().or(data.value.get()));
    let password = RwSignal::new(String::new());
    let action = Action::new();
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                    .title(
                        #[allow(unused_braces)]
                        {
                            Signal::derive(move || {
                                current
                                    .get()
                                    .map(|v| v.article_title.clone())
                                    .unwrap_or(t("文章", "Articles"))
                            })
                        },
                    )
                    .cover(
                        #[allow(unused_braces)]
                        {
                            Signal::derive(move || {
                                current
                                    .get()
                                    .map(|v| v.article_cover.clone().unwrap_or_default())
                                    .unwrap_or_default()
                            })
                        },
                    )
                    .metadata(
                        #[allow(unused_braces)]
                        {
                            current
                        },
                    )
                    .children({
                        ::leptos::children::ToChildren::to_children(move || {
                            (
                                IntoMaybeErased::into_maybe_erased({
                                    #[allow(unreachable_code)]
                                    #[allow(unused_mut)]
                                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                    component_view(
                                        #[allow(clippy::needless_borrows_for_generic_args)]
                                        &For,
                                        {
                                            let mut props =
                            component_props_builder(& For)
                            .each(#[allow(unused_braces)] { { move | | current.get()
                            .into_iter().collect:: < Vec < _ > > () } })
                            .key(#[allow(unused_braces)] { | a | (a.id, a
                            .article_content.clone()) })
                            .children(#[allow(unused_braces)] { move | article | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Reading,
        {
            let mut props = component_props_builder(&Reading)
                .article(#[allow(unused_braces)] { article })
                .build();
            props
        },
    )
}) }).build();
                                            props
                                        },
                                    )
                                }),
                                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                    move || {
                                        if current.get().is_none() {
                                            if access
                                                .value
                                                .get()
                                                .is_some_and(|v| v.password_required)
                                            {
                                                View::new(
    leptos::html::form()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("这篇文章需要访问密码",
                    "This article requires an access password") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ access.value.get()
                    .map(| v | v.tips.clone().unwrap_or_default())
                    .unwrap_or_default() }) })),
                    IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Feedback, { let mut props =
                    component_props_builder(& Feedback)
                    .error(#[allow(unused_braces)] { action.error })
                    .public(#[allow(unused_braces)] { true })
                    .fallback(#[allow(unused_braces)] {
                    t("访问密码错误或文章不可用",
                    "Incorrect password or article unavailable") }).build(); props })
                    }),
                    IntoMaybeErased::into_maybe_erased(leptos::html::label()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("访问密码", "Access password") })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::input()
                    .r#type("password").required(true).prop("value",
                    move | | password.get())
                    .on(leptos::ev::input, move | ev | password
                    .set(event_target_value(& ev)))),))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("阅读文章",
                    "Read article") }) }).r#type("submit")
                    .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | action.busy.get())))
                ,),
        )
        .class("reading-card password-form")
        .on(
            leptos::ev::submit,
            move |ev| {
                ev.prevent_default();
                action
                    .run_typed(
                        format!("/api/v1/articles/{id}/unlock"),
                        "POST",
                        Some(json!({ "password" : password.get_untracked() })),
                        move |v| {
                            unlocked.set(Some(v));
                            password.set(String::new());
                        },
                    );
            },
        ),
) .into_any()
                                            } else if access.error.get().is_some() {
                                                View::new(
                                                    leptos::html::p()
                                                        .child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    t(
                                                                        "文章不存在",
                                                                        "Article not found",
                                                                    )
                                                                })
                                                            },
                                                        )
                                                        .role("alert"),
                                                )
                                                .into_any()
                                            } else {
                                                View::new(leptos::html::p().child(
                                                    #[allow(unused_braces)]
                                                    {
                                                        IntoRender::into_render({
                                                            t("正在加载文章…", "Loading article…")
                                                        })
                                                    },
                                                ))
                                                .into_any()
                                            }
                                        } else {
                                            ().into_any()
                                        }
                                    }
                                })),
                            )
                        })
                    })
                    .build();
                props
            },
        )
    })
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
        leptos::task::spawn_local(async move {
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
            gloo_timers::callback::Timeout::new(2000, move || {
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
    let node = NodeRef::<leptos::html::Article>::new();
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
    let scroll = SendWrapper::new(gloo_events::EventListener::new(
        &web_sys::window().unwrap(),
        "scroll",
        move |_| update(),
    ));
    let resize = SendWrapper::new(gloo_events::EventListener::new(
        &web_sys::window().unwrap(),
        "resize",
        move |_| update(),
    ));
    on_cleanup(move || {
        drop(scroll);
        drop(resize)
    });
    let outline = move || {
        View::new(leptos::html::ol().child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    headings
                        .get_value()
                        .into_iter()
                        .map(|h| {
                            let id = h.id;
                            let id2 = id.clone();
                            View::new(
                                leptos::html::li()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            leptos::html::a()
                    .child(
                        #[allow(unused_braces)]
                        { IntoRender::into_render({ h.title }) },
                    )
                    .class(
                        move || {
                            if active.get() == id2 { "active" } else { "" }
                        },
                    )
                    .href(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            format!("#{id}"),
                        ),
                    )
                                        },
                                    )
                                    .class(format!("toc-level-{}", h.level)),
                            )
                        })
                        .collect_view()
                })
            },
        ))
    };
    let id = article.id;
    View::new(
    (
            IntoMaybeErased::into_maybe_erased(leptos::html::div()
            .child(#[allow(unused_braces)] { leptos::html::span()
            .style(move | | format!("width:{}%",
            progress.get())) }).class("reading-progress").role("progressbar")
            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("阅读进度",
            "Reading progress"))).aria_valuemin("0").aria_valuemax("100")
            .aria_valuenow(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
            | | progress.get()))),
            IntoMaybeErased::into_maybe_erased(leptos::html::div()
            .child((IntoMaybeErased::into_maybe_erased(leptos::html::article()
            .child((IntoMaybeErased::into_maybe_erased(leptos::html::a()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ t("← 返回文章",
            "← Back to articles") }) })
            .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/")))),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            safe_url(& article.video_url.clone().unwrap_or_default(), false).map(| u
            | View::new(
    leptos::html::video()
        .class("article-video")
        .controls(true)
        .preload("metadata")
        .poster(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                safe_url(&article.article_cover.clone().unwrap_or_default(), false),
            ),
        )
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u)),
)) })),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            (! article.password_required).then(| | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &News,
        {
            let mut props = component_props_builder(&News)
                .id(#[allow(unused_braces)] { id })
                .build();
            props
        },
    )
})) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::div()
            .class("article-markdown")
            .inner_html(::leptos::prelude::IntoAttributeValue::into_attribute_value(rendered
            .html)).on(leptos::ev::click, move | ev |
            reading_action(ev.target().and_then(| t | t.dyn_into:: < web_sys::Element
            > ().ok()), lightbox, copied)).on(leptos::ev::keydown,
            move | ev | { if ["Enter", " "].contains(& ev.key().as_str()) && ev
            .target().is_some_and(| e | e.is_instance_of:: <
            web_sys::HtmlImageElement > ()) { ev.prevent_default(); reading_action(ev
            .target().and_then(| t | t.dyn_into:: < web_sys::Element > ().ok()),
            lightbox, copied) } })),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            move | | (! copied.get().is_empty()).then(| | View::new(
    leptos::html::span()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ copied.get() }) },
        )
        .class("article-copy-status")
        .role("status"),
))
            })),
            IntoMaybeErased::into_maybe_erased(leptos::html::div()
            .child((IntoMaybeErased::into_maybe_erased(leptos::html::p()
            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            t("文章最后更新于 ", "Last updated ") })),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            crate ::dates::format(& { let u = article.update_time.clone()
            .unwrap_or_default(); if u.is_empty() { article.create_time.clone()
            .unwrap_or_default() } else { u } }) })),))),
            IntoMaybeErased::into_maybe_erased(leptos::html::a()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ format!("{} ▶ {}", article
            .sort_name.clone().unwrap_or_default(), article.label_name.clone()
            .unwrap_or_default()) }) })
            .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href(&
            format!("/sort?sort_id={}&label_id={}", article.sort_id, article
            .label_id))))),
            IntoMaybeErased::into_maybe_erased(leptos::html::blockquote()
            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            t("作者：", "Author: ") })),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            article.username.clone().unwrap_or_default() })),
            IntoMaybeErased::into_maybe_erased(leptos::html::br()),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            t("版权与许可请详阅 ", "For copyright and licensing, see ") })),
            IntoMaybeErased::into_maybe_erased(leptos::html::button()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ t("版权声明",
            "Copyright notice") }) }).r#type("button")
            .on(leptos::ev::click, move | _ | copyright
            .set(true))),))),)).class("article-end-info")),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            (article.comment_status && ! article.password_required).then(| | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Comments,
        {
            let mut props = component_props_builder(&Comments)
                .article_id(#[allow(unused_braces)] { id })
                .build();
            props
        },
    )
})) })),))
            .class("reading-card reading-article").node_ref(node)),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            (! headings.get_value().is_empty()).then(| | View::new(
    leptos::html::aside()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::nav()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("目录",
                    "Contents") }) })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    outline() })),))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::details()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::summary()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("文章目录 · ", "Contents · ") })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | progress.get() })),
                    IntoMaybeErased::into_maybe_erased("%"),))),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    outline() })),)))
                ,),
        )
        .class("reading-toc")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("文章目录", "Article contents"),
            ),
        ),
))
            })),)).class("reading-layout")),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            move | | copyright.get().then(| | View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::section()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(leptos::html::button()
                                .child(#[allow(unused_braces)] { "×" }).r#type("button")
                                .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("关闭版权声明",
                                "Close copyright notice")))
                                .on(leptos::ev::click, move | _ |
                                copyright.set(false))),
                                IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({
                                t("版权声明", "Copyright notice") }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::p()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({
                                t("文章、图片和视频的著作权归各自原作者所有。转载或使用内容时，请遵守原作者的许可要求；本站原创内容采用署名－非商业性使用－相同方式共享 4.0 国际许可协议。",
                                "Articles, images, and videos belong to their respective authors. Follow their license terms when using or republishing content. Original content on this site is licensed under CC BY-NC-SA 4.0 International.")
                                }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::p()
                                .child(#[allow(unused_braces)] {
                                leptos::html::a()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({
                                t("查看许可协议", "View license") }) })
                                .href("https://creativecommons.org/licenses/by-nc-sa/4.0/")
                                .target("_blank").rel("noopener noreferrer") }))
                            ,),
                    )
                    .role("dialog")
                    .aria_modal("true")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("版权声明", "Copyright notice"),
                        ),
                    )
                    .on(
                        leptos::ev::click,
                        move |ev| ev.stop_propagation(),
                    )
            },
        )
        .class("article-copyright-backdrop")
        .role("presentation")
        .on(leptos::ev::click, move |_| copyright.set(false)),
)) })),
            IntoMaybeErased::into_maybe_erased({
            #[allow(unreachable_code)] #[allow(unused_mut)]
            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
            & Lightbox, { let mut props =
            component_props_builder(& Lightbox)
            .src(#[allow(unused_braces)] { lightbox }).build(); props }) })
        ,),
)
}
#[component]
fn News(id: i64) -> impl IntoView {
    let data = load_typed::<Vec<NewsModel>>(Signal::derive(move || {
        format!("/api/v1/articles/{id}/news")
    }));
    View::new((
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
                        .public(
                            #[allow(unused_braces)]
                            {
                                true
                            },
                        )
                        .fallback(
                            #[allow(unused_braces)]
                            {
                                t("最新进展加载失败", "Unable to load recent updates")
                            },
                        )
                        .build();
                    props
                },
            )
        }),
        IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            move || {
                data.value.get().filter(|v| !v.is_empty()).map(|v| {
                    View::new(
                        leptos::html::section()
                            .child((
                                IntoMaybeErased::into_maybe_erased(leptos::html::h2().child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({
                                            t("最新进展", "Recent updates")
                                        })
                                    },
                                )),
                                IntoMaybeErased::into_maybe_erased(leptos::html::ol().child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({
                                            v.into_iter()
                                                .map(|n| {
                                                    View::new(leptos::html::li().child((
                                                        IntoMaybeErased::into_maybe_erased(
                                                            leptos::html::time().child(
                                                                #[allow(unused_braces)]
                                                                {
                                                                    IntoRender::into_render({
                                                                        crate::dates::format(
                                                                            &n.create_time
                                                                                .clone()
                                                                                .unwrap_or_default(
                                                                                ),
                                                                        )
                                                                    })
                                                                },
                                                            ),
                                                        ),
                                                        IntoMaybeErased::into_maybe_erased(
                                                            leptos::html::p().child(
                                                                #[allow(unused_braces)]
                                                                {
                                                                    IntoRender::into_render({
                                                                        n.content.clone()
                                                                    })
                                                                },
                                                            ),
                                                        ),
                                                    )))
                                                })
                                                .collect_view()
                                        })
                                    },
                                )),
                            ))
                            .class("article-news"),
                    )
                })
            }
        })),
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
        views.push(
            View::new(leptos::html::mark().child(
                #[allow(unused_braces)]
                {
                    IntoRender::into_render({ value[m.start()..m.end()].to_string() })
                },
            ))
            .into_any(),
        );
        previous = m.end();
    }
    views.push(value[previous..].to_string().into_any());
    views.collect_view().into_any()
}
#[component]
fn Browse(search: bool) -> impl IntoView {
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
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                .title(#[allow(unused_braces)] { title })
                .subtitle(
                    #[allow(unused_braces)]
                    {
                        Signal::derive(move || {
                            if search {
                                t(
                                    "在文字里寻找熟悉的故事。",
                                    "Find familiar stories in these words.",
                                )
                            } else {
                                t(
                                    "就算风吹散了冰雪，想念也会留下来。",
                                    "Memories remain after the snow has melted.",
                                )
                            }
                        })
                    },
                )
                .children({
                    ::leptos::children::ToChildren::to_children(move || (
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            if search { View::new(
    leptos::html::form()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::input()
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("搜索标题或正文",
                    "Search titles or content"))).prop("value",
                    move | | draft.get())
                    .maxlength("200")
                    .placeholder(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("搜索标题或正文",
                    "Search titles or content")))
                    .on(leptos::ev::input, move | ev | draft
                    .set(event_target_value(& ev)))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("搜索",
                    "Search") }) }).r#type("submit"))
                ,),
        )
        .class("search-form")
        .on(
            leptos::ev::submit,
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
            IntoMaybeErased::into_maybe_erased(leptos::html::div()
            .child((IntoMaybeErased::into_maybe_erased(leptos::html::button()
            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            t("全部 ", "All ") })),
            IntoMaybeErased::into_maybe_erased(leptos::html::small()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({
            move | | categories.value.get().unwrap_or_default().iter().map(| c | c
            .article_count).sum:: < i64 > () }) })),))
            .class(move | | if selected.get() == 0
            { "active" } else { "" }).on(leptos::ev::click, move
            | _ | choose.run(0))),
            IntoMaybeErased::into_maybe_erased(leptos::html::button()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ t("推荐", "Recommended")
            }) }).class(move | | if selected.get()
            == - 1 { "active" } else { "" })
            .on(leptos::ev::click, move | _ | choose.run(- 1))),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            move | | categories.value.get().unwrap_or_default().into_iter().map(| c |
            { let id = c.id; View::new(
    leptos::html::button()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    c.sort_name.clone() })),
                    IntoMaybeErased::into_maybe_erased(" "),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ c.article_count })
                    }))
                ,),
        )
        .class(
            move || {
                if selected.get() == id { "active" } else { "" }
            },
        )
        .on(leptos::ev::click, move |_| choose.run(id)),
) }).collect_view() })),))
            .class("original-tag-panel original-category-panel")),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            move | | (! labels.value.get().unwrap_or_default().is_empty()).then(| |
            View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("全部标签 ",
                    "All tags ") }) }).class(move |
                    | if label.get() == 0 { "active" } else { "" })
                    .on(leptos::ev::click, move | _ |
                    browse(state, & [("label_id", None), ("page", None)]))),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move | | labels.value.get().unwrap_or_default().into_iter().map(|
                    l | { let id = l.id; View::new(
    leptos::html::button()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    l.label_name.clone() })),
                    IntoMaybeErased::into_maybe_erased(" "),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ l.article_count
                    .unwrap_or(0) }) }))
                ,),
        )
        .class(
            move || {
                if label.get() == id { "active" } else { "" }
            },
        )
        .on(
            leptos::ev::click,
            move |_| browse(
                state,
                &[("label_id", Some(id.to_string())), ("page", None)],
            ),
        ),
) })
                    .collect_view() }))
                ,),
        )
        .class("original-tag-panel original-label-panel"),
)) }))
        ,),
)
                            .into_any() } })),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { categories.error })
                            .public(#[allow(unused_braces)] { true })
                            .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                            () | categories.reload())) }).build(); props }) }),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { labels.error })
                            .public(#[allow(unused_braces)] { true })
                            .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                            () | labels.reload())) }).build(); props }) }),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { data.error })
                            .public(#[allow(unused_braces)] { true })
                            .fallback(#[allow(unused_braces)] { if search {
                            t("搜索失败", "Search failed") } else {
                            t("文章加载失败", "Unable to load articles") } })
                            .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                            () | data.reload())) }).build(); props }) }),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | data.busy.get().then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    if search {
                        t("正在搜索…", "Searching…")
                    } else {
                        t("正在加载文章…", "Loading articles…")
                    }
                })
            },
        )
        .role("status"),
)) })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | (search && ! term.get().is_empty()).then(| | data
                            .value.get().map(| v | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    format!(
                        "“{}{}{}{}", term.get(), t("” 找到 ", "” found "), v
                        .total, t(" 篇文章；标题匹配优先。",
                        " articles; title matches come first.")
                    )
                })
            },
        )
        .class("original-search-count"),
))) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("🍃 发现", "🍃 Discover") }) })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | data.value.get().map(| v | v.items)
                            .unwrap_or_default().into_iter().map(| item | { if search {
                            View::new(
    leptos::html::a()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ safe_url(& item
                    .article_cover.clone().unwrap_or_default(), false).map(| u |
                    View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt("")
        .loading("lazy"),
)) }) })
                    .class("original-list-cover")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    if item.search_snippet.clone().unwrap_or_default().is_empty() {
                    t("标题匹配", "Title match") } else { t("正文匹配",
                    "Content match") } })),
                    IntoMaybeErased::into_maybe_erased(" · "),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    crate ::dates::format(& item.create_time.clone()
                    .unwrap_or_default()) })),))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                    .child(#[allow(unused_braces)] { { #[allow(unreachable_code)]
                    #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Highlight, { let mut props =
                    component_props_builder(& Highlight)
                    .value(#[allow(unused_braces)] { item.article_title.clone() })
                    .term(#[allow(unused_braces)] { term.get() }).build(); props }) }
                    })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    (! item.search_snippet.clone().unwrap_or_default().is_empty())
                    .then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                {
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Highlight,
                        {
                            let mut props = component_props_builder(
                                    &Highlight,
                                )
                                .value(
                                    #[allow(unused_braces)]
                                    { item.search_snippet.clone().unwrap_or_default() },
                                )
                                .term(#[allow(unused_braces)] { term.get() })
                                .build();
                            props
                        },
                    )
                }
            },
        )
        .class("search-snippet"),
)) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if item.view_status
                    { t("公开", "Public") } else { t("需要访问密码",
                    "Access password required") } }) })
                    .class("original-list-tag")),)).class("original-list-body"))
                ,),
        )
        .class("original-list-card")
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                public_href(&format!("/article/{}", item.id)),
            ),
        ),
)
                            .into_any() } else { View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Card,
        {
            let mut props = component_props_builder(&Card)
                .item(#[allow(unused_braces)] { item })
                .compact(#[allow(unused_braces)] { true })
                .build();
            props
        },
    )
})
                            .into_any() } }).collect_view() })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | ((search && term.get().is_empty()) || data.value
                            .get().is_some_and(| v | v.items.is_empty())).then(| | View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "⌕" }).aria_hidden("true")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if search { if term
                    .get().is_empty() { t("输入关键词开始搜索",
                    "Enter a keyword to start searching") } else {
                    t("没有找到匹配的文章", "No matching articles") } } else { t("还没有文章", "No articles yet") } }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if search { if term
                    .get().is_empty() {
                    t("可以搜索文章标题，也可以寻找正文里的一句话。",
                    "Search for an article title or a phrase from its content.") }
                    else { t("试试其它关键词，或浏览全部文章。",
                    "Try another keyword, or browse all articles.") } } else {
                    t("第一篇记录正在路上。",
                    "The first story is on its way.") } }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::a()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("浏览全部文章", "Browse all articles") })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "↗" }).aria_hidden("true")),))
                    .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/sort"))))
                ,),
        )
        .class("public-empty-state search-empty-state"),
)) })),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & BrowsePager, { let mut props =
                            component_props_builder(& BrowsePager)
                            .data(#[allow(unused_braces)] { data })
                            .state(#[allow(unused_braces)] { state })
                            .size(#[allow(unused_braces)] { 12 })
                            .always(#[allow(unused_braces)] { true })
                            .empty(#[allow(unused_braces)] { search }).build(); props })
                            }),)).class("original-article-list"))
                        ,))
                })
                .build();
                props
            },
        )
    })
}
#[component]
fn BrowsePager<T: PageMetadata + Clone + Send + Sync + 'static>(
    data: Data<T>,
    state: RwSignal<String>,
    size: i64,
    #[prop(default=t("分页","Pagination"))] label: String,
    #[prop(default = false)] always: bool,
    #[prop(default = false)] empty: bool,
) -> impl IntoView {
    move || {
        data.value
            .get()
            .filter(|v| {
                empty || v.total() > if always { 0 } else { size } || page_number(state) > 1
            })
            .map(|v| {
                let total = v.total();
                View::new(
                    leptos::html::nav()
                        .child((
                            IntoMaybeErased::into_maybe_erased(
                                leptos::html::button()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                t("上一页", "Previous page")
                                            })
                                        },
                                    )
                                    .r#type("button")
                                    .disabled(
                                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                            move || data.busy.get() || page_number(state) <= 1,
                                        ),
                                    )
                                    .on(leptos::ev::click, move |_| {
                                        let page = page_number(state).saturating_sub(1).max(1);
                                        browse(
                                            state,
                                            &[("page", (page > 1).then(|| page.to_string()))],
                                        );
                                    }),
                            ),
                            IntoMaybeErased::into_maybe_erased(leptos::html::span().child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        tr(
                                            "第 {0} 页",
                                            "Page {0}",
                                            &[page_number(state).to_string()],
                                        )
                                    })
                                },
                            )),
                            IntoMaybeErased::into_maybe_erased(
                                leptos::html::button()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({ t("下一页", "Next page") })
                                        },
                                    )
                                    .r#type("button")
                                    .disabled(
                                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                            {
                                                move || {
                                                    data.busy.get()
                                                        || (page_number(state) as i64 * size)
                                                            >= total
                                                }
                                            },
                                        ),
                                    )
                                    .on(leptos::ev::click, move |_| {
                                        browse(
                                            state,
                                            &[("page", Some((page_number(state) + 1).to_string()))],
                                        )
                                    }),
                            ),
                        ))
                        .class("pager")
                        .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            label.clone(),
                        )),
                )
            })
    }
}
#[component]
fn Journey() -> impl IntoView {
    let state = search_signal();
    let data = load(Signal::derive(move || {
        format!("/api/v1/articles?page={}&size=20", page_number(state))
    }));
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props =
                    component_props_builder(&Chrome)
                        .title(
                            #[allow(unused_braces)]
                            {
                                Signal::derive(|| "xocs".into())
                            },
                        )
                        .cover(
                            #[allow(unused_braces)]
                            {
                                Signal::derive(|| "/live/xocs-current-menory.jpg".into())
                            },
                        )
                        .children({
                            ::leptos::children::ToChildren::to_children(move || {
                                (
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Petals, { let mut props =
                            component_props_builder(& Petals)
                            .build(); props }) }),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::header()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({ t("时间线",
                            "Timeline") }) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::small()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("灵魂在路上", "The soul is walking") }) })),))),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { data.error })
                            .public(#[allow(unused_braces)] { true })
                            .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                            () | data.reload())) }).build(); props }) }),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | data.value.get().map(| v | { let items = rows(& v,
                            "items"); let len = items.len(); items.into_iter()
                            .enumerate().map(| (i, item) | { let row = i / 5; let
                            reverse = row % 2 == 1; View::new(
    leptos::html::a()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ safe_url(& text(&
                    item, "article_cover"), false).map(| u | View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt("")
        .loading("lazy"),
)) }) }).class("journey-picture")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    (i < len - 1).then(| | View::new(
    leptos::html::span()
        .class(
            if i % 5 == 4 {
                "journey-connector journey-turn"
            } else {
                "journey-connector"
            },
        )
        .aria_hidden("true"),
)) })),)).class("journey-image")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] { { #[allow(unreachable_code)]
                    #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                    & Sliding, { let mut props =
                    component_props_builder(& Sliding)
                    .title(#[allow(unused_braces)] { text(& item, "article_title") })
                    .build(); props }) } })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::time()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ crate
                    ::dates::format(& text(& item, "create_time")) }) })
                    .datetime(::leptos::prelude::IntoAttributeValue::into_attribute_value(text(&
                    item, "create_time")))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ num(& v, "total") -
                    i as i64 - (page_number(state) as i64 - 1) * 20 }) })),)))
                ,),
        )
        .style(
            format!(
                "grid-column:{};grid-row:{}", if reverse { 5 - i % 5 } else { i % 5 +
                1 }, row + 1
            ),
        )
        .class(if reverse { "journey-item journey-reverse" } else { "journey-item" })
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                public_href(&format!("/article/{}", num(& item, "id"))),
            ),
        ),
) }).collect_view() }) }) })
                            .class("journey-grid")),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & BrowsePager, { let mut props =
                            component_props_builder(& BrowsePager)
                            .data(#[allow(unused_braces)] { data })
                            .state(#[allow(unused_braces)] { state })
                            .size(#[allow(unused_braces)] { 20 }).build(); props })
                            }),)).class("journey-page"))
                        ,)
                            })
                        })
                        .build();
                props
            },
        )
    })
}
#[component]
fn Journal() -> impl IntoView {
    let state = search_signal();
    let data = load(Signal::derive(move || {
        format!("/api/v1/notes/page?page={}", page_number(state))
    }));
    let lightbox = RwSignal::new(None::<String>);
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                    .title(
                        #[allow(unused_braces)]
                        {
                            Signal::derive(|| "xocs".into())
                        },
                    )
                    .cover(
                        #[allow(unused_braces)]
                        {
                            Signal::derive(|| "/live/xocs-current-jotting.jpg".into())
                        },
                    )
                    .children({
                        ::leptos::children::ToChildren::to_children(move || {
                            (
                                IntoMaybeErased::into_maybe_erased(
                                    leptos::html::div()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { data.error })
                            .public(#[allow(unused_braces)] { true })
                            .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                            () | data.reload())) }).build(); props }) }),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | data.value.get().map(| v | rows(& v, "items"))
                            .unwrap_or_default().into_iter().map(| item | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &NoteCard,
        {
            let mut props = component_props_builder(&NoteCard)
                .item(#[allow(unused_braces)] { item })
                .lightbox(#[allow(unused_braces)] { lightbox })
                .build();
            props
        },
    )
})).collect_view() })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | data.value.get().filter(| v | rows(v, "items")
                            .is_empty()).map(| _ | View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("还没有记录。", "No entries yet.") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("新的日常记录正在路上。",
                    "New everyday stories are on their way.") }) }))
                ,),
        )
        .class("public-empty-state"),
)) })),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & BrowsePager, { let mut props =
                            component_props_builder(& BrowsePager)
                            .data(#[allow(unused_braces)] { data })
                            .state(#[allow(unused_braces)] { state })
                            .size(#[allow(unused_braces)] { 10 })
                            .label(#[allow(unused_braces)] { t("记录分页",
                            "Entry pages") }).build(); props }) }),)).class("wall-list")
                                            },
                                        )
                                        .class("wall-layout wall-reading-layout"),
                                ),
                                IntoMaybeErased::into_maybe_erased({
                                    #[allow(unreachable_code)]
                                    #[allow(unused_mut)]
                                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                    component_view(
                                        #[allow(clippy::needless_borrows_for_generic_args)]
                                        &Lightbox,
                                        {
                                            let mut props = component_props_builder(&Lightbox)
                                                .src(
                                                    #[allow(unused_braces)]
                                                    {
                                                        lightbox
                                                    },
                                                )
                                                .build();
                                            props
                                        },
                                    )
                                }),
                            )
                        })
                    })
                    .build();
                props
            },
        )
    })
}
#[component]
fn NoteCard(
    item: Value,
    lightbox: RwSignal<Option<String>>,
    #[prop(default = false)] love: bool,
) -> impl IntoView {
    View::new(
        leptos::html::article()
            .child((
                IntoMaybeErased::into_maybe_erased(leptos::html::small().child((
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                        let n = text(&item, "username");
                        if n.is_empty() {
                            t("站长", "Site owner")
                        } else {
                            n
                        }
                    })),
                    IntoMaybeErased::into_maybe_erased(" · "),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                        crate::dates::format(&text(&item, "create_time"))
                    })),
                ))),
                IntoMaybeErased::into_maybe_erased(leptos::html::p().child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({ text(&item, "content") })
                    },
                )),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    safe_url(&text(&item, "image_path"), true).map(|u| {
                        let click = u.clone();
                        View::new(
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
                            if love {
                                t("随笔图片", "Note image")
                            } else {
                                t("动态图片", "Post image")
                            },
                        ),
                    )
                    .loading("lazy")
                                    },
                                )
                                .class("wall-photo-button")
                                .r#type("button")
                                .aria_label(
                                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(t(
                                        "放大图片",
                                        "Enlarge image",
                                    )),
                                )
                                .on(leptos::ev::click, move |_| {
                                    lightbox.set(Some(click.clone()))
                                }),
                        )
                    })
                })),
            ))
            .class(if love {
                "love-moment-card"
            } else {
                "wall-card"
            }),
    )
}
#[component]
fn Message() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/tree-hole".into()));
    let draft = RwSignal::new(String::new());
    let posts = RwSignal::new(Vec::<Value>::new());
    let action = Action::new();
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            posts.update(|current| {
                for item in v.as_array().cloned().unwrap_or_default() {
                    if !current.iter().any(|v| v["id"] == item["id"]) {
                        current.push(item)
                    }
                }
            });
        }
    });
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                .plain(#[allow(unused_braces)] { true })
                .title(
                    #[allow(unused_braces)]
                    { Signal::derive(|| t("留言板", "Message board")) },
                )
                .children({
                    ::leptos::children::ToChildren::to_children(move || {
                        leptos::html::div()
                            .child(
                                (
                                        IntoMaybeErased::into_maybe_erased(leptos::html::section()
                                        .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({
                                        move | | posts.get().into_iter().enumerate().map(| (i, item)
                                        | View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    safe_url(& text(& item, "avatar"), false).map(| u | View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt(""),
)) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& item,
                    "message") }) }))
                ,),
        )
        .style(
            format!(
                "animation-delay:{}s;top:{}%",- ((i % 10) as f64) * 2.3, 8 + i % 7 *
                12
            ),
        )
        .class("message-barrage-item"),
))
                                        .collect_view() }) }).class("message-barrage")
                                        .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("留言弹幕",
                                        "Message wall")))),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::div()
                                        .child((IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({ t("弹幕",
                                        "Message wall") }) })),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::form()
                                        .child((IntoMaybeErased::into_maybe_erased(leptos::html::input()
                                        .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("发送弹幕",
                                        "Send a message"))).maxlength("60").prop("value",
                                        move | | draft.get())
                                        .on(leptos::ev::input, move | ev | draft
                                        .set(event_target_value(& ev)))
                                        .placeholder(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("留下点什么啦～",
                                        "Leave a message…")))),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::button()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({
                                        move | | if action.busy.get() { t("发射中…",
                                        "Sending…") } else { t("发射", "Send") } }) })
                                        .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                                        | | action.busy.get() || draft.get().trim()
                                        .is_empty()))),)).on(leptos::ev::submit,
                                        move | ev | { ev.prevent_default(); let content = draft
                                        .get_untracked(); if content.trim().is_empty() { return; }
                                        action.run("/api/v1/tree-hole/guest".into(), "POST",
                                        Some(json!({ "message" : content.trim() })), move | v | {
                                        posts.update(| items | { items.retain(| i | i["id"] !=
                                        v["id"]); items.insert(0, v) }); if draft.get_untracked() ==
                                        content { draft.set(String::new()) } }); })),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::p()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({
                                        t("弹幕将匿名发布，无需登录。",
                                        "Messages are posted anonymously. No account is needed.") })
                                        }).class("message-anonymous")),
                                        IntoMaybeErased::into_maybe_erased({
                                        #[allow(unreachable_code)] #[allow(unused_mut)]
                                        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                        component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                        & Feedback, { let mut props =
                                        component_props_builder(& Feedback)
                                        .error(#[allow(unused_braces)] { data.error })
                                        .public(#[allow(unused_braces)] { true })
                                        .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                                        () | data.reload())) }).build(); props }) }),
                                        IntoMaybeErased::into_maybe_erased({
                                        #[allow(unreachable_code)] #[allow(unused_mut)]
                                        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                        component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                        & Feedback, { let mut props =
                                        component_props_builder(& Feedback)
                                        .error(#[allow(unused_braces)] { action.error })
                                        .public(#[allow(unused_braces)] { true }).build(); props })
                                        }),)).class("message-form-wrap")),)).class("message-hero")),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::div()
                                        .child(#[allow(unused_braces)] { {
                                        #[allow(unreachable_code)] #[allow(unused_mut)]
                                        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                        component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                        & Comments, { let mut props =
                                        component_props_builder(& Comments)
                                        .kind(#[allow(unused_braces)] { "message" }).build(); props
                                        }) } }).class("message-comment-wrap"))
                                    ,),
                            )
                            .class("message-page")
                    })
                })
                .build();
                props
            },
        )
    })
}
#[component]
fn PhotoGrid(items: Signal<Vec<Value>>, lightbox: RwSignal<Option<String>>) -> impl IntoView {
    View::new(
        leptos::html::div()
            .child((
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
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
    leptos::html::button()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if let Some(url) =
                    cover.clone() { View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(url))
        .alt("")
        .loading("lazy")
        .decoding("async"),
) .into_any() } else { View::new(
    leptos::html::span()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("记录美好瞬间", "Capture beautiful moments")
                })
            },
        ),
) .into_any() } }) }).class("travel-photo-image")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ title.clone() })
                    })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    t("日期（UTC）：", "Date (UTC): ") })),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    text(& item, "create_time").get(..10).unwrap_or_default()
                    .to_string() })),))),)).class("travel-photo-body"))
                ,),
        )
        .class("travel-photo-card")
        .r#type("button")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                cover.is_none(),
            ),
        )
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                format!("{}{}", t("查看照片：", "View photo: "), title),
            ),
        )
        .on(
            leptos::ev::click,
            move |_| lightbox.set(target.clone()),
        ),
)
                            })
                            .collect_view()
                    }
                })),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move || {
                        items.get().is_empty().then(|| {
                            View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "▧" }).aria_hidden("true")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("这一组还没有照片", "No photos in this collection yet")
                    }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("切换其它相册，或稍后回来看看新的回忆。",
                    "Try another collection, or come back for new memories.") }) }))
                ,),
        )
        .class("travel-photo-empty public-empty-state"),
)
                        })
                    }
                })),
            ))
            .class("travel-grid"),
    )
}
#[component]
fn Photos(love: bool) -> impl IntoView {
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                .plain(#[allow(unused_braces)] { true })
                .title(
                    #[allow(unused_braces)]
                    { Signal::derive(|| t("时光相册", "Photo album")) },
                )
                .children({
                    ::leptos::children::ToChildren::to_children(move || {
                        leptos::html::div()
                            .child(
                                (
                                        IntoMaybeErased::into_maybe_erased(leptos::html::section()
                                        .child(#[allow(unused_braces)] {
                                        leptos::html::div()
                                        .child((IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({
                                        t("时光相册", "Photo album") }) })),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({
                                        t("每一张照片都是一次美好的记忆",
                                        "Every photo holds a memory") }) })),)) })
                                        .class("travel-banner")),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::div()
                                        .child(#[allow(unused_braces)] { {
                                        #[allow(unreachable_code)] #[allow(unused_mut)]
                                        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                        component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                        & Collection, { let mut props =
                                        component_props_builder(& Collection)
                                        .love(#[allow(unused_braces)] { love })
                                        .active(#[allow(unused_braces)] { Signal::derive(| | true)
                                        }).build(); props }) } }).class("travel-content"))
                                    ,),
                            )
                            .class("travel-page")
                    })
                })
                .build();
                props
            },
        )
    })
}
#[component]
fn Collection(love: bool, active: Signal<bool>) -> impl IntoView {
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
        leptos::html::div()
            .child((
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child((
                            IntoMaybeErased::into_maybe_erased(
                                leptos::html::button()
                                    .child((
                                        IntoMaybeErased::into_maybe_erased(
                                            IntoRender::into_render({
                                                if love {
                                                    t("全部", "All")
                                                } else {
                                                    t("全部照片", "All photos")
                                                }
                                            }),
                                        ),
                                        IntoMaybeErased::into_maybe_erased(" "),
                                        IntoMaybeErased::into_maybe_erased(
                                            leptos::html::small().child(
                                                #[allow(unused_braces)]
                                                {
                                                    IntoRender::into_render({
                                                        move || {
                                                            all_total
                                                                .get()
                                                                .map(|n| n.to_string())
                                                                .unwrap_or_default()
                                                        }
                                                    })
                                                },
                                            ),
                                        ),
                                    ))
                                    .class(move || {
                                        if selected.get().is_empty() {
                                            "active"
                                        } else {
                                            ""
                                        }
                                    })
                                    .r#type("button")
                                    .aria_pressed(
                                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                            move || selected.get().is_empty().to_string(),
                                        ),
                                    )
                                    .on(leptos::ev::click, move |_| selected.set(String::new())),
                            ),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                move || {
                                    list(classes.value.get())
                                        .into_iter()
                                        .map(|c| {
                                            let title = text(&c, "classify");
                                            let checked = title.clone();
                                            let click = title.clone();
                                            View::new(
    leptos::html::button()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    title.clone() })),
                    IntoMaybeErased::into_maybe_erased(" "),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ num(& c, "count") })
                    }))
                ,),
        )
        .class(
            move || {
                if selected.get() == checked { "active" } else { "" }
            },
        )
        .r#type("button")
        .aria_pressed(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value({
                let checked = title.clone();
                move || (selected.get() == checked).to_string()
            }),
        )
        .on(
            leptos::ev::click,
            move |_| selected.set(click.clone()),
        ),
)
                                        })
                                        .collect_view()
                                }
                            })),
                        ))
                        .class(if love {
                            "original-tag-panel love-photo-filters"
                        } else {
                            "original-tag-panel"
                        })
                        .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("相册分类", "Photo categories"),
                        )),
                ),
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
                                        classes.error
                                    },
                                )
                                .public(
                                    #[allow(unused_braces)]
                                    {
                                        true
                                    },
                                )
                                .fallback(
                                    #[allow(unused_braces)]
                                    {
                                        t("相册分类加载失败", "Unable to load photo categories")
                                    },
                                )
                                .retry(
                                    #[allow(unused_braces)]
                                    {
                                        Some(Callback::new(move |()| classes.reload()))
                                    },
                                )
                                .build();
                            props
                        },
                    )
                }),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child((
                            IntoMaybeErased::into_maybe_erased(leptos::html::div().child((
                                IntoMaybeErased::into_maybe_erased(leptos::html::h2().child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({
                                            move || {
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
                                            }
                                        })
                                    },
                                )),
                                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                    love.then(|| {
                                        View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t(
                        "收藏一起看过的风景。",
                        "Keep the places and moments we have shared.",
                    )
                })
            },
        ),
)
                                    })
                                })),
                            ))),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                move || {
                                    love.then(|| {
                                        total.get().map(|n| {
                                            View::new(leptos::html::span().child(
                                                #[allow(unused_braces)]
                                                {
                                                    IntoRender::into_render({
                                                        format!("{n} {}", t("张照片", "photos"))
                                                    })
                                                },
                                            ))
                                        })
                                    })
                                }
                            })),
                        ))
                        .class("love-section-heading"),
                ),
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
                                .public(
                                    #[allow(unused_braces)]
                                    {
                                        true
                                    },
                                )
                                .fallback(
                                    #[allow(unused_braces)]
                                    {
                                        t("相册加载失败", "Unable to load photos")
                                    },
                                )
                                .retry(
                                    #[allow(unused_braces)]
                                    {
                                        Some(Callback::new(move |()| {
                                            page.set(loaded.get_untracked() + 1);
                                            data.reload();
                                        }))
                                    },
                                )
                                .build();
                            props
                        },
                    )
                }),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move || {
                        (data.busy.get() && items.get().is_empty()).then(|| {
                            View::new(
                                leptos::html::p()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                t("正在加载照片…", "Loading photos…")
                                            })
                                        },
                                    )
                                    .class("public-loading")
                                    .role("status"),
                            )
                        })
                    }
                })),
                IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)]
                    #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Show,
                        {
                            let mut props = component_props_builder(&Show)
                                .when(
                                    #[allow(unused_braces)]
                                    {
                                        move || {
                                            !items.get().is_empty()
                                                || (!data.busy.get() && data.error.get().is_none())
                                        }
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
                                            &PhotoGrid,
                                            {
                                                let mut props = component_props_builder(&PhotoGrid)
                                                    .items(
                                                        #[allow(unused_braces)]
                                                        {
                                                            Signal::derive(move || items.get())
                                                        },
                                                    )
                                                    .lightbox(
                                                        #[allow(unused_braces)]
                                                        {
                                                            lightbox
                                                        },
                                                    )
                                                    .build();
                                                props
                                            },
                                        )
                                    })
                                })
                                .build();
                            props
                        },
                    )
                }),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move || {
                        total.get().map(|total| {
                            if (items.get().len() as i64) < total {
                                View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    if love {
                        t("加载更多照片", "Load more photos")
                    } else {
                        t("下一页", "Next page")
                    }
                })
            },
        )
        .class("travel-more")
        .r#type("button")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                move || data.busy.get(),
            ),
        )
        .on(
            leptos::ev::click,
            move |_| {
                page.set(loaded.get_untracked() + 1);
                data.reload();
            },
        ),
) .into_any()
                            } else if !love {
                                View::new(
                                    leptos::html::p()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    t("~~到底啦~~", "~~You have reached the end~~")
                                                })
                                            },
                                        )
                                        .class("travel-end"),
                                )
                                .into_any()
                            } else {
                                ().into_any()
                            }
                        })
                    }
                })),
                IntoMaybeErased::into_maybe_erased({
                    #[allow(unreachable_code)]
                    #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Lightbox,
                        {
                            let mut props = component_props_builder(&Lightbox)
                                .src(
                                    #[allow(unused_braces)]
                                    {
                                        lightbox
                                    },
                                )
                                .build();
                            props
                        },
                    )
                }),
            ))
            .class(if love {
                "love-photo-collection"
            } else {
                "travel-photo-collection"
            }),
    )
}
#[component]
fn Letter() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/letter".into()));
    let open = RwSignal::new(false);
    let front = NodeRef::<leptos::html::Button>::new();
    let close = NodeRef::<leptos::html::Button>::new();
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                    .plain(
                        #[allow(unused_braces)]
                        {
                            true
                        },
                    )
                    .title(
                        #[allow(unused_braces)]
                        {
                            Signal::derive(|| t("信笺", "Letter"))
                        },
                    )
                    .children({
                        ::leptos::children::ToChildren::to_children(move || {
                            leptos::html::div()
                                .child((
                                    IntoMaybeErased::into_maybe_erased(leptos::html::h1().child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({ t("信笺", "Letter") })
                                        },
                                    )),
                                    IntoMaybeErased::into_maybe_erased(
                                        leptos::html::div()
                                            .child(
                                                #[allow(unused_braces)]
                                                {
                                                    "✿　❀　✿　❀　✿"
                                                },
                                            )
                                            .class("letter-sakura")
                                            .aria_hidden("true"),
                                    ),
                                    IntoMaybeErased::into_maybe_erased({
                                        #[allow(unreachable_code)]
                                        #[allow(unused_mut)]
                                        #[allow(
                                            clippy::let_and_return,
                                            clippy::unit_arg,
                                            clippy::let_unit_value
                                        )]
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
                                                    .public(
                                                        #[allow(unused_braces)]
                                                        {
                                                            true
                                                        },
                                                    )
                                                    .fallback(
                                                        #[allow(unused_braces)]
                                                        {
                                                            t(
                                                                "信笺加载失败",
                                                                "Unable to load the letter",
                                                            )
                                                        },
                                                    )
                                                    .retry(
                                                        #[allow(unused_braces)]
                                                        {
                                                            Some(Callback::new(move |()| {
                                                                data.reload()
                                                            }))
                                                        },
                                                    )
                                                    .build();
                                                props
                                            },
                                        )
                                    }),
                                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                        move || {
                                            data.value.get().map(|letter| {
                                                if letter.is_null() {
                                                    View::new(
                                                        leptos::html::section()
                                                            .child(
                                                                #[allow(unused_braces)]
                                                                {
                                                                    leptos::html::h2()
                    .child(
                        #[allow(unused_braces)]
                        {
                            IntoRender::into_render({
                                t("暂未公开信笺", "No letter is published yet")
                            })
                        },
                    )
                                                                },
                                                            )
                                                            .class("public-empty-state"),
                                                    )
                                                    .into_any()
                                                } else {
                                                    View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ format!("{}{}",
                    t("致 ", "To "), text(& letter, "recipient")) }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("打开信封",
                    "Open letter") }) }).node_ref(front).r#type("button")
                    .on(leptos::ev::click, move | _ | { open
                    .set(true); gloo_timers::callback::Timeout::new(0, move | | { if
                    let Some(close) = close.get() { let _ = close.focus(); } })
                    .forget(); })),)).class("letter-envelope-front")
                    .aria_hidden(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | open.get().to_string()))
                    .inert(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | open.get()))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ format!("{}{}",
                    t("致 ", "To "), text(& letter, "recipient")) }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& letter,
                    "content") }) }).class("letter-content")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& letter,
                    "sender") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::button()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("合上信封",
                    "Close letter") }) }).node_ref(close).r#type("button")
                    .on(leptos::ev::click, move | _ | { open
                    .set(false); gloo_timers::callback::Timeout::new(0, move | | { if
                    let Some(front) = front.get() { let _ = front.focus(); } })
                    .forget(); })),)).class("letter-paper")
                    .aria_hidden(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | (! open.get()).to_string()))
                    .inert(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                    | | ! open.get())))
                ,),
        )
        .class(
            move || {
                if open.get() { "letter-envelope opened" } else { "letter-envelope" }
            },
        ),
) .into_any()
                                                }
                                            })
                                        }
                                    })),
                                ))
                                .class("letter-page")
                        })
                    })
                    .build();
                props
            },
        )
    })
}
#[component]
fn Toolbox() -> impl IntoView {
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
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                .plain(#[allow(unused_braces)] { true })
                .title(
                    #[allow(unused_braces)]
                    { Signal::derive(|| t("百宝箱", "Toolbox")) },
                )
                .children({
                    ::leptos::children::ToChildren::to_children(move || {
                        leptos::html::div()
                            .child(
                                (
                                        IntoMaybeErased::into_maybe_erased(leptos::html::section()
                                        .child((IntoMaybeErased::into_maybe_erased(leptos::html::video()
                                        .autoplay(true).muted(true).r#loop(true).playsinline(true)
                                        .preload("metadata").poster("/legacy/backgroundPicture.jpg")
                                        .src("/legacy/backgroundVideo.mp4")),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::div()
                                        .child((IntoMaybeErased::into_maybe_erased(leptos::html::small()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({ t("记录",
                                        "Articles") }) })),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({ t("百宝箱",
                                        "Toolbox") }) })),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::div()
                                        .child(#[allow(unused_braces)] {
                                        IntoRender::into_render({ [("favorites",
                                        t("收藏夹", "Favorites"),
                                        t("将本网站添加到您的收藏夹吧",
                                        "Add this site to your favorites")), ("music", t("音乐",
                                        "Music"), t("一曲肝肠断，天涯何处觅知音",
                                        "Music brings kindred spirits together")), ("friends",
                                        t("友人帐", "Friends"), t("看看朋友们的网站",
                                        "Explore our friends’ websites"))].into_iter().map(| (id,
                                        l, desc) | View::new(
    leptos::html::button()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ l }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ desc }) }))
                ,),
        )
        .class(
            move || {
                if tab.get() == id { "active" } else { "" }
            },
        )
        .aria_pressed(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                move || {
                    (tab.get() == id).to_string()
                },
            ),
        )
        .on(
            leptos::ev::click,
            move |_| browse(state, &[("tab", Some(id.into())), ("page", None)]),
        ),
)).collect_view() }) })
                                        .class("favorite-feature-links")),))
                                        .class("favorite-banner-overlay")),))
                                        .class("favorite-banner")),
                                        IntoMaybeErased::into_maybe_erased(leptos::html::div()
                                        .child((IntoMaybeErased::into_maybe_erased({
                                        #[allow(unreachable_code)] #[allow(unused_mut)]
                                        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                        component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                        & Feedback, { let mut props =
                                        component_props_builder(& Feedback)
                                        .error(#[allow(unused_braces)] { data.error })
                                        .public(#[allow(unused_braces)] { true })
                                        .fallback(#[allow(unused_braces)] { t("内容加载失败",
                                        "Unable to load links") }).retry(#[allow(unused_braces)] {
                                        Some(Callback::new(move | () | data.reload())) }).build();
                                        props }) }),
                                        IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                        move | | if tab.get() == "music" { data.value.get().map(| v
                                        | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Music,
        {
            let mut props = component_props_builder(&Music)
                .items(#[allow(unused_braces)] { rows(&v, "items") })
                .build();
            props
        },
    )
}))
                                        .into_any() } else { let items = data.value.get().map(| v |
                                        rows(& v, "items")).unwrap_or_default(); let mut grouped =
                                        std::collections::BTreeMap:: < String, Vec < Value > >
                                        ::new(); for item in items { let name = text(& item,
                                        "classify"); let name = if name.is_empty() { if tab.get() ==
                                        "friends" { t("🥇友情链接", "🥇 Friend links") }
                                        else { t("我的收藏", "My favorites") } } else { name };
                                        grouped.entry(name).or_default().push(item); } if tab.get()
                                        == "friends" { let mut order = vec![t("♥️青出于蓝",
                                        "♥️ Inspiration"), t("🥇友情链接",
                                        "🥇 Friend links")]; order.extend(grouped.keys().filter(|
                                        n | ! order.contains(n)).cloned().collect:: < Vec < _ > >
                                        ()); View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::section()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("🌸 本站信息", "🌸 Site information") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::div()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::b()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("网站名称",
                    "Site name") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ site.value.get()
                    .map(| v | v.web_name.clone().unwrap_or_default())
                    .unwrap_or("xocs".into()) }) })),))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::b()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("网址", "URL")
                    }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::a()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ web_sys::window()
                    .unwrap().location().origin().unwrap_or_default() }) })
                    .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(web_sys::window()
                    .unwrap().location().origin().unwrap_or_default()))),))),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    site.value.get().and_then(| v | safe_url(& v.avatar.clone()
                    .unwrap_or_default(), false)).map(| u | View::new(
    leptos::html::p()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::b()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("头像",
                    "Avatar") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::a()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("查看头像",
                    "View avatar") }) })
                    .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
                    .target("_blank").rel("noopener noreferrer"))
                ,),
        ),
)) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::b()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("描述",
                    "Description") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ site.value.get()
                    .map(| v | v.web_title.clone().unwrap_or_default())
                    .unwrap_or(t("有诗意地记录生活",
                    "Record life through poetry")) }) })),))),))),))
                    .class("friend-site-info")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    order.into_iter().map(| name | { let items = grouped.remove(&
                    name).unwrap_or_default(); View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &FavoriteGroup,
        {
            let mut props = component_props_builder(
                    &FavoriteGroup,
                )
                .name(#[allow(unused_braces)] { name })
                .items(#[allow(unused_braces)] { items })
                .build();
            props
        },
    )
}) }).collect_view() }))
                ,),
        )
        .class("friend-page"),
)
                                        .into_any() } else { (
        IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
        grouped.into_iter().map(| (name, items) | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &FavoriteGroup,
        {
            let mut props = component_props_builder(
                    &FavoriteGroup,
                )
                .name(#[allow(unused_braces)] { name })
                .items(#[allow(unused_braces)] { items })
                .build();
            props
        },
    )
})).collect_view() })),
        IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
        (data.value.get().is_some() && data.value.get().is_some_and(| v | rows(& v,
        "items").is_empty())).then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("还没有收藏的内容。", "No favorites yet.")
                })
            },
        )
        .class("favorite-empty"),
)) }))
    ,) .into_any() } } })),
                                        IntoMaybeErased::into_maybe_erased({
                                        #[allow(unreachable_code)] #[allow(unused_mut)]
                                        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                                        component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                                        & BrowsePager, { let mut props =
                                        component_props_builder(& BrowsePager)
                                        .data(#[allow(unused_braces)] { data })
                                        .state(#[allow(unused_braces)] { state })
                                        .size(#[allow(unused_braces)] { 24 })
                                        .label(#[allow(unused_braces)] { t("百宝箱分页",
                                        "Toolbox pages") }).build(); props }) }),))
                                        .class("favorite-groups"))
                                    ,),
                            )
                            .class("favorite-page")
                    })
                })
                .build();
                props
            },
        )
    })
}
#[component]
fn FavoriteGroup(name: String, items: Vec<Value>) -> impl IntoView {
    let empty = items.is_empty();
    View::new(
        leptos::html::section()
            .child((
                IntoMaybeErased::into_maybe_erased(leptos::html::h2().child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({ name })
                    },
                )),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    items
                                        .into_iter()
                                        .map(|item| {
                                            let href = safe_url(&text(&item, "url"), false);
                                            View::new(
    leptos::html::a()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    if let Some(u) = safe_url(& text(& item, "cover"), false) { View::new(
    leptos::html::img()
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt("")
        .loading("lazy"),
) .into_any() } else { View::new(
    leptos::html::span().class("favorite-link-placeholder"),
)
                    .into_any() } })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ let n = text(& item,
                    "title"); if n.is_empty() { t("未命名", "Untitled") } else { n
                    } }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ text(& item,
                    "introduction") }) })),)))
                ,),
        )
        .class("favorite-link-card")
        .href(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(href.clone()),
        )
        .target(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                href.is_some().then_some("_blank"),
            ),
        )
        .rel("noopener noreferrer"),
)
                                        })
                                        .collect_view()
                                })
                            },
                        )
                        .class("favorite-link-grid"),
                ),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    empty.then(|| {
                        View::new(
                            leptos::html::p()
                                .child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({
                                            t("这里还没有友链。", "No friend links yet.")
                                        })
                                    },
                                )
                                .class("favorite-group-empty"),
                        )
                    })
                })),
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
        leptos::html::div()
            .child((
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child((
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                items.get_value().is_empty().then(|| {
                                    View::new(leptos::html::h2().child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                t("选择一首曲目", "Select a track")
                                            })
                                        },
                                    ))
                                })
                            })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                move || {
                                    items.get_value().get(selected.get()).cloned().map(|item| {
                                        View::new(
    (
            IntoMaybeErased::into_maybe_erased(leptos::html::h2()
            .child(#[allow(unused_braces)] {
            IntoRender::into_render({ text(& item, "title") })
            })),
            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
            safe_url(& text(& item, "cover"), false).map(| u | View::new(
    leptos::html::img()
        .class("music-cover")
        .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(u))
        .alt(""),
)) })),
            IntoMaybeErased::into_maybe_erased(leptos::html::audio()
            .controls(true)
            .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(safe_url(&
            text(& item, "url"), false))).on(leptos::ev::ended,
            move | _ | selected.update(| i | * i = (* i + 1).min(items.get_value()
            .len().saturating_sub(1)))))
        ,),
)
                                    })
                                }
                            })),
                        ))
                        .class("wall-card"),
                ),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::div()
                        .child((
                            IntoMaybeErased::into_maybe_erased(leptos::html::h2().child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({ t("播放列表", "Playlist") })
                                },
                            )),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                items
                                    .get_value()
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, item)| {
                                        View::new(
                                            leptos::html::button()
                                                .child((
                                                    IntoMaybeErased::into_maybe_erased(
                                                        IntoRender::into_render({
                                                            text(&item, "title")
                                                        }),
                                                    ),
                                                    IntoMaybeErased::into_maybe_erased(
                                                        leptos::html::small().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    text(&item, "classify")
                                                                })
                                                            },
                                                        ),
                                                    ),
                                                ))
                                                .class(move || {
                                                    if selected.get() == i {
                                                        "selected"
                                                    } else {
                                                        ""
                                                    }
                                                })
                                                .on(leptos::ev::click, move |_| selected.set(i)),
                                        )
                                    })
                                    .collect_view()
                            })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                                items.get_value().is_empty().then(|| {
                                    View::new(leptos::html::p().child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                t("暂无曲目。", "No tracks yet.")
                                            })
                                        },
                                    ))
                                })
                            })),
                        ))
                        .class("wall-card"),
                ),
            ))
            .class("music-layout"),
    )
}
#[component]
fn Avatar(name: String, src: String) -> impl IntoView {
    View::new(
        leptos::html::span()
            .child(
                #[allow(unused_braces)]
                {
                    IntoRender::into_render({
                        if let Some(u) = safe_url(&src, true) {
                            View::new(
                                leptos::html::img()
                                    .src(
                                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                            u,
                                        ),
                                    )
                                    .alt(""),
                            )
                            .into_any()
                        } else {
                            View::new(
                                leptos::html::span()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                name.chars().next().unwrap_or(' ')
                                            })
                                        },
                                    )
                                    .aria_hidden("true"),
                            )
                            .into_any()
                        }
                    })
                },
            )
            .class("love-avatar"),
    )
}
#[component]
fn Love() -> impl IntoView {
    let family = load(Signal::derive(|| "/api/v1/family".into()));
    let current = Signal::derive(move || {
        list(family.value.get())
            .into_iter()
            .next()
            .unwrap_or(Value::Null)
    });
    let tab = RwSignal::new("photos".to_string());
    let now = RwSignal::new(js_sys::Date::now() as i64);
    let interval = SendWrapper::new(gloo_timers::callback::Interval::new(1000, move || {
        now.set(js_sys::Date::now() as i64)
    }));
    on_cleanup(move || drop(interval));
    let content = NodeRef::<leptos::html::Div>::new();
    let his = Signal::derive(move || {
        let n = text(&current.get(), "man_name");
        if n.is_empty() { t("他", "Him") } else { n }
    });
    let her = Signal::derive(move || {
        let n = text(&current.get(), "woman_name");
        if n.is_empty() { t("她", "Her") } else { n }
    });
    let elapsed = Signal::derive(move || {
        crate::dates::parse(&text(&current.get(), "timing")).and_then(|s| {
            chrono::DateTime::from_timestamp_millis(now.get())
                .and_then(|e| crate::dates::together(s, e))
        })
    });
    let notes_page = RwSignal::new(1u64);
    let notes = load(Signal::derive(move || {
        if tab.get() == "notes" {
            format!("/api/v1/notes/page?page={}&size=10", notes_page.get())
        } else {
            String::new()
        }
    }));
    let lightbox = RwSignal::new(None::<String>);
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &Chrome,
            {
                let mut props = component_props_builder(&Chrome)
                .plain(#[allow(unused_braces)] { true })
                .title(
                    #[allow(unused_braces)]
                    { Signal::derive(|| t("恋爱笔记", "Love journal")) },
                )
                .children({
                    ::leptos::children::ToChildren::to_children(move || (
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Petals, { let mut props =
                            component_props_builder(& Petals)
                            .build(); props }) }),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::section()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child(#[allow(unused_braces)] {
                            leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Avatar,
        {
            let mut props = component_props_builder(&Avatar)
                .name(#[allow(unused_braces)] { his.get() })
                .src(#[allow(unused_braces)] { text(&current.get(), "man_cover") })
                .build();
            props
        },
    )
}) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | his.get() }) })),))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::img()
                            .class("love-heart").src("/legacy/love-heart.svg").alt("")
                            .aria_hidden("true")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Avatar,
        {
            let mut props = component_props_builder(&Avatar)
                .name(#[allow(unused_braces)] { her.get() })
                .src(#[allow(unused_braces)] { text(&current.get(), "woman_cover") })
                .build();
            props
        },
    )
}) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | her.get() }) })),))),)).class("love-pair")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::p()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | if crate ::dates::parse(& text(& current.get(),
                            "timing")).is_some() { format!("{}{}", t("故事开始于 ",
                            "Our story began on "), crate ::dates::format(& text(&
                            current.get(), "timing"))) } else {
                            t("等待第一份属于我们的回忆",
                            "A place for our first memory together") } }) })),))
                            .class("love-pair-card") }).class("love-hero-inner")),
                            IntoMaybeErased::into_maybe_erased(leptos::svg::svg()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::svg::path()
                            .attr("opacity", ".45").attr("d",
                            "M0 35C180 100 280 0 440 45S700 95 870 40S1170 0 1440 45V110H0Z")),
                            IntoMaybeErased::into_maybe_erased(leptos::svg::path()
                            .attr("opacity", ".7").attr("d",
                            "M0 55C150 15 280 95 450 60S730 10 920 65S1210 80 1440 35V110H0Z")),
                            IntoMaybeErased::into_maybe_erased(leptos::svg::path()
                            .attr("d",
                            "M0 70C180 100 340 30 530 75S850 90 1060 55S1300 80 1440 65V110H0Z")),))
                            .class("love-wave").attr("viewBox", "0 0 1440 110")
                            .attr("preserveAspectRatio", "none").attr("aria-hidden",
                            "true")),)).style(move | |
                            safe_url(& text(& current.get(), "bg_cover"), true).map(| u
                            | format!("background-image:url({})",
                            serde_json::to_string(& u).unwrap_or_default()))
                            .unwrap_or_default()).class("love-hero")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { family.error })
                            .public(#[allow(unused_braces)] { true })
                            .fallback(#[allow(unused_braces)] { t("故事加载失败",
                            "Unable to load our story") }).retry(#[allow(unused_braces)]
                            { Some(Callback::new(move | () | family.reload())) })
                            .build(); props }) }),
                            IntoMaybeErased::into_maybe_erased(leptos::html::section()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::h1()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("这是我们一起走过的", "Our journey together") })
                            })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | if let Some(e) = elapsed.get() { View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ t("第",
                    "Together for") }) }).class("love-clock-prefix")),
                    IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    [("years", e.years as i64, t("年", if e.years == 1 { "year" }
                    else { "years" })), ("months", e.months as i64, t("月", if e
                    .months == 1 { "month" } else { "months" })), ("days", e.days,
                    t("日", if e.days == 1 { "day" } else { "days" })), ("hours", e
                    .hours, t("时", if e.hours == 1 { "hour" } else { "hours" })),
                    ("minutes", e.minutes, t("分", if e.minutes == 1 { "minute" }
                    else { "minutes" })), ("seconds", e.seconds, t("秒", if e
                    .seconds == 1 { "second" } else { "seconds" }))].into_iter()
                    .map(| (key, v, l) | View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ v }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ l }) }))
                ,),
        )
        .attr(
            "data-unit",
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(key),
        ),
))
                    .collect_view() }))
                ,),
        )
        .class("love-clock")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("相伴时长", "Time spent together"),
            ),
        ),
) .into_any() } else { View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if crate
                    ::dates::parse(& text(& current.get(), "timing")).is_some() {
                    t("新的旅程即将开始", "A new journey begins soon") } else { t("从第一份回忆开始", "Start with your first memory") }
                    }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("留下相识的日期，让时间替我们记住每一个日子。",
                    "Add the day you met and keep track of each day together.") })
                    }))
                ,),
        )
        .class("love-timer-empty"),
) .into_any() } })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | crate ::dates::parse(& text(& current.get(),
                            "countdown_time")).and_then(| target |
                            chrono::DateTime::from_timestamp_millis(now.get()).map(| n |
                            (target.date_naive() - n.date_naive()).num_days())).map(|
                            days | View::new(
    leptos::html::p()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    text(& current.get(), "countdown_title") })),
                    IntoMaybeErased::into_maybe_erased("："),))),
                    IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ if days > 0 {
                    format!("{} {days} {}", t("还有", ""), t("天", "days to go"))
                    } else if days == 0 { t("就是今天", "Today is the day") }
                    else { format!("{} {} {}", t("已过去", ""),- days, t("天",
                    "days ago")) } }) }))
                ,),
        )
        .class("love-countdown"),
)) })),))
                            .class("love-timer")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::a()
                            .child(#[allow(unused_braces)] {
                            leptos::html::span()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("飞车传信", "A letter for you") }) }) })
                            .class("love-letter")
                            .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/letter")))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::details()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::summary()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("我们的故事", "Our story") }) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::p()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            move | | { let story = text(& current.get(), "family_info");
                            if story.is_empty() {
                            t("把平常的日子写下来，让照片和文字慢慢填满这个家。",
                            "Make a home for everyday moments, one photo and one story at a time.")
                            } else { story } } }) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::a()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            t("看看日常记录", "Explore our journal") })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::span()
                            .child(#[allow(unused_braces)] { "↗" })
                            .aria_hidden("true")),))
                            .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/jotting")))),))
                            .class("love-story-card")),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::nav()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({ [("notes",
                            t("点点滴滴", "Moments"), "/legacy/love-moments.jpg",
                            t("☀️ 今朝有酒今朝醉",
                            "☀️ Little moments of joy")), ("photos",
                            t("时光相册", "Photos"), "/legacy/love-photos.jpg",
                            t("📸 记录美好瞬间",
                            "📸 Keep beautiful memories")), ("wishes", t("祝福板",
                            "Wishes"), "/legacy/love-wishes.jpg",
                            t("📋 写下你们的祝福", "📋 Leave your wishes"))]
                            .into_iter().map(| (id, l, icon, desc) | View::new(
    leptos::html::button()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::img()
                    .class("love-tab-icon")
                    .src(::leptos::prelude::IntoAttributeValue::into_attribute_value(icon))
                    .alt("")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child((IntoMaybeErased::into_maybe_erased(leptos::html::strong()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ l }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::small()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({ desc }) })),)))
                ,),
        )
        .class(
            move || {
                if tab.get() == id { "active" } else { "" }
            },
        )
        .r#type("button")
        .aria_pressed(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                move || {
                    (tab.get() == id).to_string()
                },
            ),
        )
        .aria_controls("love-panel")
        .on(
            leptos::ev::click,
            move |_| {
                tab.set(id.into());
                if let Some(c) = content.get() {
                    c.scroll_into_view();
                }
            },
        ),
)).collect_view() }) }).class("love-tabs")
                            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(t("恋爱笔记内容",
                            "Love journal content")))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::section()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child(#[allow(unused_braces)] { {
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Collection, { let mut props =
                            component_props_builder(& Collection)
                            .love(#[allow(unused_braces)] { true })
                            .active(#[allow(unused_braces)] { Signal::derive(move | |
                            tab.get() == "photos") }).build(); props }) } })
                            .hidden(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                            | | tab.get() != "photos"))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(leptos::html::h2()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("点点滴滴", "Moments") }) })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::p()
                            .child(#[allow(unused_braces)] {
                            IntoRender::into_render({
                            t("记录那些平常又特别的小事。",
                            "A place for the little things that make our days.") })
                            })),))),
                            IntoMaybeErased::into_maybe_erased(leptos::html::a()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            t("浏览更多记录", "Browse more entries") })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::span()
                            .child(#[allow(unused_braces)] { "↗" })
                            .aria_hidden("true")),))
                            .href(::leptos::prelude::IntoAttributeValue::into_attribute_value(public_href("/jotting")))),))
                            .class("love-section-heading")),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Feedback, { let mut props =
                            component_props_builder(& Feedback)
                            .error(#[allow(unused_braces)] { notes.error })
                            .public(#[allow(unused_braces)] { true })
                            .retry(#[allow(unused_braces)] { Some(Callback::new(move |
                            () | notes.reload())) }).build(); props }) }),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | notes.busy.get().then(| | View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("正在加载记录…", "Loading entries…")
                })
            },
        )
        .class("public-loading")
        .role("status"),
))
                            })),
                            IntoMaybeErased::into_maybe_erased(leptos::html::div()
                            .child((IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | notes.value.get().map(| v | rows(& v, "items"))
                            .unwrap_or_default().into_iter().map(| item | View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &NoteCard,
        {
            let mut props = component_props_builder(&NoteCard)
                .item(#[allow(unused_braces)] { item })
                .lightbox(#[allow(unused_braces)] { lightbox })
                .love(#[allow(unused_braces)] { true })
                .build();
            props
        },
    )
})).collect_view()
                            })),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | notes.value.get().is_some_and(| v | rows(& v,
                            "items").is_empty()).then(| | View::new(
    leptos::html::div()
        .child(
            (
                    IntoMaybeErased::into_maybe_erased(leptos::html::span()
                    .child(#[allow(unused_braces)] { "✎" }).aria_hidden("true")),
                    IntoMaybeErased::into_maybe_erased(leptos::html::h3()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("还没有记录。", "No entries yet.") }) })),
                    IntoMaybeErased::into_maybe_erased(leptos::html::p()
                    .child(#[allow(unused_braces)] {
                    IntoRender::into_render({
                    t("写下此刻的想法，留住每一个值得纪念的日常。",
                    "Write a thought and keep a little piece of today.") }) }))
                ,),
        )
        .class("public-empty-state"),
)) })),)).class("love-moments-grid")),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Pager, { let mut props =
                            component_props_builder(& Pager)
                            .data(#[allow(unused_braces)] { notes })
                            .page(#[allow(unused_braces)] { notes_page })
                            .public(#[allow(unused_braces)] { true })
                            .label(#[allow(unused_braces)] { t("记录分页",
                            "Entry pages") }).build(); props }) }),))
                            .hidden(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                            | | tab.get() != "notes"))),
                            IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                            move | | (tab.get() == "wishes").then(| | View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                {
                    #[allow(unreachable_code)] #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Comments,
                        {
                            let mut props = component_props_builder(
                                    &Comments,
                                )
                                .kind(#[allow(unused_braces)] { "love" })
                                .title(#[allow(unused_braces)] { t("祝福板", "Wishes") })
                                .build();
                            props
                        },
                    )
                }
            },
        )
        .class("love-wishes"),
)) })),))
                            .class("love-panel").id("love-panel")
                            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(move
                            | | match tab.get().as_str() { "notes" => t("点点滴滴",
                            "Moments"), "wishes" => t("祝福板", "Wishes"), _ =>
                            t("时光相册", "Photos") }))),)).class("love-workspace")
                            .node_ref(content)),)).class("love-content")),))
                            .class("love-page")),
                            IntoMaybeErased::into_maybe_erased({
                            #[allow(unreachable_code)] #[allow(unused_mut)]
                            #[allow(clippy::let_and_return, clippy::unit_arg, clippy::let_unit_value)]
                            component_view(#[allow(clippy::needless_borrows_for_generic_args)]
                            & Lightbox, { let mut props =
                            component_props_builder(& Lightbox)
                            .src(#[allow(unused_braces)] { lightbox }).build(); props })
                            })
                        ,))
                })
                .build();
                props
            },
        )
    })
}
