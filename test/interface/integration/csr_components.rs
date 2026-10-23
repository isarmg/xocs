//! Actual facade consumers for the maintained component/props contract.
use blog_interface::{
    children::{ToChildren, TypedChildren},
    context::{Provider, ProviderProps},
    prelude::*,
};
use core::num::NonZeroUsize;

#[derive(PartialEq, Debug)]
struct UserInfo {
    user_id: String,
    email: String,
}
#[derive(PartialEq, Debug)]
struct Admin(bool);

#[component]
fn Component(
    #[prop(optional)] optional: bool,
    #[prop(optional, into)] optional_into: Option<String>,
    #[prop(optional_no_strip)] optional_no_strip: Option<String>,
    #[prop(strip_option)] strip_option: Option<u8>,
    #[prop(default = NonZeroUsize::new(10).unwrap())] default: NonZeroUsize,
    #[prop(into)] into: String,
    impl_trait: impl Fn() -> i32 + 'static,
    #[prop(name = "data")] UserInfo { email, user_id }: UserInfo,
    #[prop(name = "tuple")] (name, id): (String, i32),
    #[prop(name = "tuple_struct")] Admin(is_admin): Admin,
    #[prop(name = "outside_name")] inside_name: i32,
) -> impl IntoView {
    let _ = (
        optional,
        optional_into,
        optional_no_strip,
        strip_option,
        default,
        into,
        impl_trait,
        email,
        user_id,
        name,
        id,
        is_admin,
        inside_name,
    );
}

#[test]
fn defaults_conversion_generics_and_destructured_props() {
    let mut props = ComponentProps::builder()
        .into("")
        .strip_option(9)
        .impl_trait(|| 42)
        .data(UserInfo {
            email: "em@il".into(),
            user_id: "1".into(),
        })
        .tuple(("Joe".into(), 12))
        .tuple_struct(Admin(true))
        .outside_name(1)
        .build();
    assert!(!props.optional);
    assert_eq!(props.optional_into, None);
    assert_eq!(props.optional_no_strip, None);
    assert_eq!(props.strip_option, Some(9));
    assert_eq!(props.default, NonZeroUsize::new(10).unwrap());
    assert_eq!(props.into, "");
    assert_eq!((props.impl_trait)(), 42);
    assert_eq!(props.data.email, "em@il");
    assert_eq!(props.data.user_id, "1");
    assert_eq!(props.tuple, ("Joe".into(), 12));
    assert_eq!(props.tuple_struct, Admin(true));
    assert_eq!(props.outside_name, 1);
    // Optional values can still be set after build, as the former nostrip syntax did.
    props.optional_into = Some("foo".into());
    props.optional_no_strip = Some("bar".into());
    assert_eq!(props.optional_into.as_deref(), Some("foo"));
    assert_eq!(props.optional_no_strip.as_deref(), Some("bar"));
    let _ = Component(props);
}

#[component]
fn WithLifetime<'a>(data: &'a str) -> impl IntoView + use<> {
    let _ = data;
    "static lifetime"
}
#[test]
fn returned_view_does_not_capture_input_lifetime() {
    fn view() -> impl IntoView {
        let value = String::from("non_static_lifetime");
        WithLifetime(WithLifetimeProps::builder().data(&value).build())
    }
    let _ = view();
}

#[component]
fn ReactiveProps(
    #[prop(into)] derived: Signal<String>,
    #[prop(into)] plain: Signal<String>,
    #[prop(into)] signal: Signal<String>,
    #[prop(into)] arc_derived: ArcSignal<usize>,
    #[prop(into)] arc_plain: ArcSignal<usize>,
    #[prop(into)] arc_signal: ArcSignal<usize>,
    #[prop(into, optional)] optional: Option<Signal<usize>>,
    #[prop(into, optional)] omitted: Option<Signal<usize>>,
    #[prop(into, strip_option)] stripped: Option<Signal<usize>>,
) -> impl IntoView {
    let _ = (
        derived,
        plain,
        signal,
        arc_derived,
        arc_plain,
        arc_signal,
        optional,
        omitted,
        stripped,
    );
}

#[test]
fn reactive_closure_plain_and_prebuilt_signal_conversions() {
    let owner = Owner::new();
    owner.with(|| {
        let props = ReactivePropsProps::builder()
            .derived(|| "derived")
            .plain("plain")
            .signal(Signal::stored("signal"))
            .arc_derived(|| 2)
            .arc_plain(3)
            .arc_signal(ArcSignal::stored(4))
            .optional(|| 5)
            .stripped(|| 6)
            .build();
        assert_eq!(props.derived.get(), "derived");
        assert_eq!(props.plain.get(), "plain");
        assert_eq!(props.signal.get(), "signal");
        assert_eq!(props.arc_derived.get(), 2);
        assert_eq!(props.arc_plain.get(), 3);
        assert_eq!(props.arc_signal.get(), 4);
        assert_eq!(props.optional.unwrap().get(), 5);
        assert!(props.omitted.is_none());
        assert_eq!(props.stripped.unwrap().get(), 6);
    });
}

#[component]
fn CallbackProps(
    #[prop(into)] sync_zero: Callback<(), String>,
    #[prop(into)] sync_one: Callback<usize, String>,
    #[prop(into)] sync_tuple: Callback<(usize,), String>,
    #[prop(into)] sync_many: Callback<(usize, String), String>,
    #[prop(into)] local_zero: UnsyncCallback<(), String>,
    #[prop(into)] local_one: UnsyncCallback<usize, String>,
    #[prop(into)] local_tuple: UnsyncCallback<(usize,), String>,
    #[prop(into)] local_many: UnsyncCallback<(usize, String), String>,
) -> impl IntoView {
    let _ = (
        sync_zero,
        sync_one,
        sync_tuple,
        sync_many,
        local_zero,
        local_one,
        local_tuple,
        local_many,
    );
}

#[test]
fn callback_arity_and_output_conversions() {
    Owner::new().with(|| {
        let p = CallbackPropsProps::builder()
            .sync_zero(|| "zero")
            .sync_one(|_: usize| "one")
            .sync_tuple(|(_n,): (usize,)| "tuple")
            .sync_many(|(_n, _s): (usize, String)| "many")
            .local_zero(|| "zero")
            .local_one(|_: usize| "one")
            .local_tuple(|(_n,): (usize,)| "tuple")
            .local_many(|(_n, _s): (usize, String)| "many")
            .build();
        assert_eq!(p.sync_zero.run(()), "zero");
        assert_eq!(p.sync_one.run(1), "one");
        assert_eq!(p.sync_tuple.run((1,)), "tuple");
        assert_eq!(p.sync_many.run((1, "s".into())), "many");
        assert_eq!(p.local_zero.run(()), "zero");
        assert_eq!(p.local_one.run(1), "one");
        assert_eq!(p.local_tuple.run((1,)), "tuple");
        assert_eq!(p.local_many.run((1, "s".into())), "many");
    });
}

#[component]
fn ChildrenConsumer<V>(children: TypedChildren<V>) -> impl IntoView
where
    V: IntoView + 'static,
{
    children.into_inner()()
}

#[test]
fn children_conversion_and_context_owner_are_preserved() {
    Owner::new().with(|| {
        provide_context(1u8);
        let _ = Provider(
            ProviderProps::builder()
                .value(2u8)
                .children(ToChildren::to_children(|| {
                    assert_eq!(use_context::<u8>(), Some(2));
                    ChildrenConsumer(
                        ChildrenConsumerProps::builder()
                            .children(ToChildren::to_children(|| "child"))
                            .build(),
                    )
                }))
                .build(),
        );
        assert_eq!(use_context::<u8>(), Some(1));
    });
}

mod macro_hygiene {
    mod blog_interface {}
    #[test]
    fn component_uses_absolute_facade_paths() {
        use ::blog_interface::{component, prelude::*};
        #[component]
        fn Local(#[prop(into)] text: String) -> impl IntoView {
            text
        }
        let _ = Local(LocalProps::builder().text("ok").build());
    }
}

#[component]
fn UntrackedRead(value: RwSignal<usize>) -> impl IntoView {
    value.get().to_string()
}

#[component(transparent)]
fn TrackedRead(value: RwSignal<usize>) -> impl IntoView {
    value.get().to_string()
}

#[test]
fn regular_components_untrack_and_transparent_components_keep_the_observer() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    Owner::new().with(|| {
        let value = RwSignal::new(0);
        let untracked_runs = Arc::new(AtomicUsize::new(0));
        let tracked_runs = Arc::new(AtomicUsize::new(0));
        let untracked = Memo::new({
            let runs = Arc::clone(&untracked_runs);
            move |_| {
                let _ = UntrackedRead(UntrackedReadProps::builder().value(value).build());
                runs.fetch_add(1, Ordering::SeqCst)
            }
        });
        let tracked = Memo::new({
            let runs = Arc::clone(&tracked_runs);
            move |_| {
                let _ = TrackedRead(TrackedReadProps::builder().value(value).build());
                runs.fetch_add(1, Ordering::SeqCst)
            }
        });
        assert_eq!(untracked.get(), 0);
        assert_eq!(tracked.get(), 0);
        value.set(1);
        assert_eq!(untracked.get(), 0);
        assert_eq!(tracked.get(), 1);
        assert_eq!(untracked_runs.load(Ordering::SeqCst), 1);
        assert_eq!(tracked_runs.load(Ordering::SeqCst), 2);
    });
}

#[test]
#[should_panic(
    expected = "Portal use_shadow=true requires the `shadow-dom` feature on blog_interface"
)]
fn unsupported_shadow_portal_still_fails_before_mounting() {
    use blog_interface::portal::{Portal, PortalProps};
    let _ = Portal(
        PortalProps::builder()
            .use_shadow(true)
            .children(ToChildren::to_children(|| ()))
            .build(),
    );
}
