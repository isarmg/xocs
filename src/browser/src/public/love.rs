//! Couple journal, shared photo and note panels, and letter interactions.
use super::appearance::{Avatar, background_style, site_media_url};
use super::journal::NoteCard;
use super::photos::Collection;
use super::shell::{Chrome, site_data};
use crate::comments::Comments;
use crate::common::{Feedback, Lightbox, Pager, Petals, href, list, load, rows, safe_url, t, text};
use blog_interface::children::ToChildren;
use blog_interface::component::component_props_builder;
use blog_interface::ev;
use blog_interface::html;
use blog_interface::prelude::*;
use blog_interface::svg;
use blog_interface::thread_bound::SendWrapper;
use serde_json::Value;

fn cover_art(source: Option<String>, placeholder: &'static str) -> AnyView {
    match source {
        Some(source) => {
            View::new(html::img().class("love-tab-icon").src(source).alt("")).into_any()
        }
        None => View::new(
            html::span()
                .class("love-tab-icon love-tab-fallback")
                .aria_hidden("true")
                .child(placeholder),
        )
        .into_any(),
    }
}

#[component]
pub(super) fn Letter() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/letter".into()));
    let open = RwSignal::new(false);
    let front = NodeRef::<html::Button>::new();
    let close = NodeRef::<html::Button>::new();
    View::new(Chrome(
        component_props_builder(&Chrome)
            .plain(true)
            .title(Signal::derive(|| t("信笺", "Letter")))
            .children(ToChildren::to_children(move || {
                html::div()
                    .child((
                        html::h1().child(t("信笺", "Letter")),
                        html::div()
                            .child("✿　❀　✿　❀　✿")
                            .class("letter-sakura")
                            .aria_hidden("true"),
                        Feedback(
                            component_props_builder(&Feedback)
                                .error(data.error)
                                .public(true)
                                .fallback(t("信笺加载失败", "Unable to load the letter"))
                                .retry(Some(Callback::new(move |()| data.reload())))
                                .build(),
                        ),
                        move || {
                            data.value.get().map(|letter| {
                                if letter.is_null() {
                                    View::new(
                                        html::section()
                                            .child(html::h2().child(t(
                                                "暂未公开信笺",
                                                "No letter is published yet",
                                            )))
                                            .class("public-empty-state"),
                                    )
                                    .into_any()
                                } else {
                                    View::new(
    html::div()
        .child(
            (
                    html::div()
                    .child((html::span()
                    .child( format!("{}{}",
                    t("致 ", "To "), text(& letter, "recipient"))),
                    html::button()
                    .child( t("打开信封",
                    "Open letter")).node_ref(front).r#type("button")
                    .on(ev::click, move | _ | { open
                    .set(true); blog_interface::timers::callback::Timeout::new(0, move || { if
                    let Some(close) = close.get() { let _ = close.focus(); } })
                    .forget(); }),)).class("letter-envelope-front")
                    .aria_hidden(move
                    | | open.get().to_string())
                    .inert(move
                    | | open.get()),
                    html::div()
                    .child((html::h2()
                    .child( format!("{}{}",
                    t("致 ", "To "), text(& letter, "recipient"))),
                    html::p()
                    .child( text(& letter,
                    "content")).class("letter-content"),
                    html::strong()
                    .child( text(& letter,
                    "sender")),
                    html::button()
                    .child( t("合上信封",
                    "Close letter")).node_ref(close).r#type("button")
                    .on(ev::click, move | _ | { open
                    .set(false); blog_interface::timers::callback::Timeout::new(0, move || { if
                    let Some(front) = front.get() { let _ = front.focus(); } })
                    .forget(); }),)).class("letter-paper")
                    .aria_hidden(move
                    | | (! open.get()).to_string())
                    .inert(move
                    | | ! open.get())
                ,),
        )
        .class(
            move || if open.get() { "letter-envelope opened" } else { "letter-envelope" },
        ),
) .into_any()
                                }
                            })
                        },
                    ))
                    .class("letter-page")
            }))
            .build(),
    ))
}

#[component]
pub(super) fn Love() -> impl IntoView {
    let site = site_data();
    let family = load(Signal::derive(|| "/api/v1/family".into()));
    let current = Signal::derive(move || {
        list(family.value.get())
            .into_iter()
            .next()
            .unwrap_or(Value::Null)
    });
    let tab = RwSignal::new("photos".to_string());
    let now = RwSignal::new(js_sys::Date::now() as i64);
    let interval = SendWrapper::new(blog_interface::timers::callback::Interval::new(
        1000,
        move || now.set(js_sys::Date::now() as i64),
    ));
    on_cleanup(move || drop(interval));
    let content = NodeRef::<html::Div>::new();
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
    View::new(Chrome(component_props_builder(&Chrome)
                .plain( true)
                .title(

                    Signal::derive(|| t("恋爱笔记", "Love journal")),
                )
                .children(ToChildren::to_children(move || (
                            Petals(),
                            html::div()
                            .child((html::section()
                            .child((html::div()
                            .child( html::div()
                            .child((html::div()
                            .child((html::div()
                            .child((move | | View::new(Avatar(component_props_builder(&Avatar)
                .name( his.get())
                .src( text(&current.get(), "man_cover"))
                .build())),
                            html::strong()
                            .child( move | | his.get()),)),
                            svg::svg()
                            .class("love-heart").attr("viewBox", "0 0 24 24").attr("aria-hidden", "true")
                            .child(svg::path().attr("d", "M12 21 3.6 12.6A5.5 5.5 0 0 1 11.4 4.8L12 5.4l.6-.6a5.5 5.5 0 0 1 7.8 7.8Z")),
                            html::div()
                            .child((move | | View::new(Avatar(component_props_builder(&Avatar)
                .name( her.get())
                .src( text(&current.get(), "woman_cover"))
                .build())),
                            html::strong()
                            .child( move | | her.get()),)),)).class("love-pair"),
                            html::p()
                            .child( move | | if crate ::dates::parse(& text(& current.get(),
                            "timing")).is_some() { format!("{}{}", t("故事开始于 ",
                            "Our story began on "), crate ::dates::format(& text(&
                            current.get(), "timing"))) } else {
                            t("等待第一份属于我们的回忆",
                            "A place for our first memory together") }),))
                            .class("love-pair-card")).class("love-hero-inner"),
                            svg::svg()
                            .child((svg::path()
                            .attr("opacity", ".45").attr("d",
                            "M0 35C180 100 280 0 440 45S700 95 870 40S1170 0 1440 45V110H0Z"),
                            svg::path()
                            .attr("opacity", ".7").attr("d",
                            "M0 55C150 15 280 95 450 60S730 10 920 65S1210 80 1440 35V110H0Z"),
                            svg::path()
                            .attr("d",
                            "M0 70C180 100 340 30 530 75S850 90 1060 55S1300 80 1440 65V110H0Z"),))
                            .class("love-wave").attr("viewBox", "0 0 1440 110")
                            .attr("preserveAspectRatio", "none").attr("aria-hidden",
                            "true"),)).style(move | |
                            background_style(safe_url(text(&current.get(), "bg_cover").trim(), true)))
                            .class("love-hero"),
                            html::div()
                            .child((Feedback(component_props_builder(& Feedback)
                            .error( family.error)
                            .public( true)
                            .fallback( t("故事加载失败",
                            "Unable to load our story")).retry(
                            Some(Callback::new(move | () | family.reload())))
                            .build()),
                            html::section()
                            .child((html::h1()
                            .child( t("这是我们一起走过的", "Our journey together")),
                            move | | if let Some(e) = elapsed.get() { View::new(
    html::div()
        .child(
            (
                    html::span()
                    .child( t("第",
                    "Together for")).class("love-clock-prefix"),
                    [("years", e.years as i64, t("年", if e.years == 1 { "year" }
                    else { "years" })), ("months", e.months as i64, t("月", if e
                    .months == 1 { "month" } else { "months" })), ("days", e.days,
                    t("日", if e.days == 1 { "day" } else { "days" })), ("hours", e
                    .hours, t("时", if e.hours == 1 { "hour" } else { "hours" })),
                    ("minutes", e.minutes, t("分", if e.minutes == 1 { "minute" }
                    else { "minutes" })), ("seconds", e.seconds, t("秒", if e
                    .seconds == 1 { "second" } else { "seconds" }))].into_iter()
                    .map(| (key, v, l) | View::new(
    html::div()
        .child(
            (
                    html::strong()
                    .child( v),
                    html::span()
                    .child( l)
                ,),
        )
        .attr(
            "data-unit",
            key,
        ),
))
                    .collect_view()
                ,),
        )
        .class("love-clock")
        .aria_label(
            t("相伴时长", "Time spent together"),
        ),
) .into_any() } else { View::new(
    html::div()
        .child(
            (
                    html::strong()
                    .child( if crate
                    ::dates::parse(& text(& current.get(), "timing")).is_some() {
                    t("新的旅程即将开始", "A new journey begins soon") } else { t("从第一份回忆开始", "Start with your first memory") }),
                    html::p()
                    .child( t("留下相识的日期，让时间替我们记住每一个日子。",
                    "Add the day you met and keep track of each day together."))
                ,),
        )
        .class("love-timer-empty"),
) .into_any() },
                            move | | crate ::dates::parse(& text(& current.get(),
                            "countdown_time")).and_then(| target |
                            chrono::DateTime::from_timestamp_millis(now.get()).map(| n |
                            (target.date_naive() - n.date_naive()).num_days())).map(|
                            days | View::new(
    html::p()
        .child(
            (
                    html::span()
                    .child((text(& current.get(), "countdown_title"),
                    "：",)),
                    html::strong()
                    .child( if days > 0 {
                    format!("{} {days} {}", t("还有", ""), t("天", "days to go"))
                    } else if days == 0 { t("就是今天", "Today is the day") }
                    else { format!("{} {} {}", t("已过去", ""),- days, t("天",
                    "days ago")) })
                ,),
        )
        .class("love-countdown"),
)),))
                            .class("love-timer"),
                            html::a()
                            .child( html::span()
                            .child( t("飞车传信", "A letter for you")))
                            .class("love-letter")
                            .style(move || background_style(site_media_url(site, |site| site.media.love_letter_background.as_deref())))
                            .href(href("/letter")),
                            html::details()
                            .child((html::summary()
                            .child( t("我们的故事", "Our story")),
                            html::p()
                            .child( move | | { let story = text(& current.get(), "family_info");
                            if story.is_empty() {
                            t("把平常的日子写下来，让照片和文字慢慢填满这个家。",
                            "Make a home for everyday moments, one photo and one story at a time.")
                            } else { story } }),
                            html::a()
                            .child((t("看看日常记录", "Explore our journal"),
                            html::span()
                            .child( "↗")
                            .aria_hidden("true"),))
                            .href(href("/jotting")),))
                            .class("love-story-card"),
                            html::div()
                            .child((html::nav()
                            .child( move || [("notes",
                            t("点点滴滴", "Moments"), site_media_url(site, |site| site.media.love_moments_cover.as_deref()), "☀",
                            t("☀️ 今朝有酒今朝醉",
                            "☀️ Little moments of joy")), ("photos",
                            t("时光相册", "Photos"), site_media_url(site, |site| site.media.love_photos_cover.as_deref()), "▧",
                            t("📸 记录美好瞬间",
                            "📸 Keep beautiful memories")), ("wishes", t("祝福板",
                            "Wishes"), site_media_url(site, |site| site.media.love_wishes_cover.as_deref()), "♡",
                            t("📋 写下你们的祝福", "📋 Leave your wishes"))]
                            .into_iter().map(| (id, l, icon, placeholder, desc) | View::new(
    html::button()
        .child(
            (
                    cover_art(icon, placeholder),
                    html::span()
                    .child((html::strong()
                    .child( l),
                    html::small()
                    .child( desc),))
                ,),
        )
        .class(
            move || if tab.get() == id { "active" } else { "" },
        )
        .r#type("button")
        .aria_pressed(
            move || (tab.get() == id).to_string(),
        )
        .aria_controls("love-panel")
        .on(
            ev::click,
            move |_| {
                tab.set(id.into());
                if let Some(c) = content.get() {
                    c.scroll_into_view();
                }
            },
        ),
)).collect_view()).class("love-tabs")
                            .aria_label(t("恋爱笔记内容",
                            "Love journal content")),
                            html::section()
                            .child((html::div()
                            .child( Collection(component_props_builder(& Collection)
                            .love( true)
                            .active( Signal::derive(move | |
                            tab.get() == "photos")).build()))
                            .hidden(move
                            | | tab.get() != "photos"),
                            html::div()
                            .child((html::div()
                            .child((html::div()
                            .child((html::h2()
                            .child( t("点点滴滴", "Moments")),
                            html::p()
                            .child( t("记录那些平常又特别的小事。",
                            "A place for the little things that make our days.")),)),
                            html::a()
                            .child((t("浏览更多记录", "Browse more entries"),
                            html::span()
                            .child( "↗")
                            .aria_hidden("true"),))
                            .href(href("/jotting")),))
                            .class("love-section-heading"),
                            Feedback(component_props_builder(& Feedback)
                            .error( notes.error)
                            .public( true)
                            .retry( Some(Callback::new(move |
                            () | notes.reload()))).build()),
                            move | | notes.busy.get().then(| | View::new(
    html::p()
        .child(

            t("正在加载记录…", "Loading entries…"),
        )
        .class("public-loading")
        .role("status"),
)),
                            html::div()
                            .child((move | | notes.value.get().map(| v | rows(& v, "items"))
                            .unwrap_or_default().into_iter().map(| item | View::new(NoteCard(component_props_builder(&NoteCard)
                .item( item)
                .lightbox( lightbox)
                .love( true)
                .build()))).collect_view(),
                            move | | notes.value.get().is_some_and(| v | rows(& v,
                            "items").is_empty()).then(| | View::new(
    html::div()
        .child(
            (
                    html::span()
                    .child( "✎").aria_hidden("true"),
                    html::h3()
                    .child( t("还没有记录。", "No entries yet.")),
                    html::p()
                    .child( t("写下此刻的想法，留住每一个值得纪念的日常。",
                    "Write a thought and keep a little piece of today."))
                ,),
        )
        .class("public-empty-state"),
)),)).class("love-moments-grid"),
                            Pager(component_props_builder(& Pager)
                            .data( notes)
                            .page( notes_page)
                            .public( true)
                            .label( t("记录分页",
                            "Entry pages")).build()),))
                            .hidden(move
                            | | tab.get() != "notes"),
                            move | | (tab.get() == "wishes").then(| | View::new(
    html::div()
        .child(

            Comments(component_props_builder(
                                    &Comments,
                                )
                                .kind( "love")
                                .title( t("祝福板", "Wishes"))
                                .build()),
        )
        .class("love-wishes"),
)),))
                            .class("love-panel").id("love-panel")
                            .aria_label(move
                            | | match tab.get().as_str() { "notes" => t("点点滴滴",
                            "Moments"), "wishes" => t("祝福板", "Wishes"), _ =>
                            t("时光相册", "Photos") }),)).class("love-workspace")
                            .node_ref(content),)).class("love-content"),))
                            .class("love-page"),
                            Lightbox(component_props_builder(& Lightbox)
                            .src( lightbox).build())
                        ,)))
                .build()))
}
