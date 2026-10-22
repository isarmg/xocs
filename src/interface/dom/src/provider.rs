use crate::reactivity::owner::{provide_context, Owner};
use crate::view_tree::reactive_graph::OwnedView;
use crate::{children::TypedChildren, component, IntoView};

#[component]
/// Provides context to children in a distinct reactive owner without overwriting
/// the surrounding owner's value. Construct with `ProviderProps::builder()` and
/// `ToChildren::to_children`.
pub fn Provider<T, Chil>(
    /// The value to be provided via context.
    value: T,
    children: TypedChildren<Chil>,
) -> impl IntoView
where
    T: Send + Sync + 'static,
    Chil: IntoView + 'static,
{
    let owner = Owner::current()
        .expect("no current reactive Owner found")
        .child();
    let children = children.into_inner();
    let children = owner.with(|| {
        provide_context(value);
        children()
    });
    OwnedView::new_with_owner(children, owner)
}
