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

#[path = "../../../../../../../test/interface/dom/unit/kernel/view_tree/view/error_boundary/tests.rs"]
#[cfg(test)]
mod tests;
