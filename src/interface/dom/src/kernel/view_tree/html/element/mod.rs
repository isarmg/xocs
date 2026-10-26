#[cfg(erase_components)]
use crate::view_tree::view::any_view::AnyView;
use crate::view_tree::{
    html::attribute::Attribute,
    renderer::{CastFrom, Rndr},
    view::{add_attr::AddAnyAttr, DomView, IntoRender, Mountable, Render},
};
use std::ops::Deref;

mod custom;
mod element_ext;
mod elements;
mod inner_html;
use super::attribute::NextAttribute;
pub use custom::*;
pub use element_ext::*;
pub use elements::*;
pub use inner_html::*;
#[cfg(any(debug_assertions, leptos_debuginfo))]
use std::panic::Location;

/// The typed representation of an HTML element.
#[derive(Debug, PartialEq, Eq)]
pub struct HtmlElement<E, At, Ch> {
    #[cfg(any(debug_assertions, leptos_debuginfo))]
    pub(crate) defined_at: &'static Location<'static>,
    pub(crate) tag: E,
    pub(crate) attributes: At,
    pub(crate) children: Ch,
}

impl<E: Clone, At: Clone, Ch: Clone> Clone for HtmlElement<E, At, Ch> {
    fn clone(&self) -> Self {
        HtmlElement {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at: self.defined_at,
            tag: self.tag.clone(),
            attributes: self.attributes.clone(),
            children: self.children.clone(),
        }
    }
}

impl<E: Copy, At: Copy, Ch: Copy> Copy for HtmlElement<E, At, Ch> {}

#[cfg(not(erase_components))]
impl<E, At, Ch, NewChild> ElementChild<NewChild> for HtmlElement<E, At, Ch>
where
    E: ElementWithChildren,
    Ch: DomView + crate::tuples::NextTuple,
    <Ch as crate::tuples::NextTuple>::Output<NewChild::Output>: Render,

    NewChild: IntoRender,
    NewChild::Output: DomView,
{
    type Output = HtmlElement<E, At, <Ch as crate::tuples::NextTuple>::Output<NewChild::Output>>;

    fn child(self, child: NewChild) -> Self::Output {
        HtmlElement {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at: self.defined_at,
            tag: self.tag,
            attributes: self.attributes,
            children: self.children.next_tuple(child.into_render()),
        }
    }
}

#[cfg(erase_components)]
impl<E, At, Ch, NewChild> ElementChild<NewChild> for HtmlElement<E, At, Ch>
where
    E: ElementWithChildren,
    Ch: DomView + NextChildren,

    NewChild: IntoRender,
    NewChild::Output: DomView,
{
    type Output = HtmlElement<E, At, crate::view_tree::view::iterators::StaticVec<AnyView>>;

    fn child(self, child: NewChild) -> Self::Output {
        use crate::view_tree::view::any_view::IntoAny;

        HtmlElement {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at: self.defined_at,
            tag: self.tag,
            attributes: self.attributes,
            children: self.children.next_children(child.into_render().into_any()),
        }
    }
}

#[cfg(erase_components)]
trait NextChildren {
    fn next_children(self, child: AnyView)
        -> crate::view_tree::view::iterators::StaticVec<AnyView>;
}

#[cfg(erase_components)]
mod erased_tuples {
    use super::*;
    use crate::view_tree::view::{any_view::IntoAny, iterators::StaticVec};

    impl NextChildren for StaticVec<AnyView> {
        fn next_children(mut self, child: AnyView) -> StaticVec<AnyView> {
            self.0.push(child);
            self
        }
    }

    impl NextChildren for () {
        fn next_children(self, child: AnyView) -> StaticVec<AnyView> {
            vec![child].into()
        }
    }

    impl<T: DomView> NextChildren for (T,) {
        fn next_children(self, child: AnyView) -> StaticVec<AnyView> {
            vec![self.0.into_owned().into_any(), child].into()
        }
    }

    macro_rules! impl_next_children_tuples {
        ($($ty:ident),*) => {
            impl<$($ty: DomView),*> NextChildren for ($($ty,)*)
             {
                fn next_children(
                    self, child: AnyView,
                ) -> StaticVec<AnyView> {
                    #[allow(non_snake_case)]
                    let ($($ty,)*) = self;
                    vec![$($ty.into_owned().into_any(),)* child].into()
                }
            }
        };
    }

    impl_next_children_tuples!(AA, BB);
    impl_next_children_tuples!(AA, BB, CC);
    impl_next_children_tuples!(AA, BB, CC, DD);
    impl_next_children_tuples!(AA, BB, CC, DD, EE);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II, JJ);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP);
    impl_next_children_tuples!(AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ);
    impl_next_children_tuples!(
        AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ, RR
    );
    impl_next_children_tuples!(
        AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ, RR, SS
    );
    impl_next_children_tuples!(
        AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ, RR, SS, TT
    );
    impl_next_children_tuples!(
        AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ, RR, SS, TT, UU
    );
    impl_next_children_tuples!(
        AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ, RR, SS, TT, UU, VV
    );
    impl_next_children_tuples!(
        AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ, RR, SS, TT, UU, VV, WW
    );
    impl_next_children_tuples!(
        AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ, RR, SS, TT, UU, VV, WW,
        XX
    );
    impl_next_children_tuples!(
        AA, BB, CC, DD, EE, FF, GG, HH, II, JJ, KK, LL, MM, NN, OO, PP, QQ, RR, SS, TT, UU, VV, WW,
        XX, YY
    );
}

impl<E, At, Ch> AddAnyAttr for HtmlElement<E, At, Ch>
where
    E: ElementType + Send,
    At: Attribute + Send,
    Ch: DomView + Send,
{
    type Output<SomeNewAttr: Attribute> =
        HtmlElement<E, <At as NextAttribute>::Output<SomeNewAttr>, Ch>;

    fn add_any_attr<NewAttr: Attribute>(self, attr: NewAttr) -> Self::Output<NewAttr> {
        let HtmlElement {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at,
            tag,
            attributes,
            children,
        } = self;
        HtmlElement {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at,
            tag,
            attributes: attributes.add_any_attr(attr),
            children,
        }
    }
}

/// Adds a child to the element.
pub trait ElementChild<NewChild>
where
    NewChild: IntoRender,
{
    /// The type of the element, with the child added.
    type Output;

    /// Adds a child to an element.
    fn child(self, child: NewChild) -> Self::Output;
}

/// An HTML element.
pub trait ElementType: Send + 'static {
    /// The underlying native widget type that this represents.
    type Output;

    /// The element's tag.
    const TAG: &'static str;
    /// Whether the element is self-closing.
    const SELF_CLOSING: bool;

    /// The element's namespace, if it is not HTML.
    const NAMESPACE: Option<&'static str>;

    /// The element's tag.
    fn tag(&self) -> &str;
}

/// Denotes that the type that implements this has a particular HTML element type.
pub trait HasElementType {
    /// The element type.
    type ElementType;
}

pub(crate) trait ElementWithChildren {}

impl<E, At, Ch> HasElementType for HtmlElement<E, At, Ch>
where
    E: ElementType,
{
    type ElementType = E::Output;
}

impl<E, At, Ch> Render for HtmlElement<E, At, Ch>
where
    E: ElementType,
    At: Attribute,
    Ch: Render,
{
    type State = ElementState<At::State, Ch::State>;

    fn rebuild(self, state: &mut Self::State) {
        // check whether the tag is the same, for custom elements
        // because this is const `false` for all other element types,
        // the compiler should be able to optimize it out
        if E::TAG.is_empty() {
            // see https://github.com/leptos-rs/leptos/issues/4412
            let new_tag = self.tag.tag();

            // this is not particularly efficient, but it saves us from
            // having to keep track of the tag name for every element state
            let old_tag = state.el.tag_name();
            if new_tag != old_tag {
                let mut new_state = self.build();
                state.insert_before_this(&mut new_state);
                state.unmount();
                *state = new_state;
                return;
            }
        }

        // rebuild attributes and children for any element
        let ElementState {
            attrs, children, ..
        } = state;
        self.attributes.rebuild(attrs);
        if let Some(children) = children {
            self.children.rebuild(children);
        }
    }

    fn build(self) -> Self::State {
        let el = Rndr::create_element(self.tag.tag(), E::NAMESPACE);

        let attrs = self.attributes.build(&el);

        let children = if E::SELF_CLOSING {
            None
        } else {
            let mut children = self.children.build();
            children.mount(&el, None);
            Some(children)
        };

        ElementState {
            el,
            attrs,
            children,
        }
    }
}

impl<E, At, Ch> DomView for HtmlElement<E, At, Ch>
where
    E: ElementType + Send,
    At: Attribute + Send,
    Ch: DomView + Send,
{
    type Owned = HtmlElement<E, At::CloneableOwned, Ch::Owned>;

    fn into_owned(self) -> Self::Owned {
        HtmlElement {
            #[cfg(any(debug_assertions, leptos_debuginfo))]
            defined_at: self.defined_at,
            tag: self.tag,
            attributes: self.attributes.into_cloneable_owned(),
            children: self.children.into_owned(),
        }
    }
}

/// The retained view state for an HTML element.
pub struct ElementState<At, Ch> {
    pub(crate) el: crate::view_tree::renderer::types::Element,
    pub(crate) attrs: At,
    pub(crate) children: Option<Ch>,
}

impl<At, Ch> Deref for ElementState<At, Ch> {
    type Target = crate::view_tree::renderer::types::Element;

    fn deref(&self) -> &Self::Target {
        &self.el
    }
}

impl<At, Ch> Mountable for ElementState<At, Ch> {
    fn unmount(&mut self) {
        Rndr::remove(&self.el);
    }

    fn mount(
        &mut self,
        parent: &crate::view_tree::renderer::types::Element,
        marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
        Rndr::insert_node(parent, &self.el, marker);
    }

    fn insert_before_this(&self, child: &mut dyn Mountable) -> bool {
        // codegen optimisation:
        fn inner(
            element: &crate::view_tree::renderer::types::Element,
            child: &mut dyn Mountable,
        ) -> bool {
            if let Some(parent) = Rndr::get_parent(element)
                .and_then(crate::view_tree::renderer::types::Element::cast_from)
            {
                child.mount(&parent, Some(element));
                true
            } else {
                false
            }
        }
        inner(&self.el, child)
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        // codegen optimisation:
        fn inner(
            element: &crate::view_tree::renderer::types::Element,
        ) -> Vec<crate::view_tree::renderer::types::Element> {
            vec![element.clone()]
        }
        inner(&self.el)
    }
}
