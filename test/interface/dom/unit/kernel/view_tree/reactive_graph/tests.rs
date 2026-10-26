use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct RetainedState {
    drops: Arc<AtomicUsize>,
    unmounts: Arc<AtomicUsize>,
}

impl Drop for RetainedState {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}

impl Mountable for RetainedState {
    fn unmount(&mut self) {
        self.unmounts.fetch_add(1, Ordering::Relaxed);
    }

    fn mount(
        &mut self,
        _: &crate::view_tree::renderer::types::Element,
        _: Option<&crate::view_tree::renderer::types::Node>,
    ) {
    }

    fn insert_before_this(&self, _: &mut dyn Mountable) -> bool {
        false
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        Vec::new()
    }
}

#[test]
fn dropping_a_reactive_view_releases_its_retained_state_before_the_executor_ticks() {
    crate::tasks::Executor::init_native().expect("native test executor should initialize once");
    let drops = Arc::new(AtomicUsize::new(0));
    let unmounts = Arc::new(AtomicUsize::new(0));
    let effect = RenderEffect::new({
        let drops = Arc::clone(&drops);
        let unmounts = Arc::clone(&unmounts);
        move |_| RetainedState {
            drops: Arc::clone(&drops),
            unmounts: Arc::clone(&unmounts),
        }
    });
    let mut state = RenderEffectState::from(effect);
    state.unmount();
    assert_eq!(unmounts.load(Ordering::Relaxed), 1);
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    drop(state);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
    crate::tasks::Executor::poll_local();
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}
