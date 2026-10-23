use blog_collections_slot_storage::Slab;
use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

#[test]
fn two_mutable_lookups_return_none_for_every_absent_key() {
    let mut slab = Slab::new();
    slab.insert(10);
    slab.insert(20);
    slab.insert(30);
    slab.remove(1);
    for left in [0, 1, 2, 3, usize::MAX] {
        for right in [0, 1, 2, 3, usize::MAX] {
            if left == right {
                continue;
            }
            let values = slab.get2_mut(left, right);
            let expected = matches!(left, 0 | 2) && matches!(right, 0 | 2);
            assert_eq!(values.is_some(), expected, "keys {left}/{right}");
            if let Some((left_value, right_value)) = values {
                assert_eq!(*left_value, if left == 0 { 10 } else { 30 });
                assert_eq!(*right_value, if right == 0 { 10 } else { 30 });
                std::mem::swap(left_value, right_value);
                std::mem::swap(left_value, right_value);
            }
        }
    }
}

#[derive(Clone)]
struct DropBomb {
    id: usize,
    drops: Rc<RefCell<Vec<usize>>>,
    panic_at: Rc<Cell<Option<usize>>>,
}

impl Drop for DropBomb {
    fn drop(&mut self) {
        self.drops.borrow_mut().push(self.id);
        if self.panic_at.get() == Some(self.id) {
            self.panic_at.set(None);
            panic!("drop failed");
        }
    }
}

#[test]
fn clear_restores_empty_metadata_before_an_element_drop_panics() {
    let drops = Rc::new(RefCell::new(Vec::new()));
    let panic_at = Rc::new(Cell::new(Some(2)));
    let value = |id| DropBomb {
        id,
        drops: Rc::clone(&drops),
        panic_at: Rc::clone(&panic_at),
    };
    let mut slab = Slab::new();
    for id in 0..4 {
        slab.insert(value(id));
    }
    drop(slab.remove(1));
    assert_eq!(slab.vacant_key(), 1);
    assert!(catch_unwind(AssertUnwindSafe(|| slab.clear())).is_err());
    assert_eq!(slab.len(), 0);
    assert!(slab.is_empty());
    assert_eq!(slab.vacant_key(), 0);
    assert_eq!(slab.iter().count(), 0);
    assert_eq!(slab.insert(value(4)), 0);
    assert_eq!(slab[0].id, 4);
    drop(slab);
    drops.borrow_mut().sort_unstable();
    assert_eq!(*drops.borrow(), [0, 1, 2, 3, 4]);
}

#[derive(Debug)]
struct CloneBomb {
    id: usize,
    panic_at: Rc<Cell<Option<usize>>>,
}

impl Clone for CloneBomb {
    fn clone(&self) -> Self {
        if self.panic_at.get() == Some(self.id) {
            self.panic_at.set(None);
            panic!("clone failed");
        }
        Self {
            id: self.id,
            panic_at: Rc::clone(&self.panic_at),
        }
    }
}

#[test]
fn clone_from_rebuilds_slot_metadata_after_a_partial_clone_panics() {
    let panic_at = Rc::new(Cell::new(None));
    let value = |id| CloneBomb {
        id,
        panic_at: Rc::clone(&panic_at),
    };
    let mut source = Slab::new();
    let mut target = Slab::new();
    for id in 0..3 {
        source.insert(value(id));
        target.insert(value(id + 10));
    }
    source.remove(0);
    panic_at.set(Some(1));
    assert!(catch_unwind(AssertUnwindSafe(|| target.clone_from(&source))).is_err());
    let actual_count = target.iter().count();
    assert_eq!(target.len(), actual_count);
    let vacant = target.vacant_key();
    assert!(!target.contains(vacant));
    assert_eq!(target.insert(value(20)), vacant);
    assert_eq!(target.len(), actual_count + 1);
    assert_eq!(target.remove(vacant).id, 20);
    target.clone_from(&source);
    assert_eq!(target.len(), 2);
    assert_eq!(target.vacant_key(), 0);
}

#[test]
fn clone_from_restores_metadata_when_truncating_old_values_panics() {
    let drops = Rc::new(RefCell::new(Vec::new()));
    let panic_at = Rc::new(Cell::new(Some(12)));
    let value = |id| DropBomb {
        id,
        drops: Rc::clone(&drops),
        panic_at: Rc::clone(&panic_at),
    };
    let mut source = Slab::new();
    source.insert(value(0));
    let mut target = Slab::new();
    for id in 10..13 {
        target.insert(value(id));
    }
    assert!(catch_unwind(AssertUnwindSafe(|| target.clone_from(&source))).is_err());
    assert_eq!(target.len(), target.iter().count());
    assert_eq!(target.len(), 1);
    assert_eq!(target.vacant_key(), 1);
    assert_eq!(target.insert(value(20)), 1);
    drop(target);
    drop(source);
    drops.borrow_mut().sort_unstable();
    assert_eq!(*drops.borrow(), [0, 10, 11, 12, 20]);
}
