use super::enqueue_dom_task;
use std::{cell::RefCell, collections::VecDeque, panic::AssertUnwindSafe, rc::Rc};

type Microtasks = Rc<RefCell<VecDeque<fn()>>>;

fn schedule_into(tasks: &Microtasks) -> impl FnOnce(fn()) {
    let tasks = Rc::clone(tasks);
    move |task| tasks.borrow_mut().push_back(task)
}

fn run_next(tasks: &Microtasks) {
    let task = tasks.borrow_mut().pop_front().expect("scheduled microtask");
    task();
}

#[test]
fn dom_writes_batch_and_reentrant_writes_run_on_the_next_microtask() {
    let tasks = Microtasks::default();
    let trace = Rc::new(RefCell::new(Vec::new()));
    enqueue_dom_task(
        Box::new({
            let trace = Rc::clone(&trace);
            let tasks = Rc::clone(&tasks);
            move || {
                trace.borrow_mut().push("first");
                enqueue_dom_task(
                    Box::new(move || trace.borrow_mut().push("nested")),
                    schedule_into(&tasks),
                );
            }
        }),
        schedule_into(&tasks),
    );
    enqueue_dom_task(
        Box::new({
            let trace = Rc::clone(&trace);
            move || trace.borrow_mut().push("second")
        }),
        schedule_into(&tasks),
    );
    assert_eq!(tasks.borrow().len(), 1, "synchronous writes share a batch");
    run_next(&tasks);
    assert_eq!(*trace.borrow(), ["first", "second"]);
    assert_eq!(
        tasks.borrow().len(),
        1,
        "nested writes schedule another batch"
    );
    run_next(&tasks);
    assert_eq!(*trace.borrow(), ["first", "second", "nested"]);
    assert!(tasks.borrow().is_empty());
}

#[test]
fn panicking_dom_callback_does_not_wedge_future_writes() {
    let tasks = Microtasks::default();
    enqueue_dom_task(Box::new(|| panic!("setter failed")), schedule_into(&tasks));
    assert!(std::panic::catch_unwind(AssertUnwindSafe(|| run_next(&tasks))).is_err());

    let applied = Rc::new(RefCell::new(false));
    enqueue_dom_task(
        Box::new({
            let applied = Rc::clone(&applied);
            move || *applied.borrow_mut() = true
        }),
        schedule_into(&tasks),
    );
    assert_eq!(tasks.borrow().len(), 1);
    run_next(&tasks);
    assert!(*applied.borrow());
    assert!(tasks.borrow().is_empty());
}
