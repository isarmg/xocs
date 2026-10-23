use super::{add_attr::AddAnyAttr, DomView};
use crate::alternatives::Either;
use crate::view_errors::{Error as AnyError, ErrorHook};
use crate::view_tree::{
    html::attribute::Attribute,
    view::{iterators::OptionState, Mountable, Render},
};
use std::sync::Arc;

impl<T, E> Render for Result<T, E>
where
    T: Render,
    E: Into<AnyError> + 'static,
{
    type State = ResultState<T>;

    fn build(self) -> Self::State {
        let hook = crate::view_errors::get_error_hook();
        let (state, error) = match self {
            Ok(view) => (Either::Left(view.build()), None),
            Err(e) => (
                Either::Right(Render::build(())),
                Some(crate::view_errors::throw(e.into())),
            ),
        };
        ResultState { state, error, hook }
    }

    fn rebuild(self, state: &mut Self::State) {
        let _guard = crate::view_errors::set_optional_error_hook(state.hook.clone());
        match (&mut state.state, self) {
            // both errors: throw the new error and replace
            (Either::Right(_), Err(new)) => {
                if let Some(old_error) = state.error.take() {
                    crate::view_errors::clear(&old_error);
                }
                state.error = Some(crate::view_errors::throw(new.into()));
            }
            // both Ok: need to rebuild child
            (Either::Left(old), Ok(new)) => {
                T::rebuild(new, old);
            }
            // Ok => Err: unmount, replace with marker, and throw
            (Either::Left(old), Err(err)) => {
                let mut new_state = Render::build(());
                old.insert_before_this(&mut new_state);
                old.unmount();
                state.state = Either::Right(new_state);
                state.error = Some(crate::view_errors::throw(err));
            }
            // Err => Ok: clear error and build
            (Either::Right(old), Ok(new)) => {
                if let Some(err) = state.error.take() {
                    crate::view_errors::clear(&err);
                }
                let mut new_state = new.build();
                old.insert_before_this(&mut new_state);
                old.unmount();
                state.state = Either::Left(new_state);
            }
        }
    }
}

/// View state for a `Result<_, _>` view.
pub struct ResultState<T>
where
    T: Render,
{
    /// The view state.
    state: OptionState<T>,
    error: Option<crate::view_errors::ErrorId>,
    hook: Option<Arc<dyn ErrorHook>>,
}

impl<T> Drop for ResultState<T>
where
    T: Render,
{
    fn drop(&mut self) {
        // when the state is cleared, unregister this error; this item is being dropped and its
        // error should no longer be shown
        if let Some(e) = self.error.take() {
            if let Some(hook) = &self.hook {
                hook.clear(&e);
            }
        }
    }
}

impl<T> Mountable for ResultState<T>
where
    T: Render,
{
    fn unmount(&mut self) {
        self.state.unmount();
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        self.state.mount(parent, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        self.state.insert_before_this(child)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        self.state.elements()
    }
}

impl<T, E> AddAnyAttr for Result<T, E>
where
    T: AddAnyAttr,

    E: Into<AnyError> + Send + 'static,
{
    type Output<SomeNewAttr: Attribute> = Result<<T as AddAnyAttr>::Output<SomeNewAttr>, E>;

    fn add_any_attr<NewAttr: Attribute>(self, attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: DomView,
    {
        self.map(|inner| inner.add_any_attr(attr))
    }
}

impl<T, E> DomView for Result<T, E>
where
    T: DomView,
    E: Into<AnyError> + Send + 'static,
{
    type Owned = Result<T::Owned, E>;

    fn into_owned(self) -> Self::Owned {
        match self {
            Ok(view) => Ok(view.into_owned()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
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
}
