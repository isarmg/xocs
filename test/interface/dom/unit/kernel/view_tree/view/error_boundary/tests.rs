use super::*;
use crate::view_errors::{self, ErrorId};
use std::{collections::HashSet, sync::Mutex};

#[derive(Default)]
struct Hook(Mutex<HashSet<ErrorId>>);

impl ErrorHook for Hook {
    fn throw(&self, _: AnyError) -> ErrorId {
        let id = ErrorId::from(0);
        assert!(self.0.lock().unwrap().insert(id.clone()));
        id
    }

    fn clear(&self, id: &ErrorId) {
        self.0.lock().unwrap().remove(id);
    }
}

struct NoDomView;

impl Render for NoDomView {
    type State = ();

    fn build(self) {}

    fn rebuild(self, _: &mut ()) {
        assert!(view_errors::get_error_hook().is_none());
    }
}

#[test]
fn dropping_error_registration_uses_its_own_hook_outside_the_render_scope() {
    for inside_another_boundary in [false, true] {
        let original = Arc::new(Hook::default());
        let other = Arc::new(Hook::default());
        let state = {
            let _scope = view_errors::set_error_hook(original.clone());
            // Keep the registration/drop path independent of browser DOM APIs.
            let mut state = Ok::<_, std::io::Error>(NoDomView).build();
            state.error = Some(view_errors::throw(std::io::Error::other("original")));
            state
        };
        let other_error = other.throw(std::io::Error::other("other").into());
        let _other_scope =
            inside_another_boundary.then(|| view_errors::set_error_hook(other.clone()));

        drop(state);

        assert!(original.0.lock().unwrap().is_empty());
        assert!(other.0.lock().unwrap().contains(&other_error));
        assert_eq!(
            view_errors::get_error_hook().is_some(),
            inside_another_boundary
        );
    }
}

#[test]
fn a_result_rebuild_preserves_its_absent_captured_hook() {
    let mut state = Ok::<_, std::io::Error>(NoDomView).build();
    let other: Arc<dyn ErrorHook> = Arc::new(Hook::default());
    let _scope = view_errors::set_error_hook(other.clone());
    Ok::<_, std::io::Error>(NoDomView).rebuild(&mut state);
    assert!(Arc::ptr_eq(&view_errors::get_error_hook().unwrap(), &other));
}
