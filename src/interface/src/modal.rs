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
            if let Err(error) = dialog.show_modal() {
                web_sys::console::warn_2(&"Unable to open the modal dialog".into(), &error);
                close.run(());
                return;
            }
            if let Some(element) = dialog
                .query_selector("[data-blog-initial-focus]")
                .expect("invariant: the initial focus selector is valid CSS")
                && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
                && let Err(error) = element.focus()
            {
                web_sys::console::warn_2(&"Unable to move browser focus".into(), &error);
            }
        }
    });
    on_cleanup(move || {
        if let Some(dialog) = dialog.get_untracked() {
            dialog.close();
        }
        if let Some(previous) = previous.as_ref()
            && previous.is_connected()
            && let Err(error) = previous.focus()
        {
            web_sys::console::warn_2(&"Unable to move browser focus".into(), &error);
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
            .expect("invariant: the media controls selector is valid CSS")
            .is_some()
    {
        return;
    }
    let nodes = dialog
        .query_selector_all("button,input,select,textarea,a[href],[tabindex]")
        .expect("invariant: the modal focus selector is valid CSS");
    let focusable: Vec<_> = (0..nodes.length())
        .map(|index| {
            nodes
                .item(index)
                .expect("invariant: the static focusable node list contains each index")
        })
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .filter(|element| {
            element.tab_index() >= 0
                && !element
                    .matches(":disabled,[hidden]")
                    .expect("invariant: the disabled and hidden selectors are valid CSS")
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
            if let Err(error) = last.focus() {
                web_sys::console::warn_2(&"Unable to move browser focus".into(), &error);
            }
        } else if !event.shift_key() && active.as_ref() == Some(last.as_ref()) {
            event.prevent_default();
            if let Err(error) = first.focus() {
                web_sys::console::warn_2(&"Unable to move browser focus".into(), &error);
            }
        }
    } else {
        event.prevent_default();
        if let Err(error) = dialog.focus() {
            web_sys::console::warn_2(&"Unable to move browser focus".into(), &error);
        }
    }
}
