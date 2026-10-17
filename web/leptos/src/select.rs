//! Popup selector ported from xcss Select.tsx: native validation and a painted
//! top-layer listbox share one controlled value; all selection state is Rust.
use crate::common::t;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
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
    let trigger = NodeRef::<leptos::html::Button>::new();
    let native = NodeRef::<leptos::html::Select>::new();
    let panel = NodeRef::<leptos::html::Span>::new();
    let open = RwSignal::new(false);
    let active = RwSignal::new(0usize);
    // Pointer hover only paints the active option. It must not move the popup
    // between pointerdown and click (Firefox otherwise retargets the click).
    let reveal = RwSignal::new(0u64);
    let message = RwSignal::new(String::new());
    let typed = StoredValue::new((String::new(), 0f64));
    let id = StoredValue::new(format!(
        "xocs-select-{}",
        web_sys::window().unwrap().crypto().unwrap().random_uuid()
    ));
    let close = Callback::new(move |()| {
        if let Some(p) = panel.get() {
            let _ = p.hide_popover();
        }
        open.set(false);
        typed.set_value((String::new(), 0.));
    });
    let position = Callback::new(move |()| {
        let (Some(anchor), Some(popup)) = (trigger.get(), panel.get()) else {
            return;
        };
        let rect = anchor.get_bounding_client_rect();
        let win = web_sys::window().unwrap();
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
        let _ = style.set_property("width", &format!("{width}px"));
        let _ = style.set_property("left", &format!("{left}px"));
        let _ = style.set_property(
            "max-height",
            &format!("{}px", 320f64.min(if upwards { above } else { below })),
        );
        let height = popup.get_bounding_client_rect().height();
        let top = if upwards {
            (rect.top() - 4. - height).max(8.)
        } else {
            rect.bottom() + 4.
        };
        let _ = style.set_property("top", &format!("{top}px"));
    });
    let show = Callback::new(move |index: Option<usize>| {
        let Some(field) = native.get() else { return };
        if field.matches(":disabled").unwrap_or(true) {
            return;
        }
        let opts = options.get_untracked();
        if opts.is_empty() {
            return;
        }
        let selected = opts
            .iter()
            .position(|(v, _)| *v == value.get_untracked())
            .unwrap_or(0);
        active.set(index.unwrap_or(selected).min(opts.len() - 1));
        if let Some(panel) = panel.get() {
            let _ = panel.show_popover();
        }
        position.run(());
        open.set(true);
        reveal.update(|n| *n += 1);
    });
    let commit = Callback::new(move |index: usize| {
        let Some(field) = native.get() else { return };
        if field.matches(":disabled").unwrap_or(true) {
            close.run(());
            return;
        }
        if let Some((next, _)) = options.get_untracked().get(index) {
            field.set_custom_validity("");
            message.set(String::new());
            if *next != value.get_untracked() {
                field.set_value(if required && next == "0" { "" } else { next });
                change.run(next.clone());
            }
        }
        close.run(());
    });
    Effect::new(move |_| {
        value.get();
        options.get();
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
            && let Ok(Some(item)) = panel.query_selector(&format!(
                "[data-option-index=\"{}\"]",
                active.get_untracked()
            ))
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
    let outside = move |event: &web_sys::Event| {
        if !open.get_untracked() {
            return;
        }
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
    let doc = web_sys::window().unwrap().document().unwrap();
    let win = web_sys::window().unwrap();
    let pointer = SendWrapper::new(gloo_events::EventListener::new_with_options(
        &doc,
        "pointerdown",
        gloo_events::EventListenerOptions::run_in_capture_phase(),
        outside,
    ));
    let focus = SendWrapper::new(gloo_events::EventListener::new(&doc, "focusin", outside));
    let resize = SendWrapper::new(gloo_events::EventListener::new(&win, "resize", move |_| {
        if open.get_untracked() {
            position.run(())
        }
    }));
    let scroll = SendWrapper::new(gloo_events::EventListener::new_with_options(
        &doc,
        "scroll",
        gloo_events::EventListenerOptions::run_in_capture_phase(),
        move |event| {
            // Scrolling options cannot change the anchor's viewport position.
            if event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
                .is_some_and(|target| panel.get().is_some_and(|p| p.contains(Some(&target))))
            {
                return;
            }
            if open.get_untracked() {
                position.run(())
            }
        },
    ));
    let blur = SendWrapper::new(gloo_events::EventListener::new(&win, "blur", move |_| {
        close.run(())
    }));
    on_cleanup(move || {
        drop(pointer);
        drop(focus);
        drop(resize);
        drop(scroll);
        drop(blur);
    });
    Effect::new(move |_| {
        if let Some(field) = native.get() {
            let callback =
                Closure::<dyn FnMut(js_sys::Array, web_sys::MutationObserver)>::new(move |_, _| {
                    if native
                        .get()
                        .is_some_and(|f| f.matches(":disabled").unwrap_or(false))
                    {
                        close.run(())
                    }
                });
            let observer =
                web_sys::MutationObserver::new(callback.as_ref().unchecked_ref()).unwrap();
            let opts = web_sys::MutationObserverInit::new();
            opts.set_attributes(true);
            let mut ancestor = field.parent_element();
            while let Some(parent) = ancestor {
                if parent.tag_name() == "FIELDSET" {
                    let _ = observer.observe_with_options(&parent, &opts);
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
            let listener = SendWrapper::new(gloo_events::EventListener::new(
                &form,
                "reset",
                move |event| {
                    let event = SendWrapper::new(event.clone());
                    let field = SendWrapper::new(field.clone());
                    let selected = value.get_untracked();
                    gloo_timers::callback::Timeout::new(0, move || {
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
        if native
            .get()
            .is_none_or(|f| f.matches(":disabled").unwrap_or(true))
        {
            return;
        }
        let opts = options.get_untracked();
        if opts.is_empty() {
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
                    Some(opts.len() - 1)
                } else {
                    None
                });
                return;
            }
            let next = match key.as_str() {
                "Home" => 0,
                "End" => opts.len() - 1,
                "ArrowDown" => (active.get_untracked() + 1).min(opts.len() - 1),
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
                text.chars().next().unwrap().to_string()
            } else {
                text
            };
            let selected = opts
                .iter()
                .position(|(v, _)| *v == value.get_untracked())
                .unwrap_or(0);
            let from = if open.get_untracked() {
                active.get_untracked()
            } else {
                selected
            };
            let offset = if term.chars().count() == 1 { 1 } else { 0 };
            let found = (0..opts.len())
                .map(|n| (from + offset + n) % opts.len())
                .find(|i| opts[*i].1.to_lowercase().starts_with(&term));
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
        leptos::html::span()
            .child((
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::button()
                        .child((
                            IntoMaybeErased::into_maybe_erased(
                                leptos::html::span()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                move || {
                                                    options
                                                        .get()
                                                        .into_iter()
                                                        .find(|(v, _)| *v == value.get())
                                                        .map(|(_, label)| label)
                                                        .unwrap_or_else(|| "\u{a0}".into())
                                                }
                                            })
                                        },
                                    )
                                    .class("xcss-select-value"),
                            ),
                            IntoMaybeErased::into_maybe_erased(
                                leptos::svg::svg()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            leptos::svg::path().attr("d", "m9 6 6 6-6 6")
                                        },
                                    )
                                    .class("xcss-select-arrow")
                                    .attr("aria-hidden", "true")
                                    .attr("width", "16")
                                    .attr("height", "16")
                                    .attr("viewBox", "0 0 24 24")
                                    .attr("fill", "none")
                                    .attr("stroke", "currentColor")
                                    .attr("stroke-width", "1.8"),
                            ),
                        ))
                        .class("xcss-input xcss-select-trigger")
                        .node_ref(trigger)
                        .id(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            format!("{}-select", id.get_value()),
                        ))
                        .r#type("button")
                        .role("combobox")
                        .aria_haspopup("listbox")
                        .aria_expanded(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || open.get().to_string(),
                        ))
                        .aria_controls(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            format!("{}-options", id.get_value()),
                        ))
                        .aria_activedescendant(
                            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                move || {
                                    open.get().then(|| {
                                        format!("{}-options-{}", id.get_value(), active.get())
                                    })
                                },
                            ),
                        )
                        .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || label.get(),
                        ))
                        .aria_describedby(
                            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                move || {
                                    (!message.get().is_empty())
                                        .then(|| format!("{}-error", id.get_value()))
                                },
                            ),
                        )
                        .aria_required(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            required.to_string(),
                        ))
                        .aria_invalid(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || (!message.get().is_empty()).to_string(),
                        ))
                        .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || disabled.get(),
                        ))
                        .on(leptos::ev::click, move |ev| {
                            ev.prevent_default();
                            if open.get_untracked() {
                                close.run(())
                            } else {
                                show.run(None)
                            }
                        })
                        .on(leptos::ev::keydown, keydown),
                ),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::select()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    move || {
                                        options
                                            .get()
                                            .into_iter()
                                            .map(|(v, l)| {
                                                View::new(
    leptos::html::option()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ l }) },
        )
        .value(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                if required && v == "0" { String::new() } else { v },
            ),
        ),
)
                                            })
                                            .collect_view()
                                    }
                                })
                            },
                        )
                        .class("xcss-select-native")
                        .node_ref(native)
                        .id(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            format!("{}-native", id.get_value()),
                        ))
                        .aria_hidden("true")
                        .tabindex("-1")
                        .required(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            required,
                        ))
                        .disabled(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || disabled.get(),
                        ))
                        .prop("value", move || {
                            let v = value.get();
                            if required && v == "0" {
                                String::new()
                            } else {
                                v
                            }
                        })
                        .on(leptos::ev::change, move |ev| {
                            message.set(String::new());
                            let field = event_target::<web_sys::HtmlSelectElement>(&ev);
                            field.set_custom_validity("");
                            change.run(field.value());
                        })
                        .on(leptos::ev::invalid, move |ev: web_sys::Event| {
                            ev.prevent_default();
                            let field = event_target::<web_sys::HtmlSelectElement>(&ev);
                            let m = t("请填写此项。", "Please fill out this field.");
                            field.set_custom_validity(&m);
                            message.set(m);
                            close.run(());
                            let field = SendWrapper::new(field);
                            gloo_timers::callback::Timeout::new(0, move || {
                                if !field.is_connected() {
                                    return;
                                }
                                let first = field.form().and_then(|f| {
                                    f.query_selector(
                                        "input:invalid,select:invalid,textarea:invalid",
                                    )
                                    .ok()
                                    .flatten()
                                });
                                if first.is_none_or(|e| {
                                    e == (*field).clone().unchecked_into::<web_sys::Element>()
                                }) && let Some(trigger) = trigger.get()
                                {
                                    let _ = trigger.focus();
                                }
                            })
                            .forget();
                        }),
                ),
                IntoMaybeErased::into_maybe_erased(
                    leptos::html::span()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    move || {
                                        options
                                            .get()
                                            .into_iter()
                                            .enumerate()
                                            .map(|(i, (v, l))| {
                                                View::new(leptos::html::span().child(
                                                    #[allow(unused_braces)]
                                                    {
                                                        leptos::html::span()
                    .child(
                        (
                                IntoMaybeErased::into_maybe_erased(leptos::html::span()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ l }) })),
                                IntoMaybeErased::into_maybe_erased(leptos::html::span()
                                .child(#[allow(unused_braces)] {
                                IntoRender::into_render({ { let v = v
                                .clone(); move | | if value.get() == v { "✓" } else { "" }
                                } }) }).aria_hidden("true"))
                            ,),
                    )
                    .class("xcss-select-option")
                    .id(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            format!("{}-options-{i}", id.get_value()),
                        ),
                    )
                    .role("option")
                    .aria_selected(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value({
                            let v = v.clone();
                            move || (value.get() == v).to_string()
                        }),
                    )
                    .attr(
                        "data-option-index",
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            i,
                        ),
                    )
                    .attr(
                        "data-active",
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || {
                                (active.get() == i).then_some("true")
                            },
                        ),
                    )
                    .on(
                        leptos::ev::pointermove,
                        move |_| active.set(i),
                    )
                    .on(
                        leptos::ev::click,
                        move |ev| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            commit.run(i);
                        },
                    )
                                                    },
                                                ))
                                            })
                                            .collect_view()
                                    }
                                })
                            },
                        )
                        .class("xcss-select-panel")
                        .node_ref(panel)
                        .id(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            format!("{}-options", id.get_value()),
                        ))
                        .popover("manual")
                        .role("listbox")
                        .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            move || label.get(),
                        ))
                        .attr(
                            "data-xcss-select-open",
                            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                move || open.get().then_some("true"),
                            ),
                        )
                        .on(leptos::ev::pointerdown, move |ev| {
                            if ev
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                .is_some_and(|t| {
                                    t.closest("[role=option]").ok().flatten().is_some()
                                })
                            {
                                ev.prevent_default();
                            }
                        }),
                ),
                IntoMaybeErased::into_maybe_erased(IntoRender::into_render({
                    move || {
                        (!message.get().is_empty()).then(|| {
                            View::new(
                                leptos::html::span()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({ message.get() })
                                        },
                                    )
                                    .class("xcss-select-error")
                                    .id(
                                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                            format!("{}-error", id.get_value()),
                                        ),
                                    )
                                    .role("alert"),
                            )
                        })
                    }
                })),
            ))
            .class("xcss-select"),
    )
}
