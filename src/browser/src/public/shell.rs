//! Public navigation, theme state, site context and shared page chrome.
use super::appearance::{background_style, site_media_url, site_name};
use crate::common::{Data, href, pathname, safe_url, t};
use crate::models::{Article as ArticleModel, Site};
use blog_interface::ev;
use blog_interface::html;
use blog_interface::prelude::*;
use blog_interface::svg;
use blog_interface::thread_bound::SendWrapper;
use wasm_bindgen::JsCast;

pub(super) fn site_data() -> Data<Site> {
    use_context::<Data<Site>>().expect("public site context")
}

#[component]
pub(super) fn Chrome(
    #[prop(default=Signal::derive(String::new))] title: Signal<String>,
    #[prop(default = false)] home: bool,
    #[prop(default = false)] plain: bool,
    #[prop(default = true)] inherit_background: bool,
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
            .and_then(|s| s.get_item("blog-theme").ok().flatten())
            .as_deref()
            == Some("dark"),
    );
    let scrolled = RwSignal::new(false);
    let hidden = RwSignal::new(false);
    let previous = StoredValue::new(0.);
    let direction = StoredValue::new(0i32);
    let distance = StoredValue::new(0.);
    let scroll = SendWrapper::new(blog_interface::events::EventListener::new(
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
    let keys = SendWrapper::new(blog_interface::events::EventListener::new(
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
    html::div()
        .child(
            (
                    html::header()
                    .child((html::a()
                    .child(move || match site_media_url(site, |site| site.media.logo.as_deref()) {
                        Some(url) => View::new(html::img().src(url).alt(site_name(site))).into_any(),
                        None => View::new(html::span().child(site_name(site))).into_any(),
                    }).class("brand")
                    .href(href("/"))
                    .aria_label(t("blog 首页",
                    "blog home")),
                    html::nav()
                    .child((html::span()
                    .child( t("欢迎光临",
                    "Welcome")).class("public-mobile-title"),
                    nav.into_iter().map(| (icon, label, path) | View::new(
    html::a()
        .child(
            (
                    html::span()
                    .child( icon)
                    .aria_hidden("true"),
                    " ",
                    label
                ,),
        )
        .href(href(path))
        .aria_current(
            (pathname() == path).then_some("page"),
        ),
)).collect_view(),))
                    .class(move | | if menu.get() {
                    "open" } else { "" })
                    .aria_label(t("主导航",
                    "Main navigation")),
                    html::div()
                    .child((blog_interface::components::LanguageToggle(),
                    html::button()
                    .child( svg::svg()
                    .child( svg::path()
                    .attr("d",
                    move
                    | | if dark.get() {
                    "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8M12 2v2m0 16v2M2 12h2m16 0h2" }
                    else { "M20.9 13A9 9 0 1 1 11 3a7 7 0 0 0 9.9 10Z" }))
                    .attr("aria-hidden", "true").attr("width", "1em").attr("height",
                    "1em").attr("viewBox", "0 0 24 24").attr("fill", "none")
                    .attr("stroke", "currentColor").attr("stroke-width", "1.7"))
                    .class("theme-toggle").r#type("button")
                    .aria_label(move
                    | | if dark.get() { t("切换到浅色模式",
                    "Switch to light mode") } else { t("切换到深色模式",
                    "Switch to dark mode") })
                    .title(move
                    | | if dark.get() { t("切换到浅色模式",
                    "Switch to light mode") } else { t("切换到深色模式",
                    "Switch to dark mode") })
                    .aria_pressed(move
                    | | dark.get().to_string())
                    .on(ev::click, move | _ | { dark
                    .update(| v | * v = ! * v); if let Some(s) = web_sys::window()
                    .unwrap().local_storage().ok().flatten() { let _ = s
                    .set_item("blog-theme", if dark.get_untracked() { "dark" } else {
                    "light" }); } }),)).class("public-header-controls"),
                    html::button()
                    .child( move | | if menu.get() { "✕" } else { "☰" })
                    .class("public-menu-toggle").r#type("button")
                    .aria_label(move
                    | | if menu.get() { t("关闭导航", "Close navigation") } else { t("打开导航", "Open navigation") })
                    .aria_expanded(move
                    | | menu.get().to_string())
                    .on(ev::click, move | _ | menu
                    .update(| v | * v = ! * v)),))
                    .class(move | |
                    format!("public-header{}{}{}", if scrolled.get() || menu.get() ||
                    plain { " entered" } else { "" }, if plain { " plain-header" }
                    else { "" }, if hidden.get() && !menu.get() { " header-hidden" }
                    else { "" })),
                    (! plain).then(| | View::new(
    html::section()
        .child(
            (
                    html::div()
                    .child((html::h1()
                    .child( move | | if home { let value = site.value.get().map(| v | v
                    .web_title.clone().unwrap_or_default()).unwrap_or_default(); if
                    value.is_empty() { t("相信记录的力量",
                    "Believe in the power of stories") } else { value } } else {
                    title.get() }),
                    home.then(| | View::new(
    html::p()
        .child(
            (
                    t("相信记录的力量！", "Believe in the power of stories!"),
                    html::span()
                    .child( "|").class("cursor")
                ,),
        )
        .class("hero-poem"),
)),
                    move | | (! subtitle.get().is_empty()).then(| | View::new(
    html::p()
        .child(

            subtitle.get(),
        )
        .class("hero-subtitle"),
)),
                    move | | metadata.get().map(| a | View::new(
    html::p()
        .child(

            format!(
                        "👤 {}　·　📅 {}　·　🔥 {}{}{}{}{}{}", a.username
                        .clone().unwrap_or_default(), crate ::dates::format(& a
                        .create_time.clone().unwrap_or_default()), a.view_count,
                        t(" 热度 · 💬 ", " views · 💬 "), a.comment_count,
                        t(" 评论 · ❤️ ", " comments · ❤️ "), a
                        .like_count, t(" 点赞", " likes")
                    ),
        )
        .class("hero-article-meta"),
)),))
                    .class(move | | if metadata
                    .get().is_some() { "hero-article-info" } else { "hero-center"
                    }),
                    home.then(| | View::new(
    (
            html::a()
            .child( "⌄").class("hero-down")
            .href("#public-content")
            .aria_label(t("浏览文章",
            "Browse articles")),
            html::div()
            .class("hero-wave hero-wave-back").aria_hidden("true"),
            html::div()
            .class("hero-wave hero-wave-front").aria_hidden("true")
        ,),
))
                ,),
        )
        .style(
            move || {
                let requested = cover.get();
                let info = site.value.get();
                let image = if requested.is_empty() && inherit_background {
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
                background_style(safe_url(image.trim(), false))
            },
        )
        .class(
            move || format!(
                    "hero{}", if home { " home-hero" } else if metadata.get()
                    .is_some() { " article-hero" } else { " compact-hero" }
                ),
        ),
)),
                    html::main()
                    .child( children())
                    .class(if home { "public-main" } else { "public-main extra-main"
                    }).id("public-content"),
                    move | | scrolled.get().then(| | View::new(
    html::div()
        .child(

            html::button()
                    .child(

                        html::span()
                                .child( "↑")
                                .aria_hidden("true"),
                    )
                    .class("back-top")
                    .r#type("button")
                    .aria_label(
                        t("返回顶部", "Back to top"),
                    )
                    .on(
                        ev::click,
                        move |_| {
                            let opts = web_sys::ScrollToOptions::new();
                            opts.set_top(0.);
                            opts.set_behavior(web_sys::ScrollBehavior::Smooth);
                            web_sys::window()
                                .unwrap()
                                .scroll_to_with_scroll_to_options(&opts);
                        },
                    ),
        )
        .class("public-tools"),
))
                ,),
        )
        .class(
            move || format!(
                    "public-site original-public page-{}{}", { let p = pathname()
                    .chars().filter(| c | c.is_ascii_alphanumeric() ||* c =='_' ||* c
                    =='-').collect::< String > (); if p.is_empty() { "home"
                    .to_string() } else { p } }, if dark.get() { " theme-dark" } else { "" }
                ),
        ),
)
}

pub(super) fn notices(v: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(v).unwrap_or_else(|_| {
        v.lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    })
}
