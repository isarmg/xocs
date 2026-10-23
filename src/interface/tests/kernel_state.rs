//! Regression contracts for the browser kernel after its libraries became modules.
use blog_interface::{prelude::*, slots::SlotMap};

#[test]
fn reused_slots_never_resurrect_a_disposed_key() {
    let mut slots = SlotMap::new();
    let removed = slots.insert(String::from("old owner"));
    assert_eq!(slots.remove(removed).as_deref(), Some("old owner"));
    let replacement = slots.insert(String::from("new owner"));
    assert_ne!(removed, replacement);
    assert!(slots.get(removed).is_none());
    assert_eq!(
        slots.get(replacement).map(String::as_str),
        Some("new owner")
    );
}

#[test]
fn memo_observes_updates_without_an_async_resource_or_executor() {
    let owner = Owner::new();
    owner.with(|| {
        let value = RwSignal::new(3);
        let doubled = Memo::new(move |_| value.get() * 2);
        assert_eq!(doubled.get(), 6);
        value.set(7);
        assert_eq!(doubled.get(), 14);
    });
}

#[test]
fn owner_cleanup_runs_once_and_disposes_its_signals() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let owner = Owner::new();
    let cleaned = Arc::new(AtomicUsize::new(0));
    let signal = owner.with(|| {
        let cleaned = Arc::clone(&cleaned);
        on_cleanup(move || {
            cleaned.fetch_add(1, Ordering::SeqCst);
        });
        RwSignal::new(42)
    });
    assert_eq!(signal.try_get(), Some(42));
    owner.cleanup();
    owner.cleanup();
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    assert_eq!(signal.try_get(), None);
}

#[test]
fn owner_scope_restores_context_after_a_caught_panic() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    let outer = Owner::new();
    let inner = outer.child();
    outer.with(|| {
        provide_context(String::from("outer"));
        let result = catch_unwind(AssertUnwindSafe(|| {
            inner.with(|| {
                provide_context(String::from("inner"));
                panic!("failed component construction");
            });
        }));
        assert!(result.is_err());
        assert_eq!(Owner::current(), Some(outer.clone()));
        assert_eq!(use_context::<String>().as_deref(), Some("outer"));

        // Values created after recovery must belong to the recovered scope.
        let value = RwSignal::new(7);
        inner.cleanup();
        assert_eq!(value.try_get(), Some(7));
        outer.cleanup();
        assert_eq!(value.try_get(), None);
    });
}

#[test]
fn unsetting_last_owner_allows_cleanup_to_access_current_owner() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    let owner = Owner::new();
    owner.set();
    let cleaned = Arc::new(AtomicBool::new(false));
    on_cleanup({
        let cleaned = Arc::clone(&cleaned);
        move || {
            assert!(Owner::current().is_none());
            // Cleanup may create an independent scope without a TLS borrow panic.
            let replacement = Owner::new();
            replacement.with(|| assert_eq!(Owner::current(), Some(replacement.clone())));
            cleaned.store(true, Ordering::SeqCst);
        }
    });
    owner.unset();
    assert!(cleaned.load(Ordering::SeqCst));
    assert!(Owner::current().is_none());
}

#[test]
fn nested_observers_preserve_the_outer_untracked_scope() {
    use blog_interface::reactive::graph::{Observer, ToAnySubscriber, WithObserver};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    let owner = Owner::new();
    owner.with(|| {
        let signal = RwSignal::new(1);
        let unrelated = RwSignal::new(10);
        let runs = Arc::new(AtomicUsize::new(0));
        let memo = ArcMemo::new({
            let runs = Arc::clone(&runs);
            move |_| {
                runs.fetch_add(1, Ordering::Relaxed);
                signal.get()
            }
        });
        assert_eq!(memo.get_untracked(), 1);
        let subscriber = memo.to_any_subscriber();

        subscriber.with_observer_untracked(|| {
            assert!(Observer::get().is_none());
            untrack(|| assert!(Observer::get().is_none()));
            assert!(Observer::get().is_none());
            assert_eq!(unrelated.get(), 10);
            subscriber.with_observer(|| assert_eq!(Observer::get(), Some(subscriber.clone())));
            assert!(Observer::get().is_none());
            subscriber.with_observer_untracked(|| assert!(Observer::get().is_none()));
            assert!(Observer::get().is_none());
        });
        assert!(Observer::get().is_none());
        unrelated.set(20);
        assert_eq!(memo.get_untracked(), 1);
        assert_eq!(runs.load(Ordering::Relaxed), 1);
        signal.set(2);
        assert_eq!(memo.get_untracked(), 2);
        assert_eq!(runs.load(Ordering::Relaxed), 2);

        subscriber.with_observer(|| {
            subscriber.with_observer_untracked(|| {
                untrack(|| {});
            });
            assert_eq!(Observer::get(), Some(subscriber.clone()));
        });
    });
}

#[test]
fn new_owners_inherit_the_current_parents_pause_state() {
    let parent = Owner::new();
    parent.pause();
    let child = parent.with(Owner::new);
    let grandchild = child.with(Owner::new);
    assert!(child.paused());
    assert!(grandchild.paused());
    parent.resume();
    assert!(!child.paused());
    assert!(!grandchild.paused());
}

#[test]
fn lazy_groups_keep_equal_keys_and_buffer_unconsumed_previous_groups() {
    use blog_interface::iterators::Itertools;
    let grouped = [1, 1, 2, 2, 1].into_iter().chunk_by(|value| *value);
    let mut groups = grouped.into_iter();
    let (first_key, first) = groups.next().unwrap();
    let (second_key, second) = groups.next().unwrap();
    assert_eq!((first_key, second_key), (1, 2));
    assert_eq!(first.collect::<Vec<_>>(), [1, 1]);
    assert_eq!(second.collect::<Vec<_>>(), [2, 2]);
    let (last_key, last) = groups.next().unwrap();
    assert_eq!(last_key, 1);
    assert_eq!(last.collect::<Vec<_>>(), [1]);
    assert!(groups.next().is_none());
}

#[test]
fn erased_render_only_views_retain_state_across_updates_and_drop_it_once() {
    use blog_interface::view_tree::{
        renderer::types::{Element, Node},
        view::{DomView, Mountable, Render, any_view::IntoAny},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    // A renderable resource has no element attributes. Erasing it must not
    // require an attribute adapter or rebuild its retained resource on update.
    struct ResourceView {
        value: usize,
        current: Arc<AtomicUsize>,
        builds: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
    }
    struct ResourceState {
        current: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
    }
    impl Drop for ResourceState {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::Relaxed);
        }
    }
    impl Mountable for ResourceState {
        fn unmount(&mut self) {}
        fn mount(&mut self, _: &Element, _: Option<&Node>) {}
        fn insert_before_this(&self, _: &mut dyn Mountable) -> bool {
            false
        }
        fn elements(&self) -> Vec<Element> {
            Vec::new()
        }
    }
    impl Render for ResourceView {
        type State = ResourceState;
        fn build(self) -> Self::State {
            self.builds.fetch_add(1, Ordering::Relaxed);
            self.current.store(self.value, Ordering::Relaxed);
            ResourceState {
                current: self.current,
                drops: self.drops,
            }
        }
        fn rebuild(self, state: &mut Self::State) {
            state.current.store(self.value, Ordering::Relaxed);
        }
    }
    impl DomView for ResourceView {
        type Owned = Self;
        fn into_owned(self) -> Self {
            self
        }
    }

    let current = Arc::new(AtomicUsize::new(0));
    let builds = Arc::new(AtomicUsize::new(0));
    let drops = Arc::new(AtomicUsize::new(0));
    let view = |value| ResourceView {
        value,
        current: current.clone(),
        builds: builds.clone(),
        drops: drops.clone(),
    };
    let mut state = view(1).into_view().into_any().build();
    assert_eq!(current.load(Ordering::Relaxed), 1);
    view(2).into_view().into_any().rebuild(&mut state);
    assert_eq!(current.load(Ordering::Relaxed), 2);
    assert_eq!(builds.load(Ordering::Relaxed), 1);
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    drop(state);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}
