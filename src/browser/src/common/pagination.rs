//! Pagination controls preserve the caller's page and read-state policy.
use super::navigation::{MAX_PAGE, replace_browse};
use super::{Data, browse, num, page_number, t, tr};
use blog_interface::prelude::*;
use blog_interface::{ev, html};
use serde_json::Value;

pub(crate) trait PageMetadata {
    fn total(&self) -> i64;
    fn size(&self) -> i64;
}

impl PageMetadata for Value {
    fn total(&self) -> i64 {
        num(self, "total")
    }
    fn size(&self) -> i64 {
        num(self, "size")
    }
}

impl<T> PageMetadata for crate::models::Page<T> {
    fn total(&self) -> i64 {
        self.total
    }
    fn size(&self) -> i64 {
        self.size
    }
}

fn last_page(total: i64, size: i64) -> u64 {
    let size = size.max(1);
    (((total.max(0) + size - 1) / size).max(1) as u64).min(MAX_PAGE)
}

#[component]
pub(crate) fn Pager<T: PageMetadata + Clone + Send + Sync + 'static>(
    data: Data<T>,
    page: RwSignal<u64>,
    #[prop(default=t("分页","Pagination"))] label: String,
    #[prop(default = false)] public: bool,
    #[prop(default = false)] preserve: bool,
    #[prop(default=Signal::derive(||false))] locked: Signal<bool>,
) -> impl IntoView {
    Effect::new(move |_| {
        if let Some(v) = data.value.get() {
            let last = last_page(v.total(), v.size());
            if page.get_untracked() > last {
                page.set(last);
            }
        }
    });
    move || {
        data.value
            .get()
            .or_else(|| preserve.then(|| data.previous.get()).flatten())
            .filter(|v| v.total() > v.size() || page.get() > 1)
            .map(|v| {
                View::new(
                    blog_interface::html::nav()
                        .child((
                            blog_interface::html::button()
                                .child(t("上一页", "Previous page"))
                                .class(if public { "" } else { "blog-button" })
                                .r#type("button")
                                .disabled(move || {
                                    locked.get()
                                        || (!preserve && data.busy.get())
                                        || page.get() <= 1
                                })
                                .on(blog_interface::ev::click, move |_| {
                                    page.update(|p| *p = p.saturating_sub(1).max(1))
                                }),
                            blog_interface::html::span().child(tr(
                                "第 {0} 页 · 共 {1} 条",
                                "Page {0} · {1} items",
                                &[page.get().to_string(), v.total().to_string()],
                            )),
                            blog_interface::html::button()
                                .child(t("下一页", "Next page"))
                                .class(if public { "" } else { "blog-button" })
                                .r#type("button")
                                .disabled(move || {
                                    locked.get()
                                        || (!preserve && data.busy.get())
                                        || page.get() >= last_page(v.total(), v.size())
                                })
                                .on(blog_interface::ev::click, move |_| page.update(|p| *p += 1)),
                        ))
                        .class("pager")
                        .aria_label(label.clone()),
                )
            })
    }
}

#[component]
pub(crate) fn BrowsePager<T: PageMetadata + Clone + Send + Sync + 'static>(
    data: Data<T>,
    state: RwSignal<String>,
    size: i64,
    #[prop(default=t("分页","Pagination"))] label: String,
    #[prop(default = false)] always: bool,
    #[prop(default = false)] empty: bool,
) -> impl IntoView {
    Effect::new(move |_| {
        if let Some(value) = data.value.get() {
            let last = last_page(value.total(), value.size());
            if untrack(move || page_number(state)) > last {
                replace_browse(state, &[("page", (last > 1).then(|| last.to_string()))]);
            }
        }
    });
    move || {
        data.value
            .get()
            .filter(|v| {
                empty || v.total() > if always { 0 } else { size } || page_number(state) > 1
            })
            .map(|v| {
                let last = last_page(v.total(), v.size());
                View::new(
                    html::nav()
                        .child((
                            html::button()
                                .child(t("上一页", "Previous page"))
                                .r#type("button")
                                .disabled(move || data.busy.get() || page_number(state) <= 1)
                                .on(ev::click, move |_| {
                                    let page = page_number(state).saturating_sub(1).max(1);
                                    browse(
                                        state,
                                        &[("page", (page > 1).then(|| page.to_string()))],
                                    );
                                }),
                            html::span().child(tr(
                                "第 {0} 页",
                                "Page {0}",
                                &[page_number(state).to_string()],
                            )),
                            html::button()
                                .child(t("下一页", "Next page"))
                                .r#type("button")
                                .disabled(move || data.busy.get() || page_number(state) >= last)
                                .on(ev::click, move |_| {
                                    browse(
                                        state,
                                        &[("page", Some((page_number(state) + 1).to_string()))],
                                    )
                                }),
                        ))
                        .class("pager")
                        .aria_label(label.clone()),
                )
            })
    }
}
