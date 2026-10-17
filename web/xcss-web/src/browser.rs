//! Browser policies and reading interactions, implemented through typed web-sys.
//! The application ships no handwritten JavaScript module.
use crate::common::t;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, closure::Closure};

pub fn markdown_html(content: &str) -> String {
    crate::markdown::html(content)
}

#[derive(Clone)]
pub struct Heading {
    pub id: String,
    pub title: String,
    pub level: u32,
}
pub struct ReadingMarkup {
    pub html: String,
    pub headings: Vec<Heading>,
}
pub fn reading_markup(content: &str, copy: &str, image: &str, section: &str) -> ReadingMarkup {
    let html = markdown_html(content);
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return ReadingMarkup {
            html,
            headings: vec![],
        };
    };
    let Ok(holder) = doc.create_element("div") else {
        return ReadingMarkup {
            html,
            headings: vec![],
        };
    };
    // This string contains only pulldown-cmark's escaped, URL-filtered output.
    holder.set_inner_html(&html);
    let mut headings = Vec::new();
    if let Ok(nodes) = holder.query_selector_all("h2,h3,h4") {
        for index in 0..nodes.length() {
            let Some(node) = nodes
                .item(index)
                .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
            else {
                continue;
            };
            let id = format!("section-{}", index + 1);
            node.set_id(&id);
            let title = node.text_content().unwrap_or_default().trim().to_owned();
            headings.push(Heading {
                id,
                title: if title.is_empty() {
                    format!("{section} {}", index + 1)
                } else {
                    title
                },
                level: node.tag_name()[1..].parse().unwrap_or(2),
            });
        }
    }
    if let Ok(nodes) = holder.query_selector_all("pre") {
        for index in 0..nodes.length() {
            if let Some(pre) = nodes
                .item(index)
                .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
                && let Ok(button) = doc.create_element("button")
            {
                let _ = button.set_attribute("type", "button");
                button.set_class_name("article-copy-code");
                button.set_text_content(Some(copy));
                let _ = pre.prepend_with_node_1(&button);
            }
        }
    }
    if let Ok(nodes) = holder.query_selector_all("img") {
        for index in 0..nodes.length() {
            if let Some(img) = nodes
                .item(index)
                .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
            {
                let _ = img.set_attribute("role", "button");
                let _ = img.set_attribute("tabindex", "0");
                let alt = img
                    .get_attribute("alt")
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| section.into());
                let _ = img.set_attribute("aria-label", &format!("{image}{alt}"));
            }
        }
    }
    ReadingMarkup {
        html: holder.inner_html(),
        headings,
    }
}

pub struct SlidingObserver {
    observer: web_sys::ResizeObserver,
    _callback: Closure<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>,
    active: Rc<Cell<bool>>,
}
impl Drop for SlidingObserver {
    fn drop(&mut self) {
        self.active.set(false);
        self.observer.disconnect();
    }
}
pub fn observe_sliding(viewport: &web_sys::Element) -> Option<SlidingObserver> {
    let label = viewport
        .query_selector(".sliding-title-text")
        .ok()
        .flatten()?;
    let viewport = viewport.dyn_ref::<web_sys::HtmlElement>()?.clone();
    let active = Rc::new(Cell::new(true));
    let measure = {
        let viewport = viewport.clone();
        let label = label.clone();
        let active = active.clone();
        move || {
            if !active.get() {
                return;
            }
            let distance = (label.scroll_width() - viewport.client_width()).max(0) as f64;
            let _ = viewport
                .style()
                .set_property("--title-travel", &format!("{distance}px"));
            let _ = viewport.style().set_property(
                "--title-duration",
                &format!("{}s", (distance / 28. + 3.).max(6.)),
            );
            let _ = viewport
                .class_list()
                .toggle_with_force("is-overflowing", distance > 0.);
        }
    };
    let callback = {
        let measure = measure.clone();
        Closure::<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>::new(move |_, _| measure())
    };
    let observer = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()).ok()?;
    observer.observe(&viewport);
    observer.observe(&label);
    measure();
    if let Some(ready) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.fonts().ready().ok())
    {
        xcss_web::task::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(ready).await;
            measure();
        });
    }
    Some(SlidingObserver {
        observer,
        _callback: callback,
        active,
    })
}

enum Control {
    Input(web_sys::HtmlInputElement),
    Select(web_sys::HtmlSelectElement),
    Textarea(web_sys::HtmlTextAreaElement),
}
impl Control {
    fn from_event(event: &web_sys::Event) -> Option<Self> {
        let target = event.target()?;
        if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
            Some(Self::Input(input.clone()))
        } else if let Some(select) = target.dyn_ref::<web_sys::HtmlSelectElement>() {
            Some(Self::Select(select.clone()))
        } else {
            target
                .dyn_ref::<web_sys::HtmlTextAreaElement>()
                .cloned()
                .map(Self::Textarea)
        }
    }
    fn validity(&self) -> web_sys::ValidityState {
        match self {
            Self::Input(node) => node.validity(),
            Self::Select(node) => node.validity(),
            Self::Textarea(node) => node.validity(),
        }
    }
    fn set_custom_validity(&self, message: &str) {
        match self {
            Self::Input(node) => node.set_custom_validity(message),
            Self::Select(node) => node.set_custom_validity(message),
            Self::Textarea(node) => node.set_custom_validity(message),
        }
    }
}
pub fn install_document_policies() {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    // These four listeners intentionally live for the single mounted document,
    // exactly as the former JS document policies did; there is no per-page leak.
    gloo_events::EventListener::new_with_options(
        &doc,
        "invalid",
        gloo_events::EventListenerOptions::run_in_capture_phase(),
        move |event| {
            let Some(control) = Control::from_event(event) else {
                return;
            };
            let v = control.validity();
            if v.custom_error() {
                return;
            }
            let textarea = matches!(control, Control::Textarea(_));
            let message = if v.value_missing() {
                t("请填写此项。", "Please fill out this field.")
            } else if !textarea && v.type_mismatch() {
                t("请输入有效的格式。", "Please enter a valid value.")
            } else if v.too_long() {
                t("输入内容过长。", "The value is too long.")
            } else if v.too_short() {
                t("输入内容过短。", "The value is too short.")
            } else if !textarea && (v.range_overflow() || v.range_underflow()) {
                t(
                    "输入值超出允许范围。",
                    "The value is outside the allowed range.",
                )
            } else if !textarea && v.pattern_mismatch() {
                t(
                    "输入内容不符合要求的格式。",
                    "Please match the required format.",
                )
            } else if !textarea && (v.step_mismatch() || v.bad_input()) {
                t(
                    "请输入允许范围内的有效数字。",
                    "Please enter a valid number within the allowed range.",
                )
            } else {
                t("请检查输入内容。", "Please check this value.")
            };
            control.set_custom_validity(&message);
        },
    )
    .forget();
    for event in ["input", "change"] {
        gloo_events::EventListener::new_with_options(
            &doc,
            event,
            gloo_events::EventListenerOptions::run_in_capture_phase(),
            move |event| {
                if let Some(control) = Control::from_event(event) {
                    control.set_custom_validity("");
                }
            },
        )
        .forget();
    }
    gloo_events::EventListener::new_with_options(
        &doc,
        "click",
        gloo_events::EventListenerOptions::run_in_capture_phase(),
        move |event| {
            let Some(target) = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };
            let Ok(Some(link)) = target.closest("a[href]") else {
                return;
            };
            if link.has_attribute("download")
                || link
                    .get_attribute("href")
                    .is_some_and(|href| href.starts_with('#'))
            {
                return;
            }
            let Some(win) = web_sys::window() else { return };
            let Ok(current) = web_sys::Url::new(&win.location().href().unwrap_or_default()) else {
                return;
            };
            if !current.search_params().has("lang") {
                return;
            }
            let Some(href) = link.get_attribute("href") else {
                return;
            };
            let Ok(url) = web_sys::Url::new_with_base(&href, &current.href()) else {
                return;
            };
            if url.origin() == current.origin()
                && ["http:", "https:"].contains(&url.protocol().as_str())
            {
                url.search_params().set("lang", &xcss_web::i18n::locale());
                let _ = link.set_attribute(
                    "href",
                    &format!("{}{}{}", url.pathname(), url.search(), url.hash()),
                );
            }
        },
    )
    .forget();
}
