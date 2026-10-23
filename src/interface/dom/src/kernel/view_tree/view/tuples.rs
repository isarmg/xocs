use super::{DomView, Mountable, Render};
use crate::view_tree::{html::attribute::Attribute, renderer::Rndr, view::add_attr::AddAnyAttr};

impl Render for () {
    type State = crate::view_tree::renderer::types::Placeholder;

    fn build(self) -> Self::State {
        Rndr::create_placeholder()
    }

    fn rebuild(self, _state: &mut Self::State) {}
}

impl DomView for () {
    type Owned = ();

    const EXISTS: bool = false;

    fn into_owned(self) -> Self::Owned {}
}

impl AddAnyAttr for () {
    type Output<SomeNewAttr: Attribute> = ();

    fn add_any_attr<NewAttr: Attribute>(self, _attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: DomView,
    {
    }
}

impl Mountable for () {
    fn unmount(&mut self) {}

    fn mount(
        &mut self,
        _parent: &crate::view_tree::renderer::types::Element,
        _marker: Option<&crate::view_tree::renderer::types::Node>,
    ) {
    }

    fn insert_before_this(&self, _child: &mut dyn Mountable) -> bool {
        false
    }

    fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
        vec![]
    }
}

impl<A: Render> Render for (A,) {
    type State = A::State;

    fn build(self) -> Self::State {
        self.0.build()
    }

    fn rebuild(self, state: &mut Self::State) {
        self.0.rebuild(state)
    }
}

impl<A> DomView for (A,)
where
    A: DomView,
{
    type Owned = (A::Owned,);

    const EXISTS: bool = A::EXISTS;

    fn into_owned(self) -> Self::Owned {
        (self.0.into_owned(),)
    }
}

impl<A> AddAnyAttr for (A,)
where
    A: AddAnyAttr,
{
    type Output<SomeNewAttr: Attribute> = (A::Output<SomeNewAttr>,);

    fn add_any_attr<NewAttr: Attribute>(self, attr: NewAttr) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: DomView,
    {
        (self.0.add_any_attr(attr),)
    }
}

macro_rules! impl_view_for_tuples {
	($first:ident, $($ty:ident),* $(,)?) => {
		impl<$first, $($ty),*> Render for ($first, $($ty,)*)
		where
			$first: Render,
			$($ty: Render),*,

		{
			type State = ($first::State, $($ty::State,)*);

			fn build(self) -> Self::State {
                #[allow(non_snake_case)]
                let ($first, $($ty,)*) = self;
                (
                    $first.build(),
                    $($ty.build()),*
                )
			}

			fn rebuild(self, state: &mut Self::State) {
				blog_component_macros::paste! {
					let ([<$first:lower>], $([<$ty:lower>],)*) = self;
					let ([<view_ $first:lower>], $([<view_ $ty:lower>],)*) = state;
					[<$first:lower>].rebuild([<view_ $first:lower>]);
					$([<$ty:lower>].rebuild([<view_ $ty:lower>]));*
				}
			}
		}

		impl<$first, $($ty),*> DomView for ($first, $($ty,)*)
		where
			$first: DomView,
			$($ty: DomView),*,

		{

            type Owned = ($first::Owned, $($ty::Owned,)*);
            const EXISTS: bool = $first::EXISTS || $($ty::EXISTS || )* false;

            fn into_owned(self) -> Self::Owned {
                #[allow(non_snake_case)]
                let ($first, $($ty,)*) = self;
                (
                    $first.into_owned(),
                    $($ty.into_owned()),*
                )
            }
		}

		impl<$first, $($ty),*> Mountable for ($first, $($ty,)*) where
			$first: Mountable,
			$($ty: Mountable),*,

		{
			fn unmount(&mut self) {
                #[allow(non_snake_case)] // better macro performance
                let ($first, $($ty,)*) = self;
                $first.unmount();
                $($ty.unmount());*
			}

			fn mount(
				&mut self,
				parent: &crate::view_tree::renderer::types::Element,
				marker: Option<&crate::view_tree::renderer::types::Node>,
			) {
                #[allow(non_snake_case)] // better macro performance
                let ($first, $($ty,)*) = self;
                $first.mount(parent, marker);
                $($ty.mount(parent, marker));*
			}

			fn insert_before_this(&self,
				child: &mut dyn Mountable,
			) -> bool {
                #[allow(non_snake_case)] // better macro performance
                let ($first, $($ty,)*) = self;
                $first.insert_before_this(child)
                $(|| $ty.insert_before_this(child))*
			}

            fn elements(&self) -> Vec<crate::view_tree::renderer::types::Element> {
                #[allow(non_snake_case)] // better macro performance
                let ($first, $($ty,)*) = self;
                $first.elements().into_iter()
                $(.chain($ty.elements()))*
                    .collect()
            }
		}

        impl<$first, $($ty,)*> AddAnyAttr for ($first, $($ty,)*)
        where
			$first: AddAnyAttr,
			$($ty: AddAnyAttr),*,

        {
            type Output<SomeNewAttr: Attribute> = ($first::Output<SomeNewAttr::Cloneable>, $($ty::Output<SomeNewAttr::Cloneable>,)*);

            fn add_any_attr<NewAttr: Attribute>(
                self,
                attr: NewAttr,
            ) -> Self::Output<NewAttr>
            where
                Self::Output<NewAttr>: DomView,
            {
                let shared = attr.into_cloneable();
                #[allow(non_snake_case)] // better macro performance
                let ($first, $($ty,)*) = self;
                ($first.add_any_attr(shared.clone()), $($ty.add_any_attr(shared.clone()),)*)
            }
        }
    };
}

impl_view_for_tuples!(A, B);
impl_view_for_tuples!(A, B, C);
impl_view_for_tuples!(A, B, C, D);
impl_view_for_tuples!(A, B, C, D, E);
impl_view_for_tuples!(A, B, C, D, E, F);
impl_view_for_tuples!(A, B, C, D, E, F, G);
impl_view_for_tuples!(A, B, C, D, E, F, G, H);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_view_for_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
