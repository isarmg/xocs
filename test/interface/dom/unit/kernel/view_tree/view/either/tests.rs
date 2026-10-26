use super::*;
use std::{cell::Cell, rc::Rc};

struct RetainedView {
    can_insert: bool,
    drops: Rc<Cell<usize>>,
}

impl Drop for RetainedView {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

impl Render for RetainedView {
    type State = Self;

    fn build(self) -> Self::State {
        self
    }

    fn rebuild(self, state: &mut Self::State) {
        *state = self;
    }
}

impl Mountable for RetainedView {
    fn unmount(&mut self) {}

    fn mount(
        &mut self,
        _: &crate::view_tree::renderer::types::Element,
        _: Option<&crate::view_tree::renderer::types::Node>,
    ) {
    }

    fn insert_before_this(&self, _: &mut dyn Mountable) -> bool {
        self.can_insert
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        Vec::new()
    }
}

#[test]
fn keep_alive_switches_to_the_retained_branch_without_requiring_another_value() {
    let drops = Rc::new(Cell::new(0));
    let a = RetainedView {
        can_insert: true,
        drops: Rc::clone(&drops),
    };
    let b = RetainedView {
        can_insert: false,
        drops: Rc::clone(&drops),
    };
    let mut state = EitherKeepAlive::<RetainedView, RetainedView> {
        a: Some(a),
        b: None,
        show_b: false,
    }
    .build();
    let mut child = ();
    assert!(state.insert_before_this(&mut child));
    EitherKeepAlive::<RetainedView, RetainedView> {
        a: None,
        b: Some(b),
        show_b: true,
    }
    .rebuild(&mut state);
    assert!(!state.insert_before_this(&mut child));
    assert_eq!(drops.get(), 0);
    EitherKeepAlive::<RetainedView, RetainedView> {
        a: None,
        b: None,
        show_b: false,
    }
    .rebuild(&mut state);
    assert!(state.insert_before_this(&mut child));
    assert_eq!(drops.get(), 0);
    drop(state);
    assert_eq!(drops.get(), 2);
}

#[test]
#[should_panic(expected = "selected keep-alive branch has been provided")]
fn a_missing_selected_keep_alive_branch_is_rejected_when_built() {
    EitherKeepAlive::<RetainedView, RetainedView> {
        a: None,
        b: None,
        show_b: true,
    }
    .build();
}
