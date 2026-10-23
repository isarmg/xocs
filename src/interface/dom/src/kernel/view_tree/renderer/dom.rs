#![allow(missing_docs)]

//! Direct browser DOM operations for the CSR view tree.

use super::{CastFrom, RemoveEventHandler};
use crate::view_tree::{
    dom::{document, window},
    ok_or_debug, or_debug,
    view::Mountable,
};
use std::cell::{Cell, LazyCell, RefCell};
use wasm_bindgen::{intern, prelude::Closure, JsCast, JsValue};
use web_sys::{AddEventListenerOptions, Comment};

/// Browser DOM operations backed by `web-sys`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Dom;

pub type Node = web_sys::Node;
pub type Text = web_sys::Text;
pub type Element = web_sys::Element;
pub type Placeholder = web_sys::Comment;
pub type Event = wasm_bindgen::JsValue;
pub type ClassList = web_sys::DomTokenList;
pub type CssStyleDeclaration = web_sys::CssStyleDeclaration;

/// A microtask is a short function which will run after the current task has
/// completed its work and when there is no other code waiting to be run before
/// control of the execution context is returned to the browser's event loop.
///
/// Microtasks are especially useful for libraries and frameworks that need
/// to perform final cleanup or other just-before-rendering tasks.
///
/// [MDN queueMicrotask](https://developer.mozilla.org/en-US/docs/Web/API/queueMicrotask)
pub fn queue_microtask(task: impl FnOnce() + 'static) {
    use js_sys::{Function, Reflect};

    let task = Closure::once_into_js(task);
    let window = window();
    let queue_microtask = Reflect::get(&window, &JsValue::from_str("queueMicrotask"))
        .expect("queueMicrotask not available");
    let queue_microtask = queue_microtask.unchecked_into::<Function>();
    _ = queue_microtask.call1(&JsValue::UNDEFINED, &task);
}

thread_local! {
    static DOM_TASK_PENDING: Cell<bool> = const { Cell::new(false) };
    static DOM_TASKS: RefCell<Vec<Box<dyn FnOnce()>>> = const { RefCell::new(Vec::new()) };
}

fn queue(fun: Box<dyn FnOnce()>) {
    enqueue_dom_task(fun, queue_microtask);
}

fn enqueue_dom_task(fun: Box<dyn FnOnce()>, schedule: impl FnOnce(fn())) {
    DOM_TASKS.with_borrow_mut(|tasks| tasks.push(fun));
    if !DOM_TASK_PENDING.replace(true) {
        schedule(flush_dom_tasks);
    }
}

fn flush_dom_tasks() {
    let tasks = DOM_TASKS.take();
    // Setters can synchronously run user code that queues another DOM write.
    // Release both the queue borrow and its pending flag before those callbacks,
    // so nested writes schedule the next microtask even if a callback panics.
    DOM_TASK_PENDING.set(false);
    for task in tasks {
        task();
    }
}

impl Dom {
    pub fn intern(text: &str) -> &str {
        intern(text)
    }

    pub fn create_element(tag: &str, namespace: Option<&str>) -> Element {
        if let Some(namespace) = namespace {
            document()
                .create_element_ns(Some(Self::intern(namespace)), Self::intern(tag))
                .unwrap()
        } else {
            document().create_element(Self::intern(tag)).unwrap()
        }
    }

    pub fn create_text_node(text: &str) -> Text {
        document().create_text_node(text)
    }

    pub fn create_placeholder() -> Placeholder {
        thread_local! {
            static COMMENT: LazyCell<Comment> = LazyCell::new(|| {
                document().create_comment("")
            });
        }
        COMMENT.with(|n| n.clone_node().unwrap().unchecked_into())
    }

    pub fn set_text(node: &Text, text: &str) {
        node.set_node_value(Some(text));
    }

    pub fn set_attribute(node: &Element, name: &str, value: &str) {
        or_debug!(node.set_attribute(name, value), node, "setAttribute");
    }

    pub fn remove_attribute(node: &Element, name: &str) {
        or_debug!(node.remove_attribute(name), node, "removeAttribute");
    }

    pub fn insert_node(parent: &Element, new_child: &Node, anchor: Option<&Node>) {
        ok_or_debug!(
            parent.insert_before(new_child, anchor),
            parent,
            "insertNode"
        );
    }

    pub fn try_insert_node(parent: &Element, new_child: &Node, anchor: Option<&Node>) -> bool {
        parent.insert_before(new_child, anchor).is_ok()
    }

    pub fn remove_node(parent: &Element, child: &Node) -> Option<Node> {
        ok_or_debug!(parent.remove_child(child), parent, "removeNode")
    }

    pub fn remove(node: &Node) {
        node.unchecked_ref::<Element>().remove();
    }

    pub fn get_parent(node: &Node) -> Option<Node> {
        node.parent_node()
    }

    pub fn first_child(node: &Node) -> Option<Node> {
        #[cfg(debug_assertions)]
        {
            let node = node.first_child();
            // if it's a comment node that starts with hot-reload, it's a marker that should be
            // ignored
            if let Some(node) = node.as_ref() {
                if node.node_type() == 8
                    && node
                        .text_content()
                        .unwrap_or_default()
                        .starts_with("hot-reload")
                {
                    return Self::next_sibling(node);
                }
            }

            node
        }
        #[cfg(not(debug_assertions))]
        {
            node.first_child()
        }
    }

    pub fn next_sibling(node: &Node) -> Option<Node> {
        #[cfg(debug_assertions)]
        {
            let node = node.next_sibling();
            // if it's a comment node that starts with hot-reload, it's a marker that should be
            // ignored
            if let Some(node) = node.as_ref() {
                if node.node_type() == 8
                    && node
                        .text_content()
                        .unwrap_or_default()
                        .starts_with("hot-reload")
                {
                    return Self::next_sibling(node);
                }
            }

            node
        }
        #[cfg(not(debug_assertions))]
        {
            node.next_sibling()
        }
    }

    pub fn log_node(node: &Node) {
        web_sys::console::log_1(node);
    }

    pub fn clear_children(parent: &Element) {
        parent.set_text_content(Some(""));
    }

    /// Mounts the new child before the marker as its sibling.
    ///
    /// ## Panics
    /// The default implementation panics if `before` does not have a parent [`crate::view_tree::renderer::types::Element`].
    pub fn mount_before<M>(new_child: &mut M, before: &Node)
    where
        M: Mountable,
    {
        let parent =
            Element::cast_from(Self::get_parent(before).expect("could not find parent element"))
                .expect("placeholder parent should be Element");
        new_child.mount(&parent, Some(before));
    }

    /// Tries to mount the new child before the marker as its sibling.
    ///
    /// Returns `false` if the child did not have a valid parent.
    #[track_caller]
    pub fn try_mount_before<M>(new_child: &mut M, before: &Node) -> bool
    where
        M: Mountable,
    {
        if let Some(parent) = Self::get_parent(before).and_then(Element::cast_from) {
            new_child.mount(&parent, Some(before));
            true
        } else {
            false
        }
    }

    pub fn set_property_or_value(el: &Element, key: &str, value: &JsValue) {
        if key == "value" {
            queue(Box::new({
                let el = el.clone();
                let value = value.clone();
                move || {
                    Self::set_property(&el, "value", &value);
                }
            }))
        } else {
            Self::set_property(el, key, value);
        }
    }

    pub fn set_property(el: &Element, key: &str, value: &JsValue) {
        or_debug!(
            js_sys::Reflect::set(el, &wasm_bindgen::JsValue::from_str(key), value,),
            el,
            "setProperty"
        );
    }

    pub fn add_event_listener(
        el: &Element,
        name: &str,
        cb: Box<dyn FnMut(Event)>,
    ) -> RemoveEventHandler {
        let cb = wasm_bindgen::closure::Closure::wrap(cb);
        let name = intern(name);
        or_debug!(
            el.add_event_listener_with_callback(name, cb.as_ref().unchecked_ref()),
            el,
            "addEventListener"
        );

        // return the remover
        RemoveEventHandler::new({
            let name = name.to_owned();
            let el = el.clone();
            // safe to construct this here, because it will only run in the browser
            // so it will always be accessed or dropped from the main thread
            let cb = crate::thread_bound::SendWrapper::new(move || {
                or_debug!(
                    el.remove_event_listener_with_callback(
                        intern(&name),
                        cb.as_ref().unchecked_ref()
                    ),
                    &el,
                    "removeEventListener"
                )
            });
            move || cb()
        })
    }

    pub fn add_event_listener_use_capture(
        el: &Element,
        name: &str,
        cb: Box<dyn FnMut(Event)>,
    ) -> RemoveEventHandler {
        let cb = wasm_bindgen::closure::Closure::wrap(cb);
        let name = intern(name);
        let options = AddEventListenerOptions::new();
        options.set_capture(true);
        or_debug!(
            el.add_event_listener_with_callback_and_add_event_listener_options(
                name,
                cb.as_ref().unchecked_ref(),
                &options
            ),
            el,
            "addEventListenerUseCapture"
        );

        // return the remover
        RemoveEventHandler::new({
            let name = name.to_owned();
            let el = el.clone();
            // safe to construct this here, because it will only run in the browser
            // so it will always be accessed or dropped from the main thread
            let cb = crate::thread_bound::SendWrapper::new(move || {
                or_debug!(
                    el.remove_event_listener_with_callback_and_bool(
                        intern(&name),
                        cb.as_ref().unchecked_ref(),
                        true
                    ),
                    &el,
                    "removeEventListener"
                )
            });
            move || cb()
        })
    }

    pub fn event_target<T>(ev: &Event) -> T
    where
        T: CastFrom<Element>,
    {
        let el = ev
            .unchecked_ref::<web_sys::Event>()
            .target()
            .expect("event.target not found")
            .unchecked_into::<Element>();
        T::cast_from(el).expect("incorrect element type")
    }

    pub fn class_list(el: &Element) -> ClassList {
        el.class_list()
    }

    pub fn add_class(list: &ClassList, name: &str) {
        or_debug!(list.add_1(name), list.unchecked_ref(), "add()");
    }

    pub fn remove_class(list: &ClassList, name: &str) {
        or_debug!(list.remove_1(name), list.unchecked_ref(), "remove()");
    }

    pub fn style(el: &Element) -> CssStyleDeclaration {
        el.unchecked_ref::<web_sys::HtmlElement>().style()
    }

    pub fn set_css_property(style: &CssStyleDeclaration, name: &str, value: &str) {
        or_debug!(
            style.set_property(name, value),
            style.unchecked_ref(),
            "setProperty"
        );
    }

    pub fn remove_css_property(style: &CssStyleDeclaration, name: &str) {
        or_debug!(
            style.remove_property(name),
            style.unchecked_ref(),
            "removeProperty"
        );
    }

    pub fn set_inner_html(el: &Element, html: &str) {
        el.set_inner_html(html);
    }
}

impl Mountable for Node {
    fn unmount(&mut self) {
        todo!()
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn try_mount(&mut self, parent: &Element, marker: Option<&Node>) -> bool {
        Dom::try_insert_node(parent, self, marker)
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        let parent = Dom::get_parent(self).and_then(Element::cast_from);
        if let Some(parent) = parent {
            child.mount(&parent, Some(self));
            return true;
        }
        false
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}

impl Mountable for Text {
    fn unmount(&mut self) {
        self.remove();
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn try_mount(&mut self, parent: &Element, marker: Option<&Node>) -> bool {
        Dom::try_insert_node(parent, self, marker)
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        let parent = Dom::get_parent(self.as_ref()).and_then(Element::cast_from);
        if let Some(parent) = parent {
            child.mount(&parent, Some(self));
            return true;
        }
        false
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}

impl Mountable for Comment {
    fn unmount(&mut self) {
        self.remove();
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn try_mount(&mut self, parent: &Element, marker: Option<&Node>) -> bool {
        Dom::try_insert_node(parent, self, marker)
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        let parent = Dom::get_parent(self.as_ref()).and_then(Element::cast_from);
        if let Some(parent) = parent {
            child.mount(&parent, Some(self));
            return true;
        }
        false
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}

impl Mountable for Element {
    fn unmount(&mut self) {
        self.remove();
    }

    fn mount(&mut self, parent: &Element, marker: Option<&Node>) {
        Dom::insert_node(parent, self, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        let parent = Dom::get_parent(self.as_ref()).and_then(Element::cast_from);
        if let Some(parent) = parent {
            child.mount(&parent, Some(self));
            return true;
        }
        false
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![self.clone()]
    }
}

impl CastFrom<Node> for Text {
    fn cast_from(node: Node) -> Option<Text> {
        node.clone().dyn_into().ok()
    }
}

impl CastFrom<Node> for Comment {
    fn cast_from(node: Node) -> Option<Comment> {
        node.clone().dyn_into().ok()
    }
}

impl CastFrom<Node> for Element {
    fn cast_from(node: Node) -> Option<Element> {
        node.clone().dyn_into().ok()
    }
}

impl<T> CastFrom<JsValue> for T
where
    T: JsCast,
{
    fn cast_from(source: JsValue) -> Option<Self> {
        source.dyn_into::<T>().ok()
    }
}

impl<T> CastFrom<Element> for T
where
    T: JsCast,
{
    fn cast_from(source: Element) -> Option<Self> {
        source.dyn_into::<T>().ok()
    }
}

#[path = "../../../../../../../test/interface/dom/unit/kernel/view_tree/renderer/dom/queue_tests.rs"]
#[cfg(test)]
mod queue_tests;
