use crate::attr::{
    any_attribute::{AnyAttribute, IntoAnyAttribute},
    Attribute, NextAttribute,
};
use blog_dom::prelude::*;

/// Function stored to build/rebuild the wrapped children when attributes are added.
type ChildBuilder<T> = dyn Fn(AnyAttribute) -> T + Send + Sync + 'static;

/// Intercepts attributes added to the returned view so the children builder can
/// apply them to an inner element. Use `AttributeInterceptorProps::builder()`;
/// the children closure receives the accumulated `AnyAttribute` value.
#[component(transparent)]
pub fn AttributeInterceptor<Chil, T>(
    /// The elements that will be rendered, with the attributes this component received as a
    /// parameter.
    children: Chil,
) -> impl IntoView
where
    Chil: Fn(AnyAttribute) -> T + Send + Sync + 'static,
    T: IntoView + 'static,
{
    AttributeInterceptorInner::new(children)
}

/// Wrapper to intercept attributes passed to a component so you can apply them to a different
/// element.
struct AttributeInterceptorInner<T: IntoView, A> {
    children_builder: Box<ChildBuilder<T>>,
    children: T,
    attributes: A,
}

impl<T: IntoView> AttributeInterceptorInner<T, ()> {
    /// Use this as the returned view from your component to collect the attributes that are passed
    /// to your component so you can manually handle them.
    pub fn new<F>(children: F) -> Self
    where
        F: Fn(AnyAttribute) -> T + Send + Sync + 'static,
    {
        let children_builder = Box::new(children);
        let children = children_builder(().into_any_attr());

        Self {
            children_builder,
            children,
            attributes: (),
        }
    }
}

impl<T: IntoView, A: Attribute> Render for AttributeInterceptorInner<T, A> {
    type State = <T as Render>::State;

    fn build(self) -> Self::State {
        self.children.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.children.rebuild(state);
    }
}

impl<T: IntoView + 'static, A> AddAnyAttr for AttributeInterceptorInner<T, A>
where
    A: Attribute,
{
    type Output<SomeNewAttr: blog_dom::attr::Attribute> = AttributeInterceptorInner<
        T,
        <<A as NextAttribute>::Output<SomeNewAttr> as Attribute>::CloneableOwned,
    >;

    fn add_any_attr<NewAttr: blog_dom::attr::Attribute>(
        self,
        attr: NewAttr,
    ) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: DomView,
    {
        let attributes = self.attributes.add_any_attr(attr).into_cloneable_owned();

        let children = (self.children_builder)(attributes.clone().into_any_attr());

        AttributeInterceptorInner {
            children_builder: self.children_builder,
            children,
            attributes,
        }
    }
}

impl<T: IntoView + 'static, A: Attribute> DomView for AttributeInterceptorInner<T, A> {
    type Owned = AttributeInterceptorInner<T, A::CloneableOwned>;

    fn into_owned(self) -> Self::Owned {
        AttributeInterceptorInner {
            children_builder: self.children_builder,
            children: self.children,
            attributes: self.attributes.into_cloneable_owned(),
        }
    }
}
