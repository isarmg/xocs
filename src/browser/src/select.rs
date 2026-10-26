//! Popup selector ported from xcss Select.tsx: native validation and a painted
//! top-layer listbox share one controlled value; all selection state is Rust.
use crate::common::t;
use blog_interface::prelude::*;
use blog_interface::thread_bound::SendWrapper;
use wasm_bindgen::{JsCast, closure::Closure};
#[component]
pub fn Select(
    label: Signal<String>,
    value: Signal<String>,
    options: Signal<Vec<(String, String)>>,
    change: Callback<String>,
    #[prop(default = false)] required: bool,
    #[prop(default=Signal::derive(||false))] disabled: Signal<bool>,
) -> impl IntoView {
    let options = Memo::new(move |_| options.get());
    let trigger = NodeRef::<blog_interface::html::Button>::new();
    let native = NodeRef::<blog_interface::html::Select>::new();
    let panel = NodeRef::<blog_interface::html::Span>::new();
    let open = RwSignal::new(false);
    let active = RwSignal::new(0usize);
    // Pointer hover only paints the active option. It must not move the popup
    // between pointerdown and click (Firefox otherwise retargets the click).
    let reveal = RwSignal::new(0u64);
    let message = RwSignal::new(String::new());
    let typed = StoredValue::new((String::new(), 0f64));
    let id = StoredValue::new(format!(
        "blog-select-{}",
        web_sys::window()
            .expect("invariant: the application is mounted in a browser window")
            .crypto()
            .expect("invariant: the application window exposes browser crypto")
            .random_uuid()
    ));
    let close = Callback::new(move |()| {
        if let Some(p) = panel.get()
            && let Err(error) = p.hide_popover()
        {
            web_sys::console::debug_2(&"Selector popup was already detached".into(), &error);
        }
        open.set(false);
        typed.set_value((String::new(), 0.));
    });
    let position = Callback::new(move |()| {
        let (Some(anchor), Some(popup)) = (trigger.get(), panel.get()) else {
            return;
        };
        let rect = anchor.get_bounding_client_rect();
        let win =
            web_sys::window().expect("invariant: the application is mounted in a browser window");
        let width = rect.width().min(
            win.inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.)
                - 16.,
        );
        let left = rect
            .left()
            .min(
                win.inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.)
                    - width
                    - 8.,
            )
            .max(8.);
        let below = (win
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.)
            - rect.bottom()
            - 12.)
            .max(0.);
        let above = (rect.top() - 12.).max(0.);
        let upwards = below < 180f64.min(popup.scroll_height() as f64) && above > below;
        let style = popup.unchecked_ref::<web_sys::HtmlElement>().style();
        style
            .set_property("width", &format!("{width}px"))
            .expect("invariant: the selector popup owns a writable style declaration");
        style
            .set_property("left", &format!("{left}px"))
            .expect("invariant: the selector popup owns a writable style declaration");
        style
            .set_property(
                "max-height",
                &format!("{}px", 320f64.min(if upwards { above } else { below })),
            )
            .expect("invariant: the selector popup owns a writable style declaration");
        let height = popup.get_bounding_client_rect().height();
        let top = if upwards {
            (rect.top() - 4. - height).max(8.)
        } else {
            rect.bottom() + 4.
        };
        style
            .set_property("top", &format!("{top}px"))
            .expect("invariant: the selector popup owns a writable style declaration");
    });
    let show = Callback::new(move |index: Option<usize>| {
        let Some(field) = native.get() else { return };
        if field
            .matches(":disabled")
            .expect("invariant: :disabled is a valid CSS selector")
        {
            return;
        }
        let selected = value.get_untracked();
        let Some(index) = options.with_untracked(|opts| {
            (!opts.is_empty()).then(|| {
                let selected = opts.iter().position(|(v, _)| *v == selected).unwrap_or(0);
                index.unwrap_or(selected).min(opts.len() - 1)
            })
        }) else {
            return;
        };
        active.set(index);
        let Some(panel) = panel.get() else { return };
        if let Err(error) = panel.show_popover() {
            web_sys::console::warn_2(&"Unable to open the selector popup".into(), &error);
            return;
        }
        position.run(());
        open.set(true);
        reveal.update(|n| *n += 1);
    });
    let commit = Callback::new(move |index: usize| {
        let Some(field) = native.get() else { return };
        if field
            .matches(":disabled")
            .expect("invariant: :disabled is a valid CSS selector")
        {
            close.run(());
            return;
        }
        let next = options.with_untracked(|opts| opts.get(index).map(|(next, _)| next.clone()));
        if let Some(next) = next {
            field.set_custom_validity("");
            message.set(String::new());
            if next != value.get_untracked() {
                field.set_value(if required && next == "0" { "" } else { &next });
                change.run(next);
            }
        }
        close.run(());
    });
    Effect::new(move |_| {
        value.track();
        options.track();
        if let Some(field) = native.get() {
            let selected = value.get_untracked();
            field.set_value(if required && selected == "0" {
                ""
            } else {
                &selected
            });
            field.set_custom_validity("");
        }
        message.set(String::new());
        if disabled.get() && open.get_untracked() {
            close.run(());
        }
        if open.get() {
            position.run(());
        }
    });
    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        reveal.get();
        if let Some(panel) = panel.get()
            && let Some(item) = panel
                .query_selector(&format!(
                    "[data-option-index=\"{}\"]",
                    active.get_untracked()
                ))
                .expect("invariant: an integer option index forms a valid CSS selector")
        {
            let rect = item.get_bounding_client_rect();
            let outer = panel.get_bounding_client_rect();
            // Scroll only the listbox, never the document/ancestor fieldset.
            // scrollIntoView may scroll every ancestor, moving the trigger and
            // causing a placement flip while a pointer selection is underway.
            let delta = if rect.top() < outer.top() {
                rect.top() - outer.top()
            } else if rect.bottom() > outer.bottom() {
                rect.bottom() - outer.bottom()
            } else {
                0.
            };
            panel.set_scroll_top(panel.scroll_top() + delta.round() as i32);
        }
    });
    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        let outside = move |event: &web_sys::Event| {
            let target = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Node>().ok());
            let inside = target.as_ref().is_some_and(|t| {
                trigger.get().is_some_and(|e| e.contains(Some(t)))
                    || panel.get().is_some_and(|e| e.contains(Some(t)))
            });
            if !inside {
                close.run(());
            }
        };
        let win =
            web_sys::window().expect("invariant: the application is mounted in a browser window");
        let doc = win
            .document()
            .expect("invariant: the application window owns its document");
        let pointer = SendWrapper::new(blog_interface::events::EventListener::new_with_options(
            &doc,
            "pointerdown",
            blog_interface::events::EventListenerOptions::run_in_capture_phase(),
            outside,
        ));
        let focus = SendWrapper::new(blog_interface::events::EventListener::new(
            &doc, "focusin", outside,
        ));
        let resize = SendWrapper::new(blog_interface::events::EventListener::new(
            &win,
            "resize",
            move |_| position.run(()),
        ));
        let scroll = SendWrapper::new(blog_interface::events::EventListener::new_with_options(
            &doc,
            "scroll",
            blog_interface::events::EventListenerOptions::run_in_capture_phase(),
            move |event| {
                // Scrolling options cannot change the anchor's viewport position.
                if event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
                    .is_some_and(|target| panel.get().is_some_and(|p| p.contains(Some(&target))))
                {
                    return;
                }
                position.run(())
            },
        ));
        let blur = SendWrapper::new(blog_interface::events::EventListener::new(
            &win,
            "blur",
            move |_| close.run(()),
        ));
        on_cleanup(move || {
            drop(pointer);
            drop(focus);
            drop(resize);
            drop(scroll);
            drop(blur);
        });
    });
    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        if let Some(field) = native.get() {
            let callback =
                Closure::<dyn FnMut(js_sys::Array, web_sys::MutationObserver)>::new(move |_, _| {
                    if native.get().is_some_and(|f| {
                        f.matches(":disabled")
                            .expect("invariant: :disabled is a valid CSS selector")
                    }) {
                        close.run(())
                    }
                });
            let observer = web_sys::MutationObserver::new(callback.as_ref().unchecked_ref())
                .expect("invariant: a mutation observer callback is callable");
            let opts = web_sys::MutationObserverInit::new();
            opts.set_attributes(true);
            let mut ancestor = field.parent_element();
            while let Some(parent) = ancestor {
                if parent.tag_name() == "FIELDSET" {
                    observer.observe_with_options(&parent, &opts).expect(
                        "invariant: the selector observes ancestor attributes with valid options",
                    );
                }
                ancestor = parent.parent_element();
            }
            let callback = SendWrapper::new(callback);
            let observer = SendWrapper::new(observer);
            on_cleanup(move || {
                observer.disconnect();
                drop(callback);
            });
        }
    });
    Effect::new(move |_| {
        if let Some(field) = native.get()
            && let Some(form) = field.form()
        {
            let listener = SendWrapper::new(blog_interface::events::EventListener::new(
                &form,
                "reset",
                move |event| {
                    let event = SendWrapper::new(event.clone());
                    let field = SendWrapper::new(field.clone());
                    let selected = value.get_untracked();
                    blog_interface::timers::callback::Timeout::new(0, move || {
                        if !event.default_prevented() && field.is_connected() {
                            field.set_custom_validity("");
                            field.set_value(if required && selected == "0" {
                                ""
                            } else {
                                &selected
                            });
                            message.set(String::new());
                            close.run(());
                        }
                    })
                    .forget();
                },
            ));
            on_cleanup(move || drop(listener));
        }
    });
    let keydown = move |event: web_sys::KeyboardEvent| {
        if native.get().is_none_or(|f| {
            f.matches(":disabled")
                .expect("invariant: :disabled is a valid CSS selector")
        }) {
            return;
        }
        let count = options.with_untracked(Vec::len);
        if count == 0 {
            return;
        }
        let key = event.key();
        if ["ArrowDown", "ArrowUp", "Home", "End"].contains(&key.as_str()) {
            event.prevent_default();
            event.stop_propagation();
            if !open.get_untracked() {
                show.run(if key == "Home" {
                    Some(0)
                } else if key == "End" {
                    Some(count - 1)
                } else {
                    None
                });
                return;
            }
            let next = match key.as_str() {
                "Home" => 0,
                "End" => count - 1,
                "ArrowDown" => (active.get_untracked() + 1).min(count - 1),
                _ => active.get_untracked().saturating_sub(1),
            };
            active.set(next);
            reveal.update(|n| *n += 1);
            return;
        }
        if key == "Enter" || key == " " {
            event.prevent_default();
            event.stop_propagation();
            if open.get_untracked() {
                commit.run(active.get_untracked())
            } else {
                show.run(None)
            }
            return;
        }
        if key == "Escape" && open.get_untracked() {
            event.prevent_default();
            event.stop_propagation();
            close.run(());
            return;
        }
        if key == "Tab" {
            if open.get_untracked() {
                commit.run(active.get_untracked())
            }
            return;
        }
        if !event.ctrl_key() && !event.alt_key() && !event.meta_key() && key.chars().count() == 1 {
            event.prevent_default();
            let now = js_sys::Date::now();
            let (previous, at) = typed.get_value();
            let text = format!(
                "{}{}",
                if now - at < 750. {
                    previous
                } else {
                    String::new()
                },
                key.to_lowercase()
            );
            typed.set_value((text.clone(), now));
            let repeated = text
                .chars()
                .next()
                .is_some_and(|first| text.chars().all(|c| c == first));
            let term = if repeated {
                text.chars()
                    .next()
                    .expect("invariant: the typeahead text contains the current character")
                    .to_string()
            } else {
                text
            };
            let selected = value.get_untracked();
            let selected = options
                .with_untracked(|opts| opts.iter().position(|(v, _)| *v == selected).unwrap_or(0));
            let from = if open.get_untracked() {
                active.get_untracked()
            } else {
                selected
            };
            let offset = if term.chars().count() == 1 { 1 } else { 0 };
            let found = options.with_untracked(|opts| {
                (0..count)
                    .map(|n| (from + offset + n) % count)
                    .find(|i| opts[*i].1.to_lowercase().starts_with(&term))
            });
            if open.get_untracked() {
                if let Some(i) = found {
                    active.set(i);
                    reveal.update(|n| *n += 1);
                }
            } else {
                show.run(found)
            }
        }
    };
    View::new(
        blog_interface::html::span()
            .child((
                blog_interface::html::button()
                    .child((
                        blog_interface::html::span()
                            .child(move || {
                                let selected = value.get();
                                options.with(|opts| {
                                    opts.iter()
                                        .find(|(v, _)| *v == selected)
                                        .map(|(_, label)| label.clone())
                                        .unwrap_or_else(|| "\u{a0}".into())
                                })
                            })
                            .class("blog-select-value"),
                        blog_interface::svg::svg()
                            .child(blog_interface::svg::path().attr("d", "m9 6 6 6-6 6"))
                            .class("blog-select-arrow")
                            .attr("aria-hidden", "true")
                            .attr("width", "16")
                            .attr("height", "16")
                            .attr("viewBox", "0 0 24 24")
                            .attr("fill", "none")
                            .attr("stroke", "currentColor")
                            .attr("stroke-width", "1.8"),
                    ))
                    .class("blog-input blog-input--compact blog-select-trigger")
                    .node_ref(trigger)
                    .id(format!("{}-select", id.get_value()))
                    .r#type("button")
                    .role("combobox")
                    .aria_haspopup("listbox")
                    .aria_expanded(move || open.get().to_string())
                    .aria_controls(format!("{}-options", id.get_value()))
                    .aria_activedescendant(move || {
                        open.get()
                            .then(|| format!("{}-options-{}", id.get_value(), active.get()))
                    })
                    .aria_label(move || label.get())
                    .aria_describedby(move || {
                        (!message.get().is_empty()).then(|| format!("{}-error", id.get_value()))
                    })
                    .aria_required(required.to_string())
                    .aria_invalid(move || (!message.get().is_empty()).to_string())
                    .disabled(move || disabled.get())
                    .on(blog_interface::ev::click, move |ev| {
                        ev.prevent_default();
                        if open.get_untracked() {
                            close.run(())
                        } else {
                            show.run(None)
                        }
                    })
                    .on(blog_interface::ev::keydown, keydown),
                blog_interface::html::select()
                    .child(move || {
                        options
                            .get()
                            .into_iter()
                            .map(|(v, l)| {
                                View::new(blog_interface::html::option().child(l).value(
                                    if required && v == "0" {
                                        String::new()
                                    } else {
                                        v
                                    },
                                ))
                            })
                            .collect_view()
                    })
                    .class("blog-select-native")
                    .node_ref(native)
                    .id(format!("{}-native", id.get_value()))
                    .aria_hidden("true")
                    .tabindex("-1")
                    .required(required)
                    .disabled(move || disabled.get())
                    .prop("value", move || {
                        let v = value.get();
                        if required && v == "0" {
                            String::new()
                        } else {
                            v
                        }
                    })
                    .on(blog_interface::ev::change, move |ev| {
                        message.set(String::new());
                        let field = event_target::<web_sys::HtmlSelectElement>(&ev);
                        field.set_custom_validity("");
                        change.run(field.value());
                    })
                    .on(blog_interface::ev::invalid, move |ev: web_sys::Event| {
                        ev.prevent_default();
                        let field = event_target::<web_sys::HtmlSelectElement>(&ev);
                        let m = t("请填写此项。", "Please fill out this field.");
                        field.set_custom_validity(&m);
                        message.set(m);
                        close.run(());
                        let field = SendWrapper::new(field);
                        blog_interface::timers::callback::Timeout::new(0, move || {
                            if !field.is_connected() {
                                return;
                            }
                            let first = field.form().and_then(|f| {
                                f.query_selector("input:invalid,select:invalid,textarea:invalid")
                                    .expect(
                                        "invariant: the invalid form control selector is valid CSS",
                                    )
                            });
                            if first.is_none_or(|e| {
                                e == (*field).clone().unchecked_into::<web_sys::Element>()
                            }) && let Some(trigger) = trigger.get()
                                && let Err(error) = trigger.focus()
                            {
                                web_sys::console::warn_2(
                                    &"Unable to move browser focus".into(),
                                    &error,
                                );
                            }
                        })
                        .forget();
                    }),
                blog_interface::html::span()
                    .child(move || {
                        options
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, (v, l))| {
                                View::new(
                                    blog_interface::html::span().child(
                                        blog_interface::html::span()
                                            .child((
                                                blog_interface::html::span().child(l),
                                                blog_interface::html::span()
                                                    .child({
                                                        let v = v.clone();
                                                        move || {
                                                            if value.get() == v {
                                                                "✓"
                                                            } else {
                                                                ""
                                                            }
                                                        }
                                                    })
                                                    .aria_hidden("true"),
                                            ))
                                            .class("blog-select-option blog-row blog-row--option")
                                            .id(format!("{}-options-{i}", id.get_value()))
                                            .role("option")
                                            .aria_selected({
                                                move || (value.get() == v).to_string()
                                            })
                                            .attr("data-option-index", i)
                                            .attr("data-active", move || {
                                                (active.get() == i).then_some("true")
                                            })
                                            .on(blog_interface::ev::pointermove, move |_| {
                                                active.set(i)
                                            })
                                            .on(blog_interface::ev::click, move |ev| {
                                                ev.prevent_default();
                                                ev.stop_propagation();
                                                commit.run(i);
                                            }),
                                    ),
                                )
                            })
                            .collect_view()
                    })
                    .class("blog-select-panel blog-frame")
                    .node_ref(panel)
                    .id(format!("{}-options", id.get_value()))
                    .popover("manual")
                    .role("listbox")
                    .aria_label(move || label.get())
                    .attr("data-blog-select-open", move || {
                        open.get().then_some("true")
                    })
                    .on(blog_interface::ev::pointerdown, move |ev| {
                        if ev
                            .target()
                            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                            .is_some_and(|t| {
                                t.closest("[role=option]")
                                    .expect("invariant: the option role selector is valid CSS")
                                    .is_some()
                            })
                        {
                            ev.prevent_default();
                        }
                    }),
                move || {
                    (!message.get().is_empty()).then(|| {
                        View::new(
                            blog_interface::html::span()
                                .child(message.get())
                                .class("blog-select-error")
                                .id(format!("{}-error", id.get_value()))
                                .role("alert"),
                        )
                    })
                },
            ))
            .class("blog-select"),
    )
}
