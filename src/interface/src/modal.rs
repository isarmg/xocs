//! Native modal semantics shared by product dialogs and media viewers.
use blog_dom::thread_bound::SendWrapper;
use blog_interface::{html, prelude::*};
use wasm_bindgen::JsCast;

/// A modal surface whose current reactive owner controls its lifetime.
///
/// Content and appearance belong to the caller. Closing invokes the caller's
/// state transition; teardown closes the native dialog and restores its opener.
#[component]
pub fn ModalSurface(
    label: String,
    class: &'static str,
    close: Callback<()>,
    children: ChildrenFn,
    #[prop(default = false)] dismiss_on_backdrop: bool,
) -> impl IntoView {
    let dialog = NodeRef::<html::Dialog>::new();
    let previous = SendWrapper::new(
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok()),
    );
    Effect::new(move |_| {
        if let Some(dialog) = dialog.get() {
            let _ = dialog.show_modal();
            if let Ok(Some(element)) = dialog.query_selector("[data-blog-initial-focus]")
                && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
            {
                let _ = element.focus();
            }
        }
    });
    on_cleanup(move || {
        if let Some(dialog) = dialog.get_untracked() {
            dialog.close();
        }
        if let Some(previous) = previous.as_ref()
            && previous.is_connected()
        {
            let _ = previous.focus();
        }
    });
    html::dialog()
        .class(class)
        .aria_label(label)
        .tabindex("-1")
        .node_ref(dialog)
        .child(children())
        .on(
            blog_interface::ev::Custom::new("cancel"),
            move |event: web_sys::Event| {
                event.prevent_default();
                close.run(());
            },
        )
        .on(blog_interface::ev::mousedown, move |event| {
            if !dismiss_on_backdrop || event.target() != event.current_target() {
                return;
            }
            let Some(dialog) = dialog.get_untracked() else {
                return;
            };
            let bounds = dialog.get_bounding_client_rect();
            let x = f64::from(event.client_x());
            let y = f64::from(event.client_y());
            if x < bounds.left() || x > bounds.right() || y < bounds.top() || y > bounds.bottom() {
                close.run(());
            }
        })
        .on(blog_interface::ev::keydown, move |event| {
            if let Some(dialog) = dialog.get_untracked() {
                contain_focus(&dialog, event);
            }
        })
}

fn contain_focus(dialog: &web_sys::HtmlDialogElement, event: web_sys::KeyboardEvent) {
    if event.key() != "Tab"
        || dialog
            .query_selector("video[controls],audio[controls]")
            .ok()
            .flatten()
            .is_some()
    {
        return;
    }
    let Ok(nodes) = dialog.query_selector_all("button,input,select,textarea,a[href],[tabindex]")
    else {
        return;
    };
    let focusable: Vec<_> = (0..nodes.length())
        .filter_map(|index| nodes.item(index))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .filter(|element| {
            element.tab_index() >= 0
                && !element.matches(":disabled,[hidden]").unwrap_or(true)
                && element.get_client_rects().length() > 0
        })
        .collect();
    let active = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element());
    if let (Some(first), Some(last)) = (focusable.first(), focusable.last()) {
        if event.shift_key()
            && (active.as_ref() == Some(first.as_ref()) || active.as_ref() == Some(dialog.as_ref()))
        {
            event.prevent_default();
            let _ = last.focus();
        } else if !event.shift_key() && active.as_ref() == Some(last.as_ref()) {
            event.prevent_default();
            let _ = first.focus();
        }
    } else {
        event.prevent_default();
        let _ = dialog.focus();
    }
}
