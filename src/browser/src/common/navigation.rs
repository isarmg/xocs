//! Browser URLs and page-owned navigation listeners.
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;

pub(super) const MAX_PAGE: u64 = 100_000;

pub(crate) fn enc(s: &str) -> String {
    js_sys::encode_uri_component(s)
        .as_string()
        .expect("invariant: encodeURIComponent returns a string")
}

pub(crate) fn navigate(hash: &str) {
    if let Err(error) = web_sys::window()
        .expect("invariant: the application is mounted in a browser window")
        .location()
        .set_hash(hash)
    {
        web_sys::console::warn_2(&"Unable to navigate to the selected page".into(), &error);
    }
}

fn hash() -> String {
    web_sys::window()
        .expect("invariant: the application is mounted in a browser window")
        .location()
        .hash()
        .unwrap_or_default()
        .trim_start_matches('#')
        .to_string()
}

pub(crate) fn pathname() -> String {
    web_sys::window()
        .expect("invariant: the application is mounted in a browser window")
        .location()
        .pathname()
        .unwrap_or_default()
}

pub(crate) fn href(path: &str) -> String {
    let lang = query("lang");
    if lang.is_empty() || path.starts_with('#') {
        return path.into();
    }
    let Some(win) = web_sys::window() else {
        return path.into();
    };
    let Ok(url) = web_sys::Url::new_with_base(path, &win.location().href().unwrap_or_default())
    else {
        return path.into();
    };
    if url.origin() != win.location().origin().unwrap_or_default()
        || !["http:", "https:"].contains(&url.protocol().as_str())
    {
        return path.into();
    }
    url.search_params()
        .set("lang", &blog_interface::i18n::locale());
    format!("{}{}{}", url.pathname(), url.search(), url.hash())
}

pub(crate) fn query(key: &str) -> String {
    web_sys::UrlSearchParams::new_with_str(
        &web_sys::window()
            .expect("invariant: the application is mounted in a browser window")
            .location()
            .search()
            .unwrap_or_default(),
    )
    .expect("invariant: URLSearchParams accepts any search string")
    .get(key)
    .unwrap_or_default()
}

pub(crate) fn hash_signal() -> RwSignal<String> {
    let s = RwSignal::new(hash());
    let listener = SendWrapper::new(blog_interface::events::EventListener::new(
        &web_sys::window().expect("invariant: the application is mounted in a browser window"),
        "hashchange",
        move |_| s.set(hash()),
    ));
    on_cleanup(move || drop(listener));
    s
}

pub(crate) fn search_signal() -> RwSignal<String> {
    let s = RwSignal::new(
        web_sys::window()
            .expect("invariant: the application is mounted in a browser window")
            .location()
            .search()
            .unwrap_or_default(),
    );
    let listener = SendWrapper::new(blog_interface::events::EventListener::new(
        &web_sys::window().expect("invariant: the application is mounted in a browser window"),
        "popstate",
        move |_| {
            s.set(
                web_sys::window()
                    .expect("invariant: the application is mounted in a browser window")
                    .location()
                    .search()
                    .unwrap_or_default(),
            )
        },
    ));
    on_cleanup(move || drop(listener));
    s
}

pub(crate) fn browse(s: RwSignal<String>, changes: &[(&str, Option<String>)]) {
    update_search(s, changes, HistoryChange::Push);
}

pub(super) fn replace_browse(s: RwSignal<String>, changes: &[(&str, Option<String>)]) {
    update_search(s, changes, HistoryChange::Replace);
}

enum HistoryChange {
    Push,
    Replace,
}

fn update_search(s: RwSignal<String>, changes: &[(&str, Option<String>)], change: HistoryChange) {
    let url = web_sys::Url::new(
        &web_sys::window()
            .expect("invariant: the application is mounted in a browser window")
            .location()
            .href()
            .unwrap_or_default(),
    )
    .expect("invariant: the current browser location is a valid URL");
    let params = url.search_params();
    for (k, v) in changes {
        if let Some(v) = v {
            params.set(k, v)
        } else {
            params.delete(k)
        }
    }
    let relative = format!("{}{}{}", url.pathname(), url.search(), url.hash());
    let history = web_sys::window()
        .expect("invariant: the application is mounted in a browser window")
        .history()
        .expect("invariant: same-origin window history is accessible");
    let result = match change {
        HistoryChange::Replace => history
            .state()
            .and_then(|state| history.replace_state_with_url(&state, "", Some(&relative))),
        HistoryChange::Push => {
            history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&relative))
        }
    };
    if let Err(error) = result {
        web_sys::console::warn_2(&"Unable to update page navigation".into(), &error);
        return;
    }
    s.set(url.search());
}

pub(crate) fn param(s: RwSignal<String>, key: &str) -> String {
    web_sys::UrlSearchParams::new_with_str(&s.get())
        .expect("invariant: URLSearchParams accepts any search string")
        .get(key)
        .unwrap_or_default()
}

pub(crate) fn page_number(s: RwSignal<String>) -> u64 {
    param(s, "page")
        .parse::<u64>()
        .ok()
        .filter(|p| *p > 0)
        .unwrap_or(1)
        .min(MAX_PAGE)
}

pub(crate) fn safe_url(s: &str, blob: bool) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    let url = web_sys::Url::new_with_base(
        s,
        &web_sys::window()
            .expect("invariant: the application is mounted in a browser window")
            .location()
            .origin()
            .unwrap_or_default(),
    )
    .ok()?;
    (["http:", "https:"].contains(&url.protocol().as_str()) || blob && url.protocol() == "blob:")
        .then(|| url.href())
}
