//! Native adapter isolation; browser task dispatch does not use these overrides.
#![cfg(not(target_family = "wasm"))]
use blog_interface::tasks::{
    CustomExecutor, Executor, ExecutorError, PinnedFuture, PinnedLocalFuture,
};
use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Waker},
};

fn run_ready(mut task: PinnedLocalFuture<()>) {
    let mut context = Context::from_waker(Waker::noop());
    assert!(task.as_mut().poll(&mut context).is_ready());
}

#[derive(Default)]
struct GlobalCounts {
    spawn: AtomicUsize,
    local: AtomicUsize,
    poll: AtomicUsize,
}
struct Global(Arc<GlobalCounts>);
impl CustomExecutor for Global {
    fn spawn(&self, task: PinnedFuture<()>) {
        self.0.spawn.fetch_add(1, Ordering::SeqCst);
        run_ready(task);
    }
    fn spawn_local(&self, task: PinnedLocalFuture<()>) {
        self.0.local.fetch_add(1, Ordering::SeqCst);
        run_ready(task);
    }
    fn poll_local(&self) {
        self.0.poll.fetch_add(1, Ordering::SeqCst);
    }
}
#[derive(Default)]
struct LocalCounts {
    spawn: Cell<usize>,
    local: Cell<usize>,
    poll: Cell<usize>,
}
struct Local(Rc<LocalCounts>);
impl CustomExecutor for Local {
    fn spawn(&self, task: PinnedFuture<()>) {
        self.0.spawn.set(self.0.spawn.get() + 1);
        run_ready(task);
    }
    fn spawn_local(&self, task: PinnedLocalFuture<()>) {
        self.0.local.set(self.0.local.get() + 1);
        run_ready(task);
    }
    fn poll_local(&self) {
        self.0.poll.set(self.0.poll.get() + 1);
    }
}

#[test]
fn local_overrides_are_isolated_reentrant_and_preserve_global_fallback() {
    let global = Arc::new(GlobalCounts::default());
    Executor::init_custom_executor(Global(Arc::clone(&global))).unwrap();
    Executor::spawn(async {});
    Executor::spawn_local(async {});
    Executor::poll_local();
    assert_eq!(global.spawn.load(Ordering::SeqCst), 1);
    assert_eq!(global.local.load(Ordering::SeqCst), 1);
    assert_eq!(global.poll.load(Ordering::SeqCst), 1);

    let main_local = Rc::new(LocalCounts::default());
    Executor::init_local_custom_executor(Local(Rc::clone(&main_local))).unwrap();
    assert!(matches!(
        Executor::init_local_custom_executor(Local(Rc::clone(&main_local))),
        Err(ExecutorError::AlreadySet)
    ));
    let completed = Rc::new(Cell::new(0));
    let completed_task = Rc::clone(&completed);
    Executor::spawn_local(async move {
        Executor::poll_local();
        Executor::spawn_local(async move {
            completed_task.set(completed_task.get() + 1);
        });
    });
    Executor::spawn(async {});
    assert_eq!(completed.get(), 1);
    assert_eq!(
        (
            main_local.spawn.get(),
            main_local.local.get(),
            main_local.poll.get()
        ),
        (1, 2, 1)
    );

    let thread_global = Arc::clone(&global);
    std::thread::spawn(move || {
        // The first thread's override neither replaces the global backend nor
        // makes this thread consult a missing thread-local executor.
        Executor::spawn(async {});
        Executor::spawn_local(async {});
        Executor::poll_local();
        assert_eq!(thread_global.spawn.load(Ordering::SeqCst), 2);
        assert_eq!(thread_global.local.load(Ordering::SeqCst), 2);
        assert_eq!(thread_global.poll.load(Ordering::SeqCst), 2);
        let local = Rc::new(LocalCounts::default());
        Executor::init_local_custom_executor(Local(Rc::clone(&local))).unwrap();
        assert!(matches!(
            Executor::init_local_custom_executor(Local(Rc::clone(&local))),
            Err(ExecutorError::AlreadySet)
        ));
        Executor::spawn(async {});
        Executor::spawn_local(async {
            Executor::poll_local();
        });
        assert_eq!(
            (local.spawn.get(), local.local.get(), local.poll.get()),
            (1, 1, 1)
        );
    })
    .join()
    .unwrap();

    Executor::poll_local();
    assert_eq!(main_local.poll.get(), 2);
    assert_eq!(global.poll.load(Ordering::SeqCst), 2);
}
