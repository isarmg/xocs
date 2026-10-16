use crate::common::Action;
use crate::{comments::Comments, common::*};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde_json::{Value, json};
use wasm_bindgen::JsCast;
#[component]
pub fn Public() -> impl IntoView {
    let site = load(Signal::derive(|| "/api/v1/site".into()));
    provide_context(site);
    match pathname().as_str(){"/"=>view!{<Home/>}.into_any(),"/sort"=>view!{<Browse search=false/>}.into_any(),"/search"=>view!{<Browse search=true/>}.into_any(),"/jotting"=>view!{<Journal/>}.into_any(),"/menory"=>view!{<Journey/>}.into_any(),"/message"=>view!{<Message/>}.into_any(),"/travel"=>view!{<Photos love=false/>}.into_any(),"/favorite"=>view!{<Toolbox/>}.into_any(),"/love"=>view!{<Love/>}.into_any(),"/letter"=>view!{<Letter/>}.into_any(),path if path.starts_with("/article/")=>view!{<Article id={path[9..].parse::<i64>().unwrap_or(0)}/>}.into_any(),_=>view!{<Chrome title=Signal::derive(||t("页面不存在","Page not found"))><section class="public-empty-state"><h2>{t("页面不存在","Page not found")}</h2><a href=public_href("/")>{t("返回首页","Back to home")}</a></section></Chrome>}.into_any()}
}
fn public_href(path: &str) -> String {
    path.into()
}
fn article_excerpt(value: &str, limit: usize) -> String {
    let mut plain = value.to_string();
    for (pattern, replacement) in [
        (r"!\[[^\]]*\]\([^)]*\)", " "),
        (r"\[([^\]]+)\]\([^)]*\)", "$1"),
        (r"<[^>]+>", " "),
        (r"[#>*_`~|]", " "),
    ] {
        plain = regex::Regex::new(pattern)
            .expect("fixed excerpt pattern")
            .replace_all(&plain, replacement)
            .into_owned();
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
fn site_data() -> Data {
    use_context::<Data>().expect("public site context")
}
#[component]
fn Chrome(
    #[prop(default=Signal::derive(String::new))] title: Signal<String>,
    #[prop(default = false)] home: bool,
    #[prop(default = false)] plain: bool,
    #[prop(default=Signal::derive(String::new))] cover: Signal<String>,
    #[prop(default=Signal::derive(String::new))] subtitle: Signal<String>,
    #[prop(default=Signal::derive(||None))] metadata: Signal<Option<Value>>,
    children: ChildrenFn,
) -> impl IntoView {
    let site = site_data();
    let random_cover = Memo::new(move |_| {
        let covers = site
            .value
            .get()
            .map(|v| notices(&text(&v, "random_cover")))
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
    view! {<div class=move||format!("public-site original-public page-{}{}",{let p=pathname().chars().filter(|c|c.is_ascii_alphanumeric()||*c=='_'||*c=='-').collect::<String>();if p.is_empty(){"home".to_string()}else{p}},if dark.get(){" theme-dark"}else{""})><header class=move||format!("public-header{}{}{}",if scrolled.get()||menu.get()||plain{" entered"}else{""},if plain{" plain-header"}else{""},if hidden.get()&&!menu.get(){" header-hidden"}else{""})><a class="brand" href=public_href("/") aria-label=t("xocs 首页","xocs home")>"xocs"</a><nav class=move||if menu.get(){"open"}else{""} aria-label=t("主导航","Main navigation")><span class="public-mobile-title">{t("欢迎光临","Welcome")}</span>{nav.into_iter().map(|(icon,label,path)|view!{<a href=public_href(path) aria-current=(pathname()==path).then_some("page")><span aria-hidden="true">{icon}</span>" "{label}</a>}).collect_view()}</nav><div class="public-header-controls"><xcss_web::components::LanguageToggle/><button class="theme-toggle" type="button" aria-label=move||if dark.get(){t("切换到浅色模式","Switch to light mode")}else{t("切换到深色模式","Switch to dark mode")} title=move||if dark.get(){t("切换到浅色模式","Switch to light mode")}else{t("切换到深色模式","Switch to dark mode")} aria-pressed=move||dark.get().to_string() on:click=move |_|{dark.update(|v|*v= !*v);if let Some(s)=web_sys::window().unwrap().local_storage().ok().flatten(){let _=s.set_item("xocs-theme",if dark.get_untracked(){"dark"}else{"light"});}}><svg aria-hidden="true" width="1em" height="1em" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d=move||if dark.get(){"M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8M12 2v2m0 16v2M2 12h2m16 0h2"}else{"M20.9 13A9 9 0 1 1 11 3a7 7 0 0 0 9.9 10Z"}/></svg></button></div><button class="public-menu-toggle" type="button" aria-label=move||if menu.get(){t("关闭导航","Close navigation")}else{t("打开导航","Open navigation")} aria-expanded=move||menu.get().to_string() on:click=move |_|menu.update(|v|*v= !*v)>{move||if menu.get(){"✕"}else{"☰"}}</button></header>{(!plain).then(||view!{<section class=move||format!("hero{}",if home{" home-hero"}else if metadata.get().is_some(){" article-hero"}else{" compact-hero"}) style=move||{let requested=cover.get();let info=site.value.get().unwrap_or(Value::Null);let image=if requested.is_empty(){let background=text(&info,"background_image");if background.is_empty(){random_cover.get()}else{background}}else{requested};let image=safe_url(&image,false).unwrap_or_else(||"/live/xocs-current-hero.jpg".into());format!("background-image:linear-gradient(#0b2c3e3d,#0b2c3e52),url({})",serde_json::to_string(&image).unwrap_or_default())}><div class=move||if metadata.get().is_some(){"hero-article-info"}else{"hero-center"}><h1>{move||if home{let value=site.value.get().map(|v|text(&v,"web_title")).unwrap_or_default();if value.is_empty(){t("相信记录的力量","Believe in the power of stories")}else{value}}else{title.get()}}</h1>{home.then(||view!{<p class="hero-poem">{t("相信记录的力量！","Believe in the power of stories!")}<span class="cursor">"|"</span></p>})}{move||(!subtitle.get().is_empty()).then(||view!{<p class="hero-subtitle">{subtitle.get()}</p>})}{move||metadata.get().map(|a|view!{<p class="hero-article-meta">{format!("👤 {}　·　📅 {}　·　🔥 {}{}{}{}{}{}",text(&a,"username"),crate::dates::format(&text(&a,"create_time")),num(&a,"view_count"),t(" 热度 · 💬 "," views · 💬 "),num(&a,"comment_count"),t(" 评论 · ❤️ "," comments · ❤️ "),num(&a,"like_count"),t(" 点赞"," likes"))}</p>})}</div>{home.then(||view!{<a class="hero-down" href="#public-content" aria-label=t("浏览文章","Browse articles")>"⌄"</a><div class="hero-wave hero-wave-back" aria-hidden="true"/><div class="hero-wave hero-wave-front" aria-hidden="true"/>})}</section>})}<main id="public-content" class=if home{"public-main"}else{"public-main extra-main"}>{children()}</main>{move||scrolled.get().then(||view!{<div class="public-tools"><button class="back-top" type="button" aria-label=t("返回顶部","Back to top") on:click=move |_|{let opts=web_sys::ScrollToOptions::new();opts.set_top(0.);opts.set_behavior(web_sys::ScrollBehavior::Smooth);web_sys::window().unwrap().scroll_to_with_scroll_to_options(&opts);}><span aria-hidden="true">"↑"</span></button></div>})}</div>}
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
                let _ = cleanup.call0(&wasm_bindgen::JsValue::NULL);
            });
        }
    });
    view! {<span node_ref=node class="sliding-title" title=title.clone()><span class="sliding-title-text">{title.clone()}</span></span>}
}
#[component]
fn Card(item: Value, #[prop(default = false)] compact: bool) -> impl IntoView {
    let title = text(&item, "article_title");
    view! {<a class=if compact{"original-list-card"}else{"article-card"} href=public_href(&format!("/article/{}",num(&item,"id")))><div class=if compact{"original-list-cover"}else{"article-card-cover"}>{if let Some(u)=safe_url(&text(&item,"article_cover"),false){view!{<img src=u alt="" loading="lazy"/>}.into_any()}else{view!{<span>{t("遇事不决，可问春风","Let the spring breeze guide you")}</span>}.into_any()}}</div><div class=if compact{"original-list-body"}else{"article-card-body"}><small>{t("📅 发布于 ","📅 Published ")}{crate::dates::format(&text(&item,"create_time"))}</small><h3><Sliding title/></h3><p class=if compact{"original-list-stats"}else{"article-card-stats"}>{format!("🔥 {}{}{}{}{}{}",num(&item,"view_count"),t(" 热度 💬 "," views 💬 "),num(&item,"comment_count"),t(" 评论 🧡 "," comments 🧡 "),num(&item,"like_count"),t(" 点赞"," likes"))}</p>{compact.then(||view!{<p class="original-list-excerpt">{let s=text(&item,"search_snippet");article_excerpt(&if s.is_empty(){text(&item,"excerpt")}else{s},140)}</p>})}<span class=if compact{"original-list-tags"}else{"article-card-tags"}><span>{format!("📁 {}",text(&item,"sort_name"))}</span><span>{format!("🏷 {}",text(&item,"label_name"))}</span>{(!flag(&item,"view_status")).then(||view!{<span>{t("🔒 密码文章","🔒 Password protected")}</span>})}</span></div></a>}
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
    let stats = load(Signal::derive(|| "/api/v1/site/stats".into()));
    let sections = load(Signal::derive(|| "/api/v1/home-sections".into()));
    let categories = load(Signal::derive(|| "/api/v1/categories".into()));
    let recommended = load(Signal::derive(|| {
        "/api/v1/articles?size=5&recommended=true".into()
    }));
    let labels = load(Signal::derive(|| "/api/v1/labels".into()));
    let messages = load(Signal::derive(|| "/api/v1/tree-hole".into()));
    let push = RwSignal::new(None::<Value>);
    Effect::new(move |_| {
        let items = site
            .value
            .get()
            .map(|v| notices(&text(&v, "notices")))
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
    view! {<Chrome home=true><Feedback error=site.error public=true fallback=t("网站信息加载失败","Unable to load site information") retry=Some(Callback::new(move|()|site.reload()))/><Feedback error=sections.error public=true fallback=t("首页栏目加载失败","Unable to load home sections") retry=Some(Callback::new(move|()|sections.reload()))/><div class="home-layout"><aside class="home-aside" aria-label=t("网站侧栏","Site sidebar")><section class="aside-profile"><div class="aside-avatar"><img src=move||site.value.get().and_then(|v|safe_url(&text(&v,"avatar"),false)).unwrap_or("/legacy/avatar.jpg".into()) alt=t("站长头像","Site owner avatar")/></div><h2>{move||site.value.get().map(|v|text(&v,"web_name")).filter(|s|!s.is_empty()).unwrap_or("xocs".into())}</h2><div class="aside-stats"><span>{t("📖 文章","📖 Articles")}<strong>{move||stats.value.get().map(|v|num(&v,"article_count")).unwrap_or(0)}</strong></span><span>{t("📒 分类","📒 Categories")}<strong>{move||list(categories.value.get()).len()}</strong></span><span>{t("🔥 访问量","🔥 Views")}<strong>{move||stats.value.get().map(|v|num(&v,"view_count")).unwrap_or(0)}</strong></span></div></section><form class="aside-search" action="/search" method="get">{(!query("lang").is_empty()).then(||view!{<input type="hidden" name="lang" value=query("lang")/>})}<label for="home-search">{t("搜索","Search")}</label><div><input id="home-search" name="q" maxlength="200" placeholder=t("搜索文章","Search articles")/><button type="submit" aria-label=t("搜索","Search")>"⌕"</button></div></form>{move||recommended.value.get().filter(|v|!rows(v,"items").is_empty()).map(|v|view!{<section class="aside-featured"><h2>{t("✧ 推荐位","✧ Featured")}</h2>{rows(&v,"items").into_iter().take(2).map(|a|view!{<a href=public_href(&format!("/article/{}",num(&a,"id")))>{safe_url(&text(&a,"article_cover"),false).map(|u|view!{<img src=u alt="" loading="lazy"/>})}<strong>{text(&a,"article_title")}</strong></a>}).collect_view()}</section><section class="aside-recommended"><h2>{t("🔥 推荐文章","🔥 Recommended articles")}</h2>{rows(&v,"items").into_iter().map(|a|view!{<a href=public_href(&format!("/article/{}",num(&a,"id")))>{safe_url(&text(&a,"article_cover"),false).map(|u|view!{<img src=u alt="" loading="lazy"/>})}<span>{text(&a,"article_title")}<small>{format!("◷ {}",crate::dates::format(&text(&a,"create_time")))}</small></span></a>}).collect_view()}</section>})}<section class="aside-labels"><h2>{t("🏷 标签","🏷 Tags")}</h2><div>{move||list(labels.value.get()).into_iter().take(28).map(|l|view!{<a href=public_href(&format!("/sort?sort_id={}&label_id={}",num(&l,"sort_id"),num(&l,"id")))>{text(&l,"label_name")}</a>}).collect_view()}</div></section><section class="aside-latest"><h2>{t("💬 最新弹幕","💬 Recent messages")}</h2><div>{move||list(messages.value.get()).into_iter().take(9).map(|m|view!{<p>{text(&m,"message")}</p>}).collect_view()}</div></section><section class="aside-categories" aria-label=t("分类速览","Category overview")>{move||list(categories.value.get()).into_iter().enumerate().map(|(i,c)|view!{<a href=public_href(&format!("/sort?sort_id={}",num(&c,"id"))) style=format!("background:{}",["linear-gradient(110deg,#358bff,#15c6ff)","linear-gradient(110deg,#18c9a7,#1eebeb)","linear-gradient(110deg,#ff6655,#ffbf37)","linear-gradient(110deg,#8c72e9,#d3a4f4)"][i%4])><small>{t("速览","Overview")}</small><strong>{text(&c,"sort_name")}</strong><span>{text(&c,"sort_description")}</span></a>}).collect_view()}</section></aside><div class="home-content"><div class="announcement"><span aria-hidden="true">"🔊"</span><p>{move||site.value.get().map(|v|notices(&text(&v,"notices")).into_iter().filter(|s|!["推送标题：","推送封面：","推送链接："].iter().any(|p|s.starts_with(p))).collect::<Vec<_>>().join(" · ")).filter(|s|!s.is_empty()).unwrap_or(t("欢迎光临","Welcome"))}</p></div><For each=move||list(sections.value.get()) key=|s|num(s,"id") children=move|section|view!{<HomeSection section/>}/></div></div>{move||push.get().map(|p|view!{<div class="push-backdrop" role="presentation" on:click=move |_|push.set(None)><section class="push-dialog" role="dialog" aria-modal="true" aria-label=t("每日推荐","Daily recommendation") on:click=move|ev|ev.stop_propagation()><button class="push-close" type="button" aria-label=t("关闭每日推荐","Close daily recommendation") on:click=move |_|push.set(None)>"×"</button><h2>{t("每日推荐","Daily recommendation")}</h2><h3>{text(&p,"title")}</h3>{safe_url(&text(&p,"cover"),false).map(|u|view!{<img src=u alt=t("推荐封面","Recommended cover")/>})}<a href=safe_url(&text(&p,"link"),false) target="_blank" rel="noopener noreferrer" on:click=move |_|push.set(None)>{t("立即前往 →","Visit now →")}</a></section></div>})}</Chrome>}
}
#[component]
fn HomeSection(section: Value) -> impl IntoView {
    let kind = text(&section, "kind");
    let filter = match kind.as_str() {
        "recommended" => "&recommended=true".into(),
        "category" => format!("&sort_id={}", num(&section, "sort_id")),
        _ => String::new(),
    };
    let path = format!("/api/v1/articles?size=6{filter}");
    let data = load(Signal::derive(move || path.clone()));
    let more = match kind.as_str() {
        "recommended" => "/sort?recommended=1".into(),
        "category" => format!("/sort?sort_id={}", num(&section, "sort_id")),
        _ => "/sort".into(),
    };
    view! {<section class="home-section" aria-label=text(&section,"title")><div class="section-heading"><h2><span class="section-squares" aria-hidden="true"/>{text(&section,"title")}</h2><a href=public_href(&more)><span class="more-chevrons" aria-hidden="true">"❯❯"</span>" "{t("更多","More")}</a></div><Feedback error=data.error public=true retry=Some(Callback::new(move|()|data.reload()))/><div class="article-grid">{move||data.value.get().map(|v|rows(&v,"items")).unwrap_or_default().into_iter().map(|item|view!{<Card item/>}).collect_view()}</div>{move||data.value.get().filter(|v|rows(v,"items").is_empty()).map(|_|view!{<div class="public-empty-state home-empty-section"><span aria-hidden="true">"✎"</span><h3>{t("这个栏目还没有文章。","No articles in this section yet.")}</h3><p>{t("浏览其它分类，或到留言页分享你的想法。","Browse other categories, or leave a thought on the message wall.")}</p><div><a href=public_href("/sort")>{t("浏览文章","Browse articles")}<span aria-hidden="true">"↗"</span></a><a href=public_href("/message")>{t("给我留言","Leave a message")}<span aria-hidden="true">"↗"</span></a></div></div>})}</section>}
}
#[component]
fn Article(id: i64) -> impl IntoView {
    let data = load(Signal::derive(move || format!("/api/v1/articles/{id}")));
    let access = load(Signal::derive(move || {
        if data.error.get().is_some() {
            format!("/api/v1/articles/{id}/access")
        } else {
            String::new()
        }
    }));
    let unlocked = RwSignal::new(None::<Value>);
    let current = Signal::derive(move || unlocked.get().or(data.value.get()));
    let password = RwSignal::new(String::new());
    let action = Action::new();
    view! {<Chrome title=Signal::derive(move||current.get().map(|v|text(&v,"article_title")).unwrap_or(t("文章","Articles"))) cover=Signal::derive(move||current.get().map(|v|text(&v,"article_cover")).unwrap_or_default()) metadata=current><For each={move||current.get().into_iter().collect::<Vec<_>>()} key=|a|(num(a,"id"),text(a,"article_content")) children=move|article|view!{<Reading article/>}/>{move||if current.get().is_none(){if access.value.get().is_some_and(|v|flag(&v,"password_required")){view!{<form class="reading-card password-form" on:submit=move|ev|{ev.prevent_default();action.run(format!("/api/v1/articles/{id}/unlock"),"POST",Some(json!({"password":password.get_untracked()})),move|v|{unlocked.set(Some(v));password.set(String::new());});}><h2>{t("这篇文章需要访问密码","This article requires an access password")}</h2><p>{access.value.get().map(|v|text(&v,"tips")).unwrap_or_default()}</p><Feedback error=action.error public=true fallback=t("访问密码错误或文章不可用","Incorrect password or article unavailable")/><label>{t("访问密码","Access password")}<input type="password" required prop:value=move||password.get() on:input=move|ev|password.set(event_target_value(&ev))/></label><button type="submit" disabled=move||action.busy.get()>{t("阅读文章","Read article")}</button></form>}.into_any()}else if access.error.get().is_some(){view!{<p role="alert">{t("文章不存在","Article not found")}</p>}.into_any()}else{view!{<p>{t("正在加载文章…","Loading article…")}</p>}.into_any()}}else{().into_any()}}</Chrome>}
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
fn Reading(article: Value) -> impl IntoView {
    let rendered: Value = serde_json::from_str(&crate::browser::reading_markup(
        &text(&article, "article_content"),
        &t("复制代码", "Copy code"),
        &t("查看图片：", "View image: "),
        &t("文章插图", "Article illustration"),
    ))
    .unwrap_or(Value::Null);
    let headings = StoredValue::new(rows(&rendered, "headings"));
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
                doc.get_element_by_id(&text(h, "id"))
                    .is_some_and(|e| e.get_bounding_client_rect().top() <= 150.)
            })
            .map(|h| text(&h, "id"))
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
    let outline = move || view! {<ol>{headings.get_value().into_iter().map(|h|{let id=text(&h,"id");let id2=id.clone();view!{<li class=format!("toc-level-{}",num(&h,"level"))><a class=move||if active.get()==id2{"active"}else{""} href=format!("#{id}")>{text(&h,"title")}</a></li>}}).collect_view()}</ol>};
    let id = num(&article, "id");
    view! {<div class="reading-progress" role="progressbar" aria-label=t("阅读进度","Reading progress") aria-valuemin="0" aria-valuemax="100" aria-valuenow=move||progress.get()><span style=move||format!("width:{}%",progress.get())/></div><div class="reading-layout"><article node_ref=node class="reading-card reading-article"><a href=public_href("/")>{t("← 返回文章","← Back to articles")}</a>{safe_url(&text(&article,"video_url"),false).map(|u|view!{<video class="article-video" controls preload="metadata" poster=safe_url(&text(&article,"article_cover"),false) src=u/>})}{(!flag(&article,"password_required")).then(||view!{<News id/>})}<div class="article-markdown" inner_html=text(&rendered,"html") on:click=move|ev|reading_action(ev.target().and_then(|t|t.dyn_into::<web_sys::Element>().ok()),lightbox,copied) on:keydown=move|ev|{if ["Enter"," "].contains(&ev.key().as_str())&&ev.target().is_some_and(|e|e.is_instance_of::<web_sys::HtmlImageElement>()){ev.prevent_default();reading_action(ev.target().and_then(|t|t.dyn_into::<web_sys::Element>().ok()),lightbox,copied)}}/>{move||(!copied.get().is_empty()).then(||view!{<span class="article-copy-status" role="status">{copied.get()}</span>})}<div class="article-end-info"><p>{t("文章最后更新于 ","Last updated ")}{crate::dates::format(&{let u=text(&article,"update_time");if u.is_empty(){text(&article,"create_time")}else{u}})}</p><a href=public_href(&format!("/sort?sort_id={}&label_id={}",num(&article,"sort_id"),num(&article,"label_id")))>{format!("{} ▶ {}",text(&article,"sort_name"),text(&article,"label_name"))}</a><blockquote>{t("作者：","Author: ")}{text(&article,"username")}<br/>{t("版权与许可请详阅 ","For copyright and licensing, see ")}<button type="button" on:click=move |_|copyright.set(true)>{t("版权声明","Copyright notice")}</button></blockquote></div>{(flag(&article,"comment_status")&&!flag(&article,"password_required")).then(||view!{<Comments article_id=id/>})}</article>{(!headings.get_value().is_empty()).then(||view!{<aside class="reading-toc" aria-label=t("文章目录","Article contents")><nav><h3>{t("目录","Contents")}</h3>{outline()}</nav><details><summary>{t("文章目录 · ","Contents · ")}{move||progress.get()}"%"</summary>{outline()}</details></aside>})}</div>{move||copyright.get().then(||view!{<div class="article-copyright-backdrop" role="presentation" on:click=move |_|copyright.set(false)><section role="dialog" aria-modal="true" aria-label=t("版权声明","Copyright notice") on:click=move|ev|ev.stop_propagation()><button type="button" aria-label=t("关闭版权声明","Close copyright notice") on:click=move |_|copyright.set(false)>"×"</button><h2>{t("版权声明","Copyright notice")}</h2><p>{t("文章、图片和视频的著作权归各自原作者所有。转载或使用内容时，请遵守原作者的许可要求；本站原创内容采用署名－非商业性使用－相同方式共享 4.0 国际许可协议。","Articles, images, and videos belong to their respective authors. Follow their license terms when using or republishing content. Original content on this site is licensed under CC BY-NC-SA 4.0 International.")}</p><p><a href="https://creativecommons.org/licenses/by-nc-sa/4.0/" target="_blank" rel="noopener noreferrer">{t("查看许可协议","View license")}</a></p></section></div>})}<Lightbox src=lightbox/>}
}
#[component]
fn News(id: i64) -> impl IntoView {
    let data = load(Signal::derive(move || {
        format!("/api/v1/articles/{id}/news")
    }));
    view! {<Feedback error=data.error public=true fallback=t("最新进展加载失败","Unable to load recent updates")/>{move||data.value.get().filter(|v|v.as_array().is_some_and(|a|!a.is_empty())).map(|v|view!{<section class="article-news"><h2>{t("最新进展","Recent updates")}</h2><ol>{v.as_array().cloned().unwrap_or_default().into_iter().map(|n|view!{<li><time>{crate::dates::format(&text(&n,"create_time"))}</time><p>{text(&n,"content")}</p></li>}).collect_view()}</ol></section>})}}
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
        views.push(view! {<mark>{value[m.start()..m.end()].to_string()}</mark>}.into_any());
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
    let categories = load(Signal::derive(move || {
        if search {
            String::new()
        } else {
            "/api/v1/categories".into()
        }
    }));
    let labels = load(Signal::derive(move || {
        if !search && selected.get() > 0 {
            format!("/api/v1/labels?sort_id={}", selected.get())
        } else {
            String::new()
        }
    }));
    let data = load(Signal::derive(move || {
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
            list(categories.value.get())
                .iter()
                .find(|c| num(c, "id") == selected.get())
                .map(|c| text(c, "sort_name"))
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
    view! {<Chrome title subtitle=Signal::derive(move||if search{t("在文字里寻找熟悉的故事。","Find familiar stories in these words.")}else{t("就算风吹散了冰雪，想念也会留下来。","Memories remain after the snow has melted.")})>{if search{view!{<form class="search-form" on:submit=move|ev|{ev.prevent_default();let next=draft.get_untracked().trim().to_string();if next==term.get_untracked()&&page.get_untracked()==1{data.reload();}browse(state,&[("q",(!next.is_empty()).then_some(next)),("page",None)]);} ><input aria-label=t("搜索标题或正文","Search titles or content") prop:value=move||draft.get() maxlength="200" placeholder=t("搜索标题或正文","Search titles or content") on:input=move|ev|draft.set(event_target_value(&ev))/><button type="submit">{t("搜索","Search")}</button></form>}.into_any()}else{view!{<div class="original-tag-panel original-category-panel"><button class=move||if selected.get()==0{"active"}else{""} on:click=move |_|choose.run(0)>{t("全部 ","All ")}<small>{move||list(categories.value.get()).iter().map(|c|num(c,"article_count")).sum::<i64>()}</small></button><button class=move||if selected.get()==-1{"active"}else{""} on:click=move |_|choose.run(-1)>{t("推荐","Recommended")}</button>{move||list(categories.value.get()).into_iter().map(|c|{let id=num(&c,"id");view!{<button class=move||if selected.get()==id{"active"}else{""} on:click=move |_|choose.run(id)>{text(&c,"sort_name")}" "<small>{num(&c,"article_count")}</small></button>}}).collect_view()}</div>{move||(!list(labels.value.get()).is_empty()).then(||view!{<div class="original-tag-panel original-label-panel"><button class=move||if label.get()==0{"active"}else{""} on:click=move |_|browse(state,&[("label_id",None),("page",None)])>{t("全部标签 ","All tags ")}</button>{move||list(labels.value.get()).into_iter().map(|l|{let id=num(&l,"id");view!{<button class=move||if label.get()==id{"active"}else{""} on:click=move |_|browse(state,&[("label_id",Some(id.to_string())),("page",None)])>{text(&l,"label_name")}" "<small>{num(&l,"article_count")}</small></button>}}).collect_view()}</div>})}}.into_any()}}<Feedback error=categories.error public=true retry=Some(Callback::new(move|()|categories.reload()))/><Feedback error=labels.error public=true retry=Some(Callback::new(move|()|labels.reload()))/><Feedback error=data.error public=true fallback=if search{t("搜索失败","Search failed")}else{t("文章加载失败","Unable to load articles")} retry=Some(Callback::new(move|()|data.reload()))/>{move||data.busy.get().then(||view!{<p role="status">{if search{t("正在搜索…","Searching…")}else{t("正在加载文章…","Loading articles…")}}</p>})}{move||(search&&!term.get().is_empty()).then(||data.value.get().map(|v|view!{<p class="original-search-count">{format!("“{}{}{}{}",term.get(),t("” 找到 ","” found "),num(&v,"total"),t(" 篇文章；标题匹配优先。"," articles; title matches come first."))}</p>}))}<div class="original-article-list"><h2>{t("🍃 发现","🍃 Discover")}</h2>{move||data.value.get().map(|v|rows(&v,"items")).unwrap_or_default().into_iter().map(|item|{if search{view!{<a class="original-list-card" href=public_href(&format!("/article/{}",num(&item,"id")))><div class="original-list-cover">{safe_url(&text(&item,"article_cover"),false).map(|u|view!{<img src=u alt="" loading="lazy"/>})}</div><div class="original-list-body"><small>{if text(&item,"search_snippet").is_empty(){t("标题匹配","Title match")}else{t("正文匹配","Content match")}}" · "{crate::dates::format(&text(&item,"create_time"))}</small><h3><Highlight value=text(&item,"article_title") term=term.get()/></h3>{(!text(&item,"search_snippet").is_empty()).then(||view!{<p class="search-snippet"><Highlight value=text(&item,"search_snippet") term=term.get()/></p>})}<span class="original-list-tag">{if flag(&item,"view_status"){t("公开","Public")}else{t("需要访问密码","Access password required")}}</span></div></a>}.into_any()}else{view!{<Card item compact=true/>}.into_any()}}).collect_view()}{move||((search&&term.get().is_empty())||data.value.get().is_some_and(|v|rows(&v,"items").is_empty())).then(||view!{<div class="public-empty-state search-empty-state"><span aria-hidden="true">"⌕"</span><h3>{if search{if term.get().is_empty(){t("输入关键词开始搜索","Enter a keyword to start searching")}else{t("没有找到匹配的文章","No matching articles")}}else{t("还没有文章","No articles yet")}}</h3><p>{if search{if term.get().is_empty(){t("可以搜索文章标题，也可以寻找正文里的一句话。","Search for an article title or a phrase from its content.")}else{t("试试其它关键词，或浏览全部文章。","Try another keyword, or browse all articles.")}}else{t("第一篇记录正在路上。","The first story is on its way.")}}</p><a href=public_href("/sort")>{t("浏览全部文章","Browse all articles")}<span aria-hidden="true">"↗"</span></a></div>})}<BrowsePager data state size=12 always=true empty=search/></div></Chrome>}
}
#[component]
fn BrowsePager(
    data: Data,
    state: RwSignal<String>,
    size: i64,
    #[prop(default=t("分页","Pagination"))] label: String,
    #[prop(default = false)] always: bool,
    #[prop(default = false)] empty: bool,
) -> impl IntoView {
    move || {
        data.value.get().filter(|v|empty||num(v,"total")>if always{0}else{size}||page_number(state)>1).map(|v|{let total=num(&v,"total");view!{<nav class="pager" aria-label=label.clone()><button type="button" disabled=move||data.busy.get()||page_number(state)<=1 on:click=move |_|{let page=page_number(state).saturating_sub(1).max(1);browse(state,&[("page",(page>1).then(||page.to_string()))]);}>{t("上一页","Previous page")}</button><span>{tr("第 {0} 页","Page {0}",&[page_number(state).to_string()])}</span><button type="button" disabled={move||data.busy.get()||(page_number(state) as i64*size)>=total} on:click=move |_|browse(state,&[("page",Some((page_number(state)+1).to_string()))])>{t("下一页","Next page")}</button></nav>}})
    }
}
#[component]
fn Journey() -> impl IntoView {
    let state = search_signal();
    let data = load(Signal::derive(move || {
        format!("/api/v1/articles?page={}&size=20", page_number(state))
    }));
    view! {<Chrome title=Signal::derive(||"xocs".into()) cover=Signal::derive(||"/live/xocs-current-menory.jpg".into())><Petals/><div class="journey-page"><header><strong>{t("时间线","Timeline")}</strong><small>{t("灵魂在路上","The soul is walking")}</small></header><Feedback error=data.error public=true retry=Some(Callback::new(move|()|data.reload()))/><div class="journey-grid">{move||data.value.get().map(|v|{let items=rows(&v,"items");let len=items.len();items.into_iter().enumerate().map(|(i,item)|{let row=i/5;let reverse=row%2==1;view!{<a href=public_href(&format!("/article/{}",num(&item,"id"))) class=if reverse{"journey-item journey-reverse"}else{"journey-item"} style=format!("grid-column:{};grid-row:{}",if reverse{5-i%5}else{i%5+1},row+1)><span class="journey-image"><span class="journey-picture">{safe_url(&text(&item,"article_cover"),false).map(|u|view!{<img src=u alt="" loading="lazy"/>})}</span>{(i<len-1).then(||view!{<span class=if i%5==4{"journey-connector journey-turn"}else{"journey-connector"} aria-hidden="true"/>})}</span><strong><Sliding title=text(&item,"article_title")/></strong><small><time datetime=text(&item,"create_time")>{crate::dates::format(&text(&item,"create_time"))}</time><span>{num(&v,"total")-i as i64-(page_number(state) as i64-1)*20}</span></small></a>}}).collect_view()})}</div><BrowsePager data state size=20/></div></Chrome>}
}
#[component]
fn Journal() -> impl IntoView {
    let state = search_signal();
    let data = load(Signal::derive(move || {
        format!("/api/v1/notes/page?page={}", page_number(state))
    }));
    let lightbox = RwSignal::new(None::<String>);
    view! {<Chrome title=Signal::derive(||"xocs".into()) cover=Signal::derive(||"/live/xocs-current-jotting.jpg".into())><div class="wall-layout wall-reading-layout"><div class="wall-list"><Feedback error=data.error public=true retry=Some(Callback::new(move|()|data.reload()))/>{move||data.value.get().map(|v|rows(&v,"items")).unwrap_or_default().into_iter().map(|item|view!{<NoteCard item lightbox/>}).collect_view()}{move||data.value.get().filter(|v|rows(v,"items").is_empty()).map(|_|view!{<div class="public-empty-state"><h3>{t("还没有记录。","No entries yet.")}</h3><p>{t("新的日常记录正在路上。","New everyday stories are on their way.")}</p></div>})}<BrowsePager data state size=10 label=t("记录分页","Entry pages")/></div></div><Lightbox src=lightbox/></Chrome>}
}
#[component]
fn NoteCard(
    item: Value,
    lightbox: RwSignal<Option<String>>,
    #[prop(default = false)] love: bool,
) -> impl IntoView {
    view! {<article class=if love{"love-moment-card"}else{"wall-card"}><small>{let n=text(&item,"username");if n.is_empty(){t("站长","Site owner")}else{n}}" · "{crate::dates::format(&text(&item,"create_time"))}</small><p>{text(&item,"content")}</p>{safe_url(&text(&item,"image_path"),true).map(|u|{let click=u.clone();view!{<button type="button" class="wall-photo-button" aria-label=t("放大图片","Enlarge image") on:click=move |_|lightbox.set(Some(click.clone()))><img src=u alt=if love{t("随笔图片","Note image")}else{t("动态图片","Post image")} loading="lazy"/></button>}})}</article>}
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
    view! {<Chrome plain=true title=Signal::derive(||t("留言板","Message board"))><div class="message-page"><section class="message-hero"><div class="message-barrage" aria-label=t("留言弹幕","Message wall")>{move||posts.get().into_iter().enumerate().map(|(i,item)|view!{<div class="message-barrage-item" style=format!("animation-delay:{}s;top:{}%",-((i%10) as f64)*2.3,8+i%7*12)>{safe_url(&text(&item,"avatar"),false).map(|u|view!{<img src=u alt=""/>})}<span>{text(&item,"message")}</span></div>}).collect_view()}</div><div class="message-form-wrap"><h1>{t("弹幕","Message wall")}</h1><form on:submit=move|ev|{ev.prevent_default();let content=draft.get_untracked();if content.trim().is_empty(){return;}action.run("/api/v1/tree-hole/guest".into(),"POST",Some(json!({"message":content.trim()})),move|v|{posts.update(|items|{items.retain(|i|i["id"]!=v["id"]);items.insert(0,v)});if draft.get_untracked()==content{draft.set(String::new())}});}><input aria-label=t("发送弹幕","Send a message") maxlength="60" prop:value=move||draft.get() on:input=move|ev|draft.set(event_target_value(&ev)) placeholder=t("留下点什么啦～","Leave a message…")/><button disabled=move||action.busy.get()||draft.get().trim().is_empty()>{move||if action.busy.get(){t("发射中…","Sending…")}else{t("发射","Send")}}</button></form><p class="message-anonymous">{t("弹幕将匿名发布，无需登录。","Messages are posted anonymously. No account is needed.")}</p><Feedback error=data.error public=true retry=Some(Callback::new(move|()|data.reload()))/><Feedback error=action.error public=true/></div></section><div class="message-comment-wrap"><Comments kind="message"/></div></div></Chrome>}
}
#[component]
fn PhotoGrid(items: Signal<Vec<Value>>, lightbox: RwSignal<Option<String>>) -> impl IntoView {
    view! {<div class="travel-grid">{move||items.get().into_iter().map(|item|{let cover=safe_url(&text(&item,"cover"),true);let target=cover.clone();let title={let n=text(&item,"title");if n.is_empty(){let c=text(&item,"classify");if c.is_empty(){t("相册照片","Album photo")}else{c}}else{n}};view!{<button class="travel-photo-card" type="button" disabled=cover.is_none() aria-label=format!("{}{}",t("查看照片：","View photo: "),title) on:click=move |_|lightbox.set(target.clone())><span class="travel-photo-image">{if let Some(url)=cover.clone(){view!{<img src=url alt="" loading="lazy" decoding="async"/>}.into_any()}else{view!{<span>{t("记录美好瞬间","Capture beautiful moments")}</span>}.into_any()}}</span><span class="travel-photo-body"><strong>{title.clone()}</strong><small>{t("日期（UTC）：","Date (UTC): ")}{text(&item,"create_time").get(..10).unwrap_or_default().to_string()}</small></span></button>}}).collect_view()}{move||items.get().is_empty().then(||view!{<div class="travel-photo-empty public-empty-state"><span aria-hidden="true">"▧"</span><h3>{t("这一组还没有照片","No photos in this collection yet")}</h3><p>{t("切换其它相册，或稍后回来看看新的回忆。","Try another collection, or come back for new memories.")}</p></div>})}</div>}
}
#[component]
fn Photos(love: bool) -> impl IntoView {
    view! {<Chrome plain=true title=Signal::derive(||t("时光相册","Photo album"))><div class="travel-page"><section class="travel-banner"><div><h1>{t("时光相册","Photo album")}</h1><h2>{t("每一张照片都是一次美好的记忆","Every photo holds a memory")}</h2></div></section><div class="travel-content"><Collection love active=Signal::derive(||true)/></div></div></Chrome>}
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
    view! {<div class=if love{"love-photo-collection"}else{"travel-photo-collection"}><div class=if love{"original-tag-panel love-photo-filters"}else{"original-tag-panel"} aria-label=t("相册分类","Photo categories")><button type="button" class=move||if selected.get().is_empty(){"active"}else{""} aria-pressed=move||selected.get().is_empty().to_string() on:click=move |_|selected.set(String::new())>{if love{t("全部","All")}else{t("全部照片","All photos")}}" "<small>{move||all_total.get().map(|n|n.to_string()).unwrap_or_default()}</small></button>{move||list(classes.value.get()).into_iter().map(|c|{let title=text(&c,"classify");let checked=title.clone();let click=title.clone();view!{<button type="button" class=move||if selected.get()==checked{"active"}else{""} aria-pressed={let checked=title.clone();move||(selected.get()==checked).to_string()} on:click=move |_|selected.set(click.clone())>{title.clone()}" "<small>{num(&c,"count")}</small></button>}}).collect_view()}</div><Feedback error=classes.error public=true fallback=t("相册分类加载失败","Unable to load photo categories") retry=Some(Callback::new(move|()|classes.reload()))/><div class="love-section-heading"><div><h2>{move||{let v=selected.get();if v.is_empty(){if love{t("时光相册","Photo album")}else{t("全部照片","All photos")}}else{v}}}</h2>{love.then(||view!{<p>{t("收藏一起看过的风景。","Keep the places and moments we have shared.")}</p>})}</div>{move||love.then(||total.get().map(|n|view!{<span>{format!("{n} {}",t("张照片","photos"))}</span>}))}</div><Feedback error=data.error public=true fallback=t("相册加载失败","Unable to load photos") retry=Some(Callback::new(move|()|{page.set(loaded.get_untracked()+1);data.reload();})) />{move||(data.busy.get()&&items.get().is_empty()).then(||view!{<p class="public-loading" role="status">{t("正在加载照片…","Loading photos…")}</p>})}<Show when=move||!items.get().is_empty()||(!data.busy.get()&&data.error.get().is_none())><PhotoGrid items=Signal::derive(move||items.get()) lightbox/></Show>{move||total.get().map(|total|if(items.get().len() as i64)<total{view!{<button type="button" class="travel-more" disabled=move||data.busy.get() on:click=move |_|{page.set(loaded.get_untracked()+1);data.reload();}>{if love{t("加载更多照片","Load more photos")}else{t("下一页","Next page")}}</button>}.into_any()}else if !love{view!{<p class="travel-end">{t("~~到底啦~~","~~You have reached the end~~")}</p>}.into_any()}else{().into_any()})}<Lightbox src=lightbox/></div>}
}
#[component]
fn Letter() -> impl IntoView {
    let data = load(Signal::derive(|| "/api/v1/letter".into()));
    let open = RwSignal::new(false);
    let front = NodeRef::<leptos::html::Button>::new();
    let close = NodeRef::<leptos::html::Button>::new();
    view! {<Chrome plain=true title=Signal::derive(||t("信笺","Letter"))><div class="letter-page"><h1>{t("信笺","Letter")}</h1><div class="letter-sakura" aria-hidden="true">"✿　❀　✿　❀　✿"</div><Feedback error=data.error public=true fallback=t("信笺加载失败","Unable to load the letter") retry=Some(Callback::new(move|()|data.reload()))/>{move||data.value.get().map(|letter|if letter.is_null(){view!{<section class="public-empty-state"><h2>{t("暂未公开信笺","No letter is published yet")}</h2></section>}.into_any()}else{view!{<div class=move||if open.get(){"letter-envelope opened"}else{"letter-envelope"}><div class="letter-envelope-front" aria-hidden=move||open.get().to_string() inert=move||open.get()><span>{format!("{}{}",t("致 ","To "),text(&letter,"recipient"))}</span><button node_ref=front type="button" on:click=move |_|{open.set(true);gloo_timers::callback::Timeout::new(0,move||{if let Some(close)=close.get(){let _=close.focus();}}).forget();}>{t("打开信封","Open letter")}</button></div><div class="letter-paper" aria-hidden=move||(!open.get()).to_string() inert=move||!open.get()><h2>{format!("{}{}",t("致 ","To "),text(&letter,"recipient"))}</h2><p class="letter-content">{text(&letter,"content")}</p><strong>{text(&letter,"sender")}</strong><button node_ref=close type="button" on:click=move |_|{open.set(false);gloo_timers::callback::Timeout::new(0,move||{if let Some(front)=front.get(){let _=front.focus();}}).forget();}>{t("合上信封","Close letter")}</button></div></div>}.into_any()})}</div></Chrome>}
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
    view! {<Chrome plain=true title=Signal::derive(||t("百宝箱","Toolbox"))><div class="favorite-page"><section class="favorite-banner"><video autoplay muted loop playsinline preload="metadata" poster="/legacy/backgroundPicture.jpg" src="/legacy/backgroundVideo.mp4"/><div class="favorite-banner-overlay"><small>{t("记录","Articles")}</small><h1>{t("百宝箱","Toolbox")}</h1><div class="favorite-feature-links">{[("favorites",t("收藏夹","Favorites"),t("将本网站添加到您的收藏夹吧","Add this site to your favorites")),("music",t("音乐","Music"),t("一曲肝肠断，天涯何处觅知音","Music brings kindred spirits together")),("friends",t("友人帐","Friends"),t("看看朋友们的网站","Explore our friends’ websites"))].into_iter().map(|(id,l,desc)|view!{<button class=move||if tab.get()==id{"active"}else{""} aria-pressed=move||(tab.get()==id).to_string() on:click=move |_|browse(state,&[("tab",Some(id.into())),("page",None)])><strong>{l}</strong><span>{desc}</span></button>}).collect_view()}</div></div></section><div class="favorite-groups"><Feedback error=data.error public=true fallback=t("内容加载失败","Unable to load links") retry=Some(Callback::new(move|()|data.reload()))/>{move||if tab.get()=="music"{data.value.get().map(|v|view!{<Music items=rows(&v,"items")/>}).into_any()}else{let items=data.value.get().map(|v|rows(&v,"items")).unwrap_or_default();let mut grouped=std::collections::BTreeMap::<String,Vec<Value>>::new();for item in items{let name=text(&item,"classify");let name=if name.is_empty(){if tab.get()=="friends"{t("🥇友情链接","🥇 Friend links")}else{t("我的收藏","My favorites")}}else{name};grouped.entry(name).or_default().push(item);}
    if tab.get()=="friends"{let mut order=vec![t("♥️青出于蓝","♥️ Inspiration"),t("🥇友情链接","🥇 Friend links")];order.extend(grouped.keys().filter(|n|!order.contains(n)).cloned().collect::<Vec<_>>());view!{<div class="friend-page"><section class="friend-site-info"><h2>{t("🌸 本站信息","🌸 Site information")}</h2><div><p><b>{t("网站名称","Site name")}</b><span>{site.value.get().map(|v|text(&v,"web_name")).unwrap_or("xocs".into())}</span></p><p><b>{t("网址","URL")}</b><a href=web_sys::window().unwrap().location().origin().unwrap_or_default()>{web_sys::window().unwrap().location().origin().unwrap_or_default()}</a></p>{site.value.get().and_then(|v|safe_url(&text(&v,"avatar"),false)).map(|u|view!{<p><b>{t("头像","Avatar")}</b><a href=u target="_blank" rel="noopener noreferrer">{t("查看头像","View avatar")}</a></p>})}<p><b>{t("描述","Description")}</b><span>{site.value.get().map(|v|text(&v,"web_title")).unwrap_or(t("有诗意地记录生活","Record life through poetry"))}</span></p></div></section>{order.into_iter().map(|name|{let items=grouped.remove(&name).unwrap_or_default();view!{<FavoriteGroup name items/>}}).collect_view()}</div>}.into_any()}else{view!{<>{grouped.into_iter().map(|(name,items)|view!{<FavoriteGroup name items/>}).collect_view()}{(data.value.get().is_some()&&data.value.get().is_some_and(|v|rows(&v,"items").is_empty())).then(||view!{<p class="favorite-empty">{t("还没有收藏的内容。","No favorites yet.")}</p>})}</>}.into_any()}}}<BrowsePager data state size=24 label=t("百宝箱分页","Toolbox pages")/></div></div></Chrome>}
}
#[component]
fn FavoriteGroup(name: String, items: Vec<Value>) -> impl IntoView {
    let empty = items.is_empty();
    view! {<section class="favorite-link-group"><h2>{name}</h2><div class="favorite-link-grid">{items.into_iter().map(|item|{let href=safe_url(&text(&item,"url"),false);view!{<a class="favorite-link-card" href=href.clone() target=href.is_some().then_some("_blank") rel="noopener noreferrer">{if let Some(u)=safe_url(&text(&item,"cover"),false){view!{<img src=u alt="" loading="lazy"/>}.into_any()}else{view!{<span class="favorite-link-placeholder"/>}.into_any()}}<span><strong>{let n=text(&item,"title");if n.is_empty(){t("未命名","Untitled")}else{n}}</strong><small>{text(&item,"introduction")}</small></span></a>}}).collect_view()}</div>{empty.then(||view!{<p class="favorite-group-empty">{t("这里还没有友链。","No friend links yet.")}</p>})}</section>}
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
    view! {<div class="music-layout"><div class="wall-card">{items.get_value().is_empty().then(||view!{<h2>{t("选择一首曲目","Select a track")}</h2>})}{move||items.get_value().get(selected.get()).cloned().map(|item|view!{<h2>{text(&item,"title")}</h2>{safe_url(&text(&item,"cover"),false).map(|u|view!{<img class="music-cover" src=u alt=""/>})}<audio controls src=safe_url(&text(&item,"url"),false) on:ended=move |_|selected.update(|i|*i=(*i+1).min(items.get_value().len().saturating_sub(1)))/>})}</div><div class="wall-card"><h2>{t("播放列表","Playlist")}</h2>{items.get_value().into_iter().enumerate().map(|(i,item)|view!{<button class=move||if selected.get()==i{"selected"}else{""} on:click=move |_|selected.set(i)>{text(&item,"title")}<small>{text(&item,"classify")}</small></button>}).collect_view()}{items.get_value().is_empty().then(||view!{<p>{t("暂无曲目。","No tracks yet.")}</p>})}</div></div>}
}
#[component]
fn Avatar(name: String, src: String) -> impl IntoView {
    view! {<span class="love-avatar">{if let Some(u)=safe_url(&src,true){view!{<img src=u alt=""/>}.into_any()}else{view!{<span aria-hidden="true">{name.chars().next().unwrap_or(' ')}</span>}.into_any()}}</span>}
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
    view! {<Chrome plain=true title=Signal::derive(||t("恋爱笔记","Love journal"))><Petals/><div class="love-page"><section class="love-hero" style=move||safe_url(&text(&current.get(),"bg_cover"),true).map(|u|format!("background-image:url({})",serde_json::to_string(&u).unwrap_or_default())).unwrap_or_default()><div class="love-hero-inner"><div class="love-pair-card"><div class="love-pair"><div>{move||view!{<Avatar name=his.get() src=text(&current.get(),"man_cover")/>}}<strong>{move||his.get()}</strong></div><img class="love-heart" src="/legacy/love-heart.svg" alt="" aria-hidden="true"/><div>{move||view!{<Avatar name=her.get() src=text(&current.get(),"woman_cover")/>}}<strong>{move||her.get()}</strong></div></div><p>{move||if crate::dates::parse(&text(&current.get(),"timing")).is_some(){format!("{}{}",t("故事开始于 ","Our story began on "),crate::dates::format(&text(&current.get(),"timing")))}else{t("等待第一份属于我们的回忆","A place for our first memory together")}}</p></div></div><svg class="love-wave" viewBox="0 0 1440 110" preserveAspectRatio="none" aria-hidden="true"><path opacity=".45" d="M0 35C180 100 280 0 440 45S700 95 870 40S1170 0 1440 45V110H0Z"/><path opacity=".7" d="M0 55C150 15 280 95 450 60S730 10 920 65S1210 80 1440 35V110H0Z"/><path d="M0 70C180 100 340 30 530 75S850 90 1060 55S1300 80 1440 65V110H0Z"/></svg></section><div class="love-content"><Feedback error=family.error public=true fallback=t("故事加载失败","Unable to load our story") retry=Some(Callback::new(move|()|family.reload()))/><section class="love-timer"><h1>{t("这是我们一起走过的","Our journey together")}</h1>{move||if let Some(e)=elapsed.get(){view!{<div class="love-clock" aria-label=t("相伴时长","Time spent together")><span class="love-clock-prefix">{t("第","Together for")}</span>{[("years",e.years as i64,t("年",if e.years==1{"year"}else{"years"})),("months",e.months as i64,t("月",if e.months==1{"month"}else{"months"})),("days",e.days,t("日",if e.days==1{"day"}else{"days"})),("hours",e.hours,t("时",if e.hours==1{"hour"}else{"hours"})),("minutes",e.minutes,t("分",if e.minutes==1{"minute"}else{"minutes"})),("seconds",e.seconds,t("秒",if e.seconds==1{"second"}else{"seconds"}))].into_iter().map(|(key,v,l)|view!{<div data-unit=key><strong>{v}</strong><span>{l}</span></div>}).collect_view()}</div>}.into_any()}else{view!{<div class="love-timer-empty"><strong>{if crate::dates::parse(&text(&current.get(),"timing")).is_some(){t("新的旅程即将开始","A new journey begins soon")}else{t("从第一份回忆开始","Start with your first memory")}}</strong><p>{t("留下相识的日期，让时间替我们记住每一个日子。","Add the day you met and keep track of each day together.")}</p></div>}.into_any()}}{move||crate::dates::parse(&text(&current.get(),"countdown_time")).and_then(|target|chrono::DateTime::from_timestamp_millis(now.get()).map(|n|(target.date_naive()-n.date_naive()).num_days())).map(|days|view!{<p class="love-countdown"><span>{text(&current.get(),"countdown_title")}"："</span><strong>{if days>0{format!("{} {days} {}",t("还有",""),t("天","days to go"))}else if days==0{t("就是今天","Today is the day")}else{format!("{} {} {}",t("已过去",""),-days,t("天","days ago"))}}</strong></p>})}</section><a class="love-letter" href=public_href("/letter")><span>{t("飞车传信","A letter for you")}</span></a><details class="love-story-card"><summary>{t("我们的故事","Our story")}</summary><p>{move||{let story=text(&current.get(),"family_info");if story.is_empty(){t("把平常的日子写下来，让照片和文字慢慢填满这个家。","Make a home for everyday moments, one photo and one story at a time.")}else{story}}}</p><a href=public_href("/jotting")>{t("看看日常记录","Explore our journal")}<span aria-hidden="true">"↗"</span></a></details><div node_ref=content class="love-workspace"><nav class="love-tabs" aria-label=t("恋爱笔记内容","Love journal content")>{[("notes",t("点点滴滴","Moments"),"/legacy/love-moments.jpg",t("☀️ 今朝有酒今朝醉","☀️ Little moments of joy")),("photos",t("时光相册","Photos"),"/legacy/love-photos.jpg",t("📸 记录美好瞬间","📸 Keep beautiful memories")),("wishes",t("祝福板","Wishes"),"/legacy/love-wishes.jpg",t("📋 写下你们的祝福","📋 Leave your wishes"))].into_iter().map(|(id,l,icon,desc)|view!{<button type="button" class=move||if tab.get()==id{"active"}else{""} aria-pressed=move||(tab.get()==id).to_string() aria-controls="love-panel" on:click=move |_|{tab.set(id.into());if let Some(c)=content.get(){c.scroll_into_view();}}><img class="love-tab-icon" src=icon alt=""/><span><strong>{l}</strong><small>{desc}</small></span></button>}).collect_view()}</nav><section id="love-panel" class="love-panel" aria-label=move||match tab.get().as_str(){"notes"=>t("点点滴滴","Moments"),"wishes"=>t("祝福板","Wishes"),_=>t("时光相册","Photos")}><div hidden=move||tab.get()!="photos"><Collection love=true active=Signal::derive(move||tab.get()=="photos")/></div><div hidden=move||tab.get()!="notes"><div class="love-section-heading"><div><h2>{t("点点滴滴","Moments")}</h2><p>{t("记录那些平常又特别的小事。","A place for the little things that make our days.")}</p></div><a href=public_href("/jotting")>{t("浏览更多记录","Browse more entries")}<span aria-hidden="true">"↗"</span></a></div><Feedback error=notes.error public=true retry=Some(Callback::new(move|()|notes.reload()))/>{move||notes.busy.get().then(||view!{<p class="public-loading" role="status">{t("正在加载记录…","Loading entries…")}</p>})}<div class="love-moments-grid">{move||notes.value.get().map(|v|rows(&v,"items")).unwrap_or_default().into_iter().map(|item|view!{<NoteCard item lightbox love=true/>}).collect_view()}{move||notes.value.get().is_some_and(|v|rows(&v,"items").is_empty()).then(||view!{<div class="public-empty-state"><span aria-hidden="true">"✎"</span><h3>{t("还没有记录。","No entries yet.")}</h3><p>{t("写下此刻的想法，留住每一个值得纪念的日常。","Write a thought and keep a little piece of today.")}</p></div>})}</div><Pager data=notes page=notes_page public=true label=t("记录分页","Entry pages")/></div>{move||(tab.get()=="wishes").then(||view!{<div class="love-wishes"><Comments kind="love" title=t("祝福板","Wishes")/></div>})}</section></div></div></div><Lightbox src=lightbox/></Chrome>}
}
