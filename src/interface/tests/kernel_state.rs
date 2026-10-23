//! Regression contracts for the browser kernel after its libraries became modules.
use blog_interface::{html_escaping, prelude::*, slots::SlotMap};

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
fn escaping_keeps_text_and_quoted_attribute_boundaries() {
    let input = "<script data-x=\"'&\">";
    assert_eq!(
        html_escaping::encode_text(input),
        "&lt;script data-x=\"'&amp;\"&gt;"
    );
    assert_eq!(
        html_escaping::encode_double_quoted_attribute(input),
        "&lt;script data-x=&quot;'&amp;&quot;&gt;"
    );
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
fn tuple_template_class_and_style_skip_empty_fragments() {
    use blog_interface::tachys::view::{Position, ToTemplate};
    struct Empty;
    struct First;
    struct Last;
    macro_rules! fragment {
        ($name:ident, $class:literal, $style:literal) => {
            impl ToTemplate for $name {
                const CLASS: &'static str = $class;
                const STYLE: &'static str = $style;
                fn to_template(
                    _: &mut String,
                    _: &mut String,
                    _: &mut String,
                    _: &mut String,
                    _: &mut Position,
                ) {
                }
            }
        };
    }
    fragment!(Empty, "", "");
    fragment!(First, "red", "color:red");
    fragment!(Last, "selected", "display:block");
    type Mixed = (Empty, First, Empty, Last, Empty);
    assert_eq!(Mixed::CLASS, "red selected");
    assert_eq!(Mixed::STYLE, "color:red;display:block");
    type Blank = (Empty, Empty);
    assert_eq!(Blank::CLASS, "");
    assert_eq!(Blank::STYLE, "");
    type Single = (Empty, First, Empty);
    assert_eq!(Single::CLASS, "red");
    assert_eq!(Single::STYLE, "color:red");
}
