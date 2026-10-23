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
