use super::{ReactiveFunction, SharedReactiveFunction};
use crate::reactivity::effect::RenderEffect;
use crate::view_tree::html::element::InnerHtmlValue;

impl<F, V> InnerHtmlValue for F
where
    F: ReactiveFunction<Output = V>,
    V: InnerHtmlValue + 'static,
    V::State: 'static,
{
    type State = RenderEffect<V::State>;
    type Cloneable = SharedReactiveFunction<V>;
    type CloneableOwned = SharedReactiveFunction<V>;

    fn build(mut self, el: &crate::view_tree::renderer::types::Element) -> Self::State {
        let el = el.to_owned();
        RenderEffect::new(move |prev| {
            let value = self.invoke();
            if let Some(mut state) = prev {
                value.rebuild(&mut state);
                state
            } else {
                value.build(&el)
            }
        })
    }

    fn rebuild(mut self, state: &mut Self::State) {
        let prev_value = state.take_value();
        *state = RenderEffect::new_with_value(
            move |prev| {
                let value = self.invoke();
                if let Some(mut state) = prev {
                    value.rebuild(&mut state);
                    state
                } else {
                    unreachable!()
                }
            },
            prev_value,
        );
    }

    fn into_cloneable(self) -> Self::Cloneable {
        self.into_shared()
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        self.into_shared()
    }
}

macro_rules! inner_html_reactive {
    ($name:ident, <$($gen:ident),*>, $v:ty, $dry_resolve:literal, $( $where_clause:tt )*) =>
    {
        #[allow(deprecated)]
        impl<$($gen),*> InnerHtmlValue for $name<$($gen),*>
        where
            $v: InnerHtmlValue + Clone + Send + Sync + 'static,
            <$v as InnerHtmlValue>::State: 'static,
            $($where_clause)*
        {

            type State = RenderEffect<<$v as InnerHtmlValue>::State>;
            type Cloneable = Self;
            type CloneableOwned = Self;

            fn build(
                self,
                el: &crate::view_tree::renderer::types::Element,
            ) -> Self::State {
                (move || self.get()).build(el)
            }

            fn rebuild(self, state: &mut Self::State) {
                (move || self.get()).rebuild(state)
            }

            fn into_cloneable(self) -> Self::Cloneable {
                self
            }

            fn into_cloneable_owned(self) -> Self::CloneableOwned {
                self
            }

        }
    };
}

mod stable {
    use crate::reactivity::{
        computed::{ArcMemo, Memo},
        effect::RenderEffect,
        owner::Storage,
        signal::{ArcReadSignal, ArcRwSignal, ReadSignal, RwSignal},
        traits::Get,
        wrappers::read::{ArcSignal, Signal},
    };
    use crate::view_tree::html::element::InnerHtmlValue;

    inner_html_reactive!(
        RwSignal,
        <V, S>,
        V,
        false,
        RwSignal<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    inner_html_reactive!(
        ReadSignal,
        <V, S>,
        V,
        false,
        ReadSignal<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    inner_html_reactive!(
        Memo,
        <V, S>,
        V,
        true,
        Memo<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    inner_html_reactive!(
        Signal,
        <V, S>,
        V,
        true,
        Signal<V, S>: Get<Value = V>,
        S: Storage<V> + Storage<Option<V>>,
        S: Send + Sync + 'static,
    );
    inner_html_reactive!(ArcRwSignal, <V>, V, false, ArcRwSignal<V>: Get<Value = V>);
    inner_html_reactive!(ArcReadSignal, <V>, V, false, ArcReadSignal<V>: Get<Value = V>);
    inner_html_reactive!(ArcMemo, <V>, V, true, ArcMemo<V>: Get<Value = V>);
    inner_html_reactive!(ArcSignal, <V>, V, true, ArcSignal<V>: Get<Value = V>);
}
