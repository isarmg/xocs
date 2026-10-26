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

#[test]
fn bidirectional_context_search_preserves_the_requested_type_and_nearest_scope() {
    let ancestor = Owner::new();
    let parent = ancestor.child();
    let child = parent.child();
    ancestor.with(|| provide_context(7_u32));
    parent.with(|| provide_context(Some(99_u32)));
    child.with(|| provide_context(String::from("descendant")));

    assert_eq!(parent.use_context_bidirectional::<u32>(), Some(7));
    assert_eq!(
        parent.use_context_bidirectional::<Option<u32>>(),
        Some(Some(99))
    );
    assert_eq!(
        parent.use_context_bidirectional::<String>().as_deref(),
        Some("descendant")
    );
    parent.with(|| provide_context(8_u32));
    assert_eq!(parent.use_context_bidirectional::<u32>(), Some(8));
    assert_eq!(child.use_context_bidirectional::<u32>(), Some(8));
    assert!(parent.use_context_bidirectional::<bool>().is_none());
}

#[test]
fn context_reads_updates_and_takes_share_the_nearest_typed_value() {
    let parent = Owner::new();
    let child = parent.child();
    parent.with(|| provide_context(String::from("parent")));
    child.with(|| {
        assert_eq!(use_context::<String>().as_deref(), Some("parent"));
        assert_eq!(with_context::<String, _>(String::len), Some(6));
        assert_eq!(
            update_context::<String, _>(|text| text.push_str(" updated")),
            Some(())
        );
        assert_eq!(take_context::<String>().as_deref(), Some("parent updated"));
        assert!(take_context::<String>().is_none());
        provide_context(String::from("child"));
        assert_eq!(take_context::<String>().as_deref(), Some("child"));
    });
    parent.with(|| assert!(use_context::<String>().is_none()));
}
