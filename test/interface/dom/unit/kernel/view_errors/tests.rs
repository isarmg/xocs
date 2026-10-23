use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

struct NoopHook;

impl ErrorHook for NoopHook {
    fn throw(&self, _: Error) -> ErrorId {
        ErrorId::from(0)
    }

    fn clear(&self, _: &ErrorId) {}
}

struct ReentrantHook;

impl ErrorHook for ReentrantHook {
    fn throw(&self, _: Error) -> ErrorId {
        let _nested = set_error_hook(Arc::new(NoopHook));
        ErrorId::from(1)
    }

    fn clear(&self, _: &ErrorId) {
        let _nested = set_error_hook(Arc::new(NoopHook));
    }
}

#[test]
fn error_callbacks_can_enter_a_nested_hook_scope() {
    let hook: Arc<dyn ErrorHook> = Arc::new(ReentrantHook);
    let _scope = set_error_hook(hook.clone());
    let id = throw(std::io::Error::other("nested hook"));
    assert_eq!(id, ErrorId::from(1));
    clear(&id);
    assert!(Arc::ptr_eq(&get_error_hook().unwrap(), &hook));
}

struct DropHook(Arc<AtomicBool>);

impl ErrorHook for DropHook {
    fn throw(&self, _: Error) -> ErrorId {
        ErrorId::from(0)
    }

    fn clear(&self, _: &ErrorId) {}
}

impl Drop for DropHook {
    fn drop(&mut self) {
        assert!(get_error_hook().is_none());
        self.0.store(true, Ordering::Relaxed);
    }
}

#[test]
fn restoring_a_scope_drops_its_hook_outside_the_thread_local_borrow() {
    let dropped = Arc::new(AtomicBool::new(false));
    let scope = set_error_hook(Arc::new(DropHook(dropped.clone())));
    drop(scope);
    assert!(dropped.load(Ordering::Relaxed));
    assert!(get_error_hook().is_none());
}

#[test]
fn a_future_preserves_an_absent_captured_hook_when_polled_in_another_scope() {
    let future = ErrorHookFuture::new(async {
        assert!(get_error_hook().is_none());
        42
    });
    let outer: Arc<dyn ErrorHook> = Arc::new(NoopHook);
    let _scope = set_error_hook(outer.clone());
    assert_eq!(futures::executor::block_on(future), 42);
    assert!(Arc::ptr_eq(&get_error_hook().unwrap(), &outer));
}
