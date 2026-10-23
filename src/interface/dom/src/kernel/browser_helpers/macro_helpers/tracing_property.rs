use wasm_bindgen::UnwrapThrowExt;

#[macro_export]
/// Use for tracing property
macro_rules! tracing_props {
    () => {
        ::leptos::tracing::span!(
            ::leptos::tracing::Level::TRACE,
            "blog_dom::browser_helpers::tracing_props",
            props = String::from("[]")
        );
    };
    ($($prop:tt),+ $(,)?) => {
        {
            use ::leptos::leptos_dom::macro_helpers::tracing_property::{Match, SerializeMatch, DefaultMatch};
            let mut props = String::from('[');
            $(
                let prop = (&&Match {
                    name: stringify!{$prop},
                    value: std::cell::Cell::new(Some(&$prop))
                }).spez();
                props.push_str(&format!("{prop},"));
            )*
            props.pop();
            props.push(']');
            ::leptos::tracing::span!(
                ::leptos::tracing::Level::TRACE,
                "blog_dom::browser_helpers::tracing_props",
                props
            );
        }
    };
}

// Implementation based on spez
// see https://github.com/m-ou-se/spez

pub struct Match<T> {
    pub name: &'static str,
    pub value: std::cell::Cell<Option<T>>,
}

pub trait SerializeMatch {
    type Return;
    fn spez(&self) -> Self::Return;
}
impl<T: serde::Serialize> SerializeMatch for &Match<&T> {
    type Return = String;
    fn spez(&self) -> Self::Return {
        let name = self.name;

        // suppresses warnings when serializing signals into props
        #[cfg(debug_assertions)]
        let _z = crate::reactivity::diagnostics::SpecialNonReactiveZone::enter();

        serde_json::to_string(self.value.get().unwrap_throw()).map_or_else(
            |err| format!(r#"{{"name": "{name}", "error": "{err}"}}"#),
            |value| format!(r#"{{"name": "{name}", "value": {value}}}"#),
        )
    }
}

pub trait DefaultMatch {
    type Return;
    fn spez(&self) -> Self::Return;
}
impl<T> DefaultMatch for Match<&T> {
    type Return = String;
    fn spez(&self) -> Self::Return {
        let name = self.name;
        format!(r#"{{"name": "{name}", "value": "[unserializable value]"}}"#)
    }
}

#[cfg(test)]
include!("../../../../../../../test/interface/dom/unit/kernel/browser_helpers/macro_helpers/tracing_property/match_primitive.rs");

#[cfg(test)]
include!("../../../../../../../test/interface/dom/unit/kernel/browser_helpers/macro_helpers/tracing_property/match_serialize.rs");

#[cfg(test)]
include!("../../../../../../../test/interface/dom/unit/kernel/browser_helpers/macro_helpers/tracing_property/match_no_serialize.rs");
