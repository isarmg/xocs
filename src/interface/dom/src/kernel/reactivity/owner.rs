//! The reactive ownership model, which manages effect cancellation, cleanups, and arena allocation.

use crate::poison::OrPoisoned;

use rustc_hash::FxHashMap;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    fmt::Debug,
    mem,
    sync::{Arc, RwLock, Weak},
};

mod arc_stored_value;
mod arena;
mod arena_item;
mod context;
mod storage;
mod stored_value;
use self::arena::Arena;
pub use arc_stored_value::ArcStoredValue;

use arena::NodeId;
pub use arena_item::*;
pub use context::*;
pub use storage::*;
#[allow(deprecated)] // allow exporting deprecated fn
pub use stored_value::{FromLocal, StoredValue};

/// A reactive owner, which manages
/// 1) the cancellation of [`Effect`](crate::reactivity::effect::Effect)s,
/// 2) providing and accessing environment data via [`provide_context`] and [`use_context`],
/// 3) running cleanup functions defined via [`Owner::on_cleanup`], and
/// 4) an arena storage system to provide `Copy` handles via [`ArenaItem`], which is what allows
///    types like [`RwSignal`](blog_dom::reactivity::signal::RwSignal), [`Memo`](blog_dom::reactivity::computed::Memo), and so on to be `Copy`.
///
/// Every effect and computed reactive value has an associated `Owner`. While it is running, this
/// is marked as the current `Owner`. Whenever it re-runs, this `Owner` is cleared by calling
/// [`Owner::with_cleanup`]. This runs cleanup functions, cancels any [`Effect`](crate::reactivity::effect::Effect)s created during the
/// last run, drops signals stored in the arena, and so on, because those effects and signals will
/// be re-created as needed during the next run.
///
/// When the owner is ultimately dropped, it will clean up its owned resources in the same way.
///
/// The "current owner" is set on the thread-local basis: whenever one of these reactive nodes is
/// running, it will set the current owner on its thread with [`Owner::with`] or [`Owner::set`],
/// allowing other reactive nodes implicitly to access the fact that it is currently the owner.
///
/// For a longer discussion of the ownership model, [see
/// here](https://book.leptos.dev/appendix_life_cycle.html).
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct Owner {
    pub(crate) inner: Arc<RwLock<OwnerInner>>,
}

impl Owner {
    /// Creates a [`WeakOwner`] reference to this owner that does not prevent cleanup.
    pub fn downgrade(&self) -> WeakOwner {
        WeakOwner {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

/// A weak reference to an [`Owner`] that does not prevent cleanup.
///
/// This is useful for capturing an owner reference without creating reference
/// cycles that would prevent the owner from being dropped and cleaned up.
#[derive(Clone)]
pub struct WeakOwner {
    inner: Weak<RwLock<OwnerInner>>,
}

impl WeakOwner {
    /// Attempts to upgrade this weak reference to a strong [`Owner`].
    ///
    /// Returns `None` if the owner has already been dropped.
    pub fn upgrade(&self) -> Option<Owner> {
        self.inner.upgrade().map(|inner| Owner { inner })
    }
}

impl PartialEq for Owner {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

thread_local! {
    static OWNER: RefCell<Option<WeakOwner>> = Default::default();
}

struct RestoreOwner(Option<WeakOwner>);

impl Drop for RestoreOwner {
    fn drop(&mut self) {
        OWNER.with_borrow_mut(|owner| *owner = self.0.take());
    }
}

impl Owner {
    /// Returns a unique identifier for this owner, which can be used to identify it for debugging
    /// purposes.
    ///
    /// Intended for debugging only; this is not guaranteed to be stable between runs.
    pub fn debug_id(&self) -> usize {
        Arc::as_ptr(&self.inner) as usize
    }

    /// Returns the list of parents, grandparents, and ancestors, with values corresponding to
    /// [`Owner::debug_id`] for each.
    ///
    /// Intended for debugging only; this is not guaranteed to be stable between runs.
    pub fn ancestry(&self) -> Vec<usize> {
        let mut ancestors = Vec::new();
        let mut curr_parent = self
            .inner
            .read()
            .or_poisoned()
            .parent
            .as_ref()
            .and_then(|n| n.upgrade());
        while let Some(parent) = curr_parent {
            ancestors.push(Arc::as_ptr(&parent) as usize);
            curr_parent = parent
                .read()
                .or_poisoned()
                .parent
                .as_ref()
                .and_then(|n| n.upgrade());
        }
        ancestors
    }

    /// Creates a new `Owner` and registers it as a child of the current `Owner`, if there is one.
    pub fn new() -> Self {
        Self::current()
            .map(|parent| parent.child())
            .unwrap_or_default()
    }

    /// Returns the parent of this `Owner`, if any.
    ///
    /// None when:
    /// - This is a root owner
    /// - The parent has been dropped
    pub fn parent(&self) -> Option<Owner> {
        self.inner
            .read()
            .or_poisoned()
            .parent
            .as_ref()
            .and_then(|p| p.upgrade())
            .map(|inner| Owner { inner })
    }

    /// Creates a new `Owner` that is the child of the current `Owner`, if any.
    pub fn child(&self) -> Self {
        let parent = Some(Arc::downgrade(&self.inner));
        let mut inner = self.inner.write().or_poisoned();

        let paused = inner.paused;
        let child = Self {
            inner: Arc::new(RwLock::new(OwnerInner {
                parent,
                nodes: Default::default(),
                contexts: Default::default(),
                cleanups: Default::default(),
                children: Default::default(),

                paused,
            })),
        };
        inner.register_child(&child);
        child
    }

    /// Sets this as the current `Owner`.
    pub fn set(&self) {
        OWNER.with_borrow_mut(|owner| *owner = Some(self.downgrade()));
    }

    /// Runs the given function with this as the current `Owner`.
    /// Restores the previous owner on return and during panic unwinding.
    pub fn with<T>(&self, fun: impl FnOnce() -> T) -> T {
        let _restore = RestoreOwner(OWNER.with_borrow_mut(|o| o.replace(self.downgrade())));
        fun()
    }

    /// Cleans up this owner, the given function with this as the current `Owner`.
    pub fn with_cleanup<T>(&self, fun: impl FnOnce() -> T) -> T {
        self.cleanup();
        self.with(fun)
    }

    /// Cleans up this owner in the following order:
    /// 1) Runs `cleanup` on all children,
    /// 2) Runs all cleanup functions registered with [`Owner::on_cleanup`],
    /// 3) Drops the values of any arena-allocated [`ArenaItem`]s.
    pub fn cleanup(&self) {
        self.inner.cleanup();
    }

    /// Registers a function to be run the next time the current owner is cleaned up.
    ///
    /// Because the ownership model is associated with reactive nodes, each "decision point" in an
    /// application tends to have a separate `Owner`: as a result, these cleanup functions often
    /// fill the same need as an "on unmount" function in other UI approaches, etc.
    pub fn on_cleanup(fun: impl FnOnce() + Send + Sync + 'static) {
        if let Some(owner) = Owner::current() {
            let mut inner = owner.inner.write().or_poisoned();

            inner.cleanups.push(Box::new(fun));
        }
    }

    fn register(&self, node: NodeId) {
        self.inner.write().or_poisoned().nodes.push(node);
    }

    /// Returns the current `Owner`, if any.
    pub fn current() -> Option<Owner> {
        OWNER.with(|o| o.borrow().as_ref().and_then(|n| n.upgrade()))
    }

    /// Removes this from its state as the thread-local owner and drops it.
    /// If there are other holders of this owner, it may not cleanup, if always cleaning up is required,
    /// see [`Owner::unset_with_forced_cleanup`].
    pub fn unset(self) {
        OWNER.with_borrow_mut(|owner| {
            if owner
                .as_ref()
                .is_some_and(|current| Weak::ptr_eq(&current.inner, &Arc::downgrade(&self.inner)))
            {
                *owner = None;
            }
        });
        // Drop the strong reference only after releasing the thread-local
        // borrow: cleanups may themselves read or set the current owner.
    }

    /// Removes this from its state as the thread-local owner and drops it.
    /// Unlike [`Owner::unset`], this will always run cleanup on this owner,
    /// even if there are other holders of this owner.
    pub fn unset_with_forced_cleanup(self) {
        OWNER.with_borrow_mut(|owner| {
            if owner
                .as_ref()
                .is_some_and(|current| Weak::ptr_eq(&current.inner, &Arc::downgrade(&self.inner)))
            {
                *owner = None;
            }
        });
        self.cleanup();
    }

    /// Pauses the execution of side effects for this owner, and any of its descendants.
    ///
    /// If this owner is the owner for an [`Effect`](crate::reactivity::effect::Effect) or [`RenderEffect`](crate::reactivity::effect::RenderEffect), this effect will not run until [`Owner::resume`] is called. All children of this effects are also paused.
    ///
    /// Any notifications will be ignored; effects that are notified will paused will not run when
    /// resumed, until they are notified again by a source after being resumed.
    pub fn pause(&self) {
        let mut stack = Vec::with_capacity(16);
        stack.push(Arc::downgrade(&self.inner));
        while let Some(curr) = stack.pop() {
            if let Some(curr) = curr.upgrade() {
                let mut curr = curr.write().or_poisoned();
                curr.paused = true;
                stack.extend(curr.children.iter().map(Weak::clone));
            }
        }
    }

    /// Whether this owner has been paused by [`Owner::pause`].
    pub fn paused(&self) -> bool {
        self.inner.read().or_poisoned().paused
    }

    /// Resumes side effects that have been paused by [`Owner::pause`].
    ///
    /// All children will also be resumed.
    ///
    /// This will *not* cause side effects that were notified while paused to run, until they are
    /// notified again by a source after being resumed.
    pub fn resume(&self) {
        let mut stack = Vec::with_capacity(16);
        stack.push(Arc::downgrade(&self.inner));
        while let Some(curr) = stack.pop() {
            if let Some(curr) = curr.upgrade() {
                let mut curr = curr.write().or_poisoned();
                curr.paused = false;
                stack.extend(curr.children.iter().map(Weak::clone));
            }
        }
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IsHydrating(pub bool);

/// Registers a function to be run the next time the current owner is cleaned up.
///
/// Because the ownership model is associated with reactive nodes, each "decision point" in an
/// application tends to have a separate `Owner`: as a result, these cleanup functions often
/// fill the same need as an "on unmount" function in other UI approaches, etc.
///
/// This is an alias for [`Owner::on_cleanup`].
pub fn on_cleanup(fun: impl FnOnce() + Send + Sync + 'static) {
    Owner::on_cleanup(fun)
}

#[derive(Default)]
pub(crate) struct OwnerInner {
    pub parent: Option<Weak<RwLock<OwnerInner>>>,
    nodes: Vec<NodeId>,
    pub contexts: FxHashMap<TypeId, Box<dyn Any + Send + Sync>>,
    pub cleanups: Vec<Box<dyn FnOnce() + Send + Sync>>,
    pub children: Vec<Weak<RwLock<OwnerInner>>>,

    paused: bool,
}

impl OwnerInner {
    fn register_child(&mut self, child: &Owner) {
        // Keyed rows outlive the list's render effect. Their parent can stay
        // alive for the whole page, so retaining every removed row's weak
        // allocation until parent cleanup would grow with all past edits.
        if self.children.len() == self.children.capacity() {
            self.children.retain(|child| child.strong_count() > 0);
            // Leave geometric headroom even if only a few entries expired.
            // Otherwise a nearly full live list would be scanned on every add.
            if self.children.len() > self.children.capacity() / 2 {
                self.children.reserve(self.children.len());
            }
        }
        self.children.push(Arc::downgrade(&child.inner));
    }
}

impl Debug for OwnerInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnerInner")
            .field("parent", &self.parent)
            .field("nodes", &self.nodes)
            .field("contexts", &self.contexts)
            .field("cleanups", &self.cleanups.len())
            .finish()
    }
}

impl Drop for OwnerInner {
    fn drop(&mut self) {
        for child in std::mem::take(&mut self.children) {
            if let Some(child) = child.upgrade() {
                child.cleanup();
            }
        }

        for cleanup in mem::take(&mut self.cleanups) {
            cleanup();
        }

        let nodes = mem::take(&mut self.nodes);
        if !nodes.is_empty() {
            let mut nodes_to_drop = Vec::with_capacity(nodes.len());

            Arena::with_mut(|arena| {
                for node in nodes {
                    nodes_to_drop.extend(arena.remove(node));
                }
            });
        }
    }
}

trait Cleanup {
    fn cleanup(&self);
}

impl Cleanup for RwLock<OwnerInner> {
    fn cleanup(&self) {
        let (cleanups, nodes, children) = {
            let mut lock = self.write().or_poisoned();
            (
                mem::take(&mut lock.cleanups),
                mem::take(&mut lock.nodes),
                mem::take(&mut lock.children),
            )
        };
        for child in children {
            if let Some(child) = child.upgrade() {
                child.cleanup();
            }
        }
        for cleanup in cleanups {
            cleanup();
        }

        if !nodes.is_empty() {
            let mut nodes_to_drop = Vec::with_capacity(nodes.len());

            Arena::with_mut(|arena| {
                for node in nodes {
                    nodes_to_drop.extend(arena.remove(node));
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn changing_keyed_rows_does_not_retain_every_removed_owner() {
        let parent = Owner::new();
        let cleaned = Arc::new(AtomicUsize::new(0));
        let live = (0..128)
            .map(|_| {
                let child = parent.child();
                child.with(|| {
                    let cleaned = Arc::clone(&cleaned);
                    on_cleanup(move || {
                        cleaned.fetch_add(1, Ordering::Relaxed);
                    });
                });
                child
            })
            .collect::<Vec<_>>();

        for index in 0..10_000 {
            // Both ways of creating a child share the bounded registry.
            let row = if index % 2 == 0 {
                parent.child()
            } else {
                parent.with(Owner::new)
            };
            assert_eq!(row.parent(), Some(parent.clone()));
            drop(row);
        }
        assert!(parent.inner.read().or_poisoned().children.len() <= 2 * live.len());
        assert_eq!(cleaned.load(Ordering::Relaxed), 0);

        // Compaction must preserve all surviving children for parent cleanup.
        parent.cleanup();
        assert_eq!(cleaned.load(Ordering::Relaxed), live.len());
        drop(live);
        assert_eq!(cleaned.load(Ordering::Relaxed), 128);
    }
}
