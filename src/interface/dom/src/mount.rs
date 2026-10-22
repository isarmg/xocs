use crate::reactivity::owner::Owner;
use crate::tasks::Executor;
use crate::view_tree::{
    dom::body,
    view::{Mountable, Render},
};
use crate::IntoView;
use web_sys::HtmlElement;

/// Runs the provided closure and mounts the result to the `<body>`.
pub fn mount_to_body<F, N>(f: F)
where
    F: FnOnce() -> N + 'static,
    N: IntoView,
{
    let owner = mount_to(body(), f);
    owner.forget();
}

/// Runs the provided closure and mounts the result to the provided element.
pub fn mount_to<F, N>(parent: HtmlElement, f: F) -> UnmountHandle<N::State>
where
    F: FnOnce() -> N + 'static,
    N: IntoView,
{
    // use wasm-bindgen-futures to drive the reactive system
    // we ignore the return value because an Err here just means the wasm-bindgen executor is
    // already initialized, which is not an issue
    _ = Executor::init_wasm_bindgen();

    // create a new reactive owner and use it as the root node to run the app
    let owner = Owner::new();
    let mountable = owner.with(move || {
        let view = f().into_view();
        let mut mountable = view.build();
        mountable.mount(&parent, None);
        mountable
    });

    // returns a handle that owns the owner
    // when this is dropped, it will clean up the reactive system and unmount the view
    UnmountHandle { owner, mountable }
}

/// Runs the provided closure and mounts the result to the provided element.
pub fn mount_to_renderer<F, N>(
    parent: &crate::view_tree::renderer::types::Element,
    f: F,
) -> UnmountHandle<N::State>
where
    F: FnOnce() -> N + 'static,
    N: Render,
{
    // use wasm-bindgen-futures to drive the reactive system
    // we ignore the return value because an Err here just means the wasm-bindgen executor is
    // already initialized, which is not an issue
    _ = Executor::init_wasm_bindgen();

    // create a new reactive owner and use it as the root node to run the app
    let owner = Owner::new();
    let mountable = owner.with(move || {
        let view = f();
        let mut mountable = view.build();
        mountable.mount(parent, None);
        mountable
    });

    // returns a handle that owns the owner
    // when this is dropped, it will clean up the reactive system and unmount the view
    UnmountHandle { owner, mountable }
}

/// On drop, this will clean up the reactive [`Owner`] and unmount the view created by
/// [`mount_to`].
///
/// If you are using it to create the root of an application, you should use
/// [`UnmountHandle::forget`] to leak it.
#[must_use = "Dropping an `UnmountHandle` will unmount the view and cancel the \
              reactive system. You should either call `.forget()` to keep the \
              view permanently mounted, or store the `UnmountHandle` somewhere \
              and drop it when you'd like to unmount the view."]
pub struct UnmountHandle<M>
where
    M: Mountable,
{
    #[allow(dead_code)]
    owner: Owner,
    mountable: M,
}

impl<M> UnmountHandle<M>
where
    M: Mountable,
{
    /// Leaks the handle, preventing the reactive system from being cleaned up and the view from
    /// being unmounted. This should always be called when [`mount_to`] is used for the root of an
    /// application that should live for the long term.
    pub fn forget(self) {
        std::mem::forget(self);
    }
}

impl<M> Drop for UnmountHandle<M>
where
    M: Mountable,
{
    fn drop(&mut self) {
        self.mountable.unmount();
    }
}
