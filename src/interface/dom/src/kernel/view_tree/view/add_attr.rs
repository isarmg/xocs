use super::DomView;
use crate::view_tree::html::attribute::Attribute;

/// Allows adding a new attribute to some type, before it is rendered.
/// This takes place at compile time as part of the builder syntax for creating a statically typed
/// view tree.
///
/// HTML elements and typed composite views implement this capability. Rendering a [`DomView`]
/// does not require it: attributes must be composed while the receiving view is still typed,
/// before converting it into an erased view.
pub trait AddAnyAttr {
    /// The new type once the attribute has been added.
    type Output<SomeNewAttr: Attribute>: DomView;

    /// Adds an attribute to the view.
    fn add_any_attr<NewAttr: Attribute>(self, attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: DomView;
}

/// Declares that spreading attributes onto a particular type has no effect.
#[macro_export]
macro_rules! no_attrs {
    ($ty_name:ty) => {
        impl<'a> $crate::view_tree::view::add_attr::AddAnyAttr for $ty_name {
            type Output<SomeNewAttr: $crate::view_tree::html::attribute::Attribute> = $ty_name;

            fn add_any_attr<NewAttr: $crate::view_tree::html::attribute::Attribute>(
                self,
                _attr: NewAttr,
            ) -> Self::Output<NewAttr> {
                self
            }
        }
    };
}
