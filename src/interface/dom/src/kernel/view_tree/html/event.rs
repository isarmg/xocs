use crate::thread_bound::SendWrapper;
use crate::view_tree::{
    html::attribute::{
        maybe_next_attr_erasure_macros::next_attr_combine, Attribute, NamedAttributeKey,
    },
    renderer::{CastFrom, RemoveEventHandler, Rndr},
};
use std::{
    borrow::Cow,
    cell::RefCell,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use wasm_bindgen::convert::FromWasmAbi;

/// A cloneable event callback.
pub type SharedEventCallback<E> = Rc<RefCell<dyn FnMut(E)>>;

/// A function that can be called in response to an event.
pub trait EventCallback<E>: 'static {
    /// Runs the event handler.
    fn invoke(&mut self, event: E);

    /// Converts this into a cloneable/shared event handler.
    fn into_shared(self) -> SharedEventCallback<E>;
}

impl<E: 'static> EventCallback<E> for SharedEventCallback<E> {
    fn invoke(&mut self, event: E) {
        let mut fun = self.borrow_mut();
        fun(event)
    }

    fn into_shared(self) -> SharedEventCallback<E> {
        self
    }
}

impl<F, E> EventCallback<E> for F
where
    F: FnMut(E) + 'static,
{
    fn invoke(&mut self, event: E) {
        self(event)
    }

    fn into_shared(self) -> SharedEventCallback<E> {
        Rc::new(RefCell::new(self))
    }
}

/// An event listener with a typed event target.
pub struct Targeted<E, T> {
    event: E,
    el_ty: PhantomData<T>,
}

impl<E, T> Targeted<E, T> {
    /// Returns the inner event.
    pub fn into_inner(self) -> E {
        self.event
    }

    /// Returns the event's target, as an HTML element of the correct type.
    pub fn target(&self) -> T
    where
        T: CastFrom<crate::view_tree::renderer::types::Element>,

        crate::view_tree::renderer::types::Event: From<E>,
        E: Clone,
    {
        let ev = crate::view_tree::renderer::types::Event::from(self.event.clone());
        Rndr::event_target(&ev)
    }
}

impl<E, T> Deref for Targeted<E, T> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<E, T> DerefMut for Targeted<E, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

impl<E, T> From<E> for Targeted<E, T> {
    fn from(event: E) -> Self {
        Targeted {
            event,
            el_ty: PhantomData,
        }
    }
}

/// Creates an [`Attribute`] that will add an event listener to an element.
pub fn on<E, F>(event: E, cb: F) -> On<E, F>
where
    F: FnMut(E::EventType) + 'static,
    E: EventDescriptor + Send + 'static,
    E::EventType: 'static,
    E::EventType: From<crate::view_tree::renderer::types::Event>,
{
    On {
        event,

        owner: crate::reactivity::owner::Owner::current().unwrap_or_default(),
        cb: SendWrapper::new(cb),
    }
}

/// Creates an [`Attribute`] that will add an event listener with a typed target to an element.
#[allow(clippy::type_complexity)]
pub fn on_target<E, T, F>(event: E, mut cb: F) -> On<E, Box<dyn FnMut(E::EventType)>>
where
    T: HasElementType,
    F: FnMut(Targeted<E::EventType, <T as HasElementType>::ElementType>) + 'static,
    E: EventDescriptor + Send + 'static,
    E::EventType: 'static,

    E::EventType: From<crate::view_tree::renderer::types::Event>,
{
    on(event, Box::new(move |ev: E::EventType| cb(ev.into())))
}

/// An [`Attribute`] that adds an event listener to an element.
pub struct On<E, F> {
    event: E,

    owner: crate::reactivity::owner::Owner,
    cb: SendWrapper<F>,
}

impl<E, F> Clone for On<E, F>
where
    E: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            event: self.event.clone(),

            owner: self.owner.clone(),
            cb: self.cb.clone(),
        }
    }
}

impl<E, F> On<E, F>
where
    F: EventCallback<E::EventType>,
    E: EventDescriptor + Send + 'static,
    E::EventType: 'static,
    E::EventType: From<crate::view_tree::renderer::types::Event>,
{
    /// Attaches the event listener to the element.
    pub fn attach(self, el: &crate::view_tree::renderer::types::Element) -> RemoveEventHandler {
        let mut cb = self.cb.take();

        let cb = Box::new(move |ev: crate::view_tree::renderer::types::Event| {
            #[cfg(debug_assertions)]
            let _rx_guard = crate::reactivity::diagnostics::SpecialNonReactiveZone::enter();

            let ev = E::EventType::from(ev);

            self.owner.with(|| cb.invoke(ev));
        }) as Box<dyn FnMut(crate::view_tree::renderer::types::Event)>;

        Rndr::add_event_listener(el, &self.event.name(), cb)
    }

    /// Attaches the event listener to the element as a listener that is triggered during the capture phase,
    /// meaning it will fire before any event listeners further down in the DOM.
    pub fn attach_capture(
        self,
        el: &crate::view_tree::renderer::types::Element,
    ) -> RemoveEventHandler {
        fn attach_inner(
            el: &crate::view_tree::renderer::types::Element,
            cb: Box<dyn FnMut(crate::view_tree::renderer::types::Event)>,
            name: Cow<'static, str>,
        ) -> RemoveEventHandler {
            Rndr::add_event_listener_use_capture(el, &name, cb)
        }

        let mut cb = self.cb.take();

        let cb = Box::new(move |ev: crate::view_tree::renderer::types::Event| {
            #[cfg(debug_assertions)]
            let _rx_guard = crate::reactivity::diagnostics::SpecialNonReactiveZone::enter();

            let ev = E::EventType::from(ev);

            self.owner.with(|| cb.invoke(ev));
        }) as Box<dyn FnMut(crate::view_tree::renderer::types::Event)>;

        attach_inner(el, cb, self.event.name())
    }
}

impl<E, F> Debug for On<E, F>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("On").field(&self.event).finish()
    }
}

impl<E, F> Attribute for On<E, F>
where
    F: EventCallback<E::EventType>,
    E: EventDescriptor + Send + 'static,
    E::EventType: 'static,

    E::EventType: From<crate::view_tree::renderer::types::Event>,
{
    // a function that can be called once to remove the event listener
    type State = (
        crate::view_tree::renderer::types::Element,
        Option<RemoveEventHandler>,
    );
    type Cloneable = On<E, SharedEventCallback<E::EventType>>;
    type CloneableOwned = On<E, SharedEventCallback<E::EventType>>;

    #[inline(always)]
    fn build(self, el: &crate::view_tree::renderer::types::Element) -> Self::State {
        let cleanup = if E::CAPTURE {
            self.attach_capture(el)
        } else {
            self.attach(el)
        };
        (el.clone(), Some(cleanup))
    }

    #[inline(always)]
    fn rebuild(self, state: &mut Self::State) {
        let (el, prev_cleanup) = state;
        if let Some(prev) = prev_cleanup.take() {
            if let Some(remove) = prev.into_inner() {
                remove();
            }
        }
        *prev_cleanup = Some(if E::CAPTURE {
            self.attach_capture(el)
        } else {
            self.attach(el)
        });
    }

    fn into_cloneable(self) -> Self::Cloneable {
        On {
            cb: SendWrapper::new(self.cb.take().into_shared()),

            owner: self.owner,
            event: self.event,
        }
    }

    fn into_cloneable_owned(self) -> Self::CloneableOwned {
        On {
            cb: SendWrapper::new(self.cb.take().into_shared()),

            owner: self.owner,
            event: self.event,
        }
    }

    fn keys(&self) -> Vec<NamedAttributeKey> {
        vec![]
    }
}

impl<E, F> NextAttribute for On<E, F>
where
    F: EventCallback<E::EventType>,
    E: EventDescriptor + Send + 'static,
    E::EventType: 'static,

    E::EventType: From<crate::view_tree::renderer::types::Event>,
{
    next_attr_output_type!(Self, NewAttr);

    fn add_any_attr<NewAttr: Attribute>(self, new_attr: NewAttr) -> Self::Output<NewAttr> {
        next_attr_combine!(self, new_attr)
    }
}

/// A trait for converting types into [web_sys events](web_sys).
pub trait EventDescriptor: Clone {
    /// The [`web_sys`] event type, such as [`web_sys::MouseEvent`].
    type EventType: FromWasmAbi;

    /// Indicates if this event should be handled during the capture phase.
    const CAPTURE: bool = false;

    /// The name of the event, such as `click` or `mouseover`.
    fn name(&self) -> Cow<'static, str>;
}

/// A wrapper that tells the framework to handle an event during the capture phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture<E> {
    inner: E,
}

/// Wraps an event to indicate that it should be handled during the capture phase.
pub fn capture<E>(event: E) -> Capture<E> {
    Capture { inner: event }
}

impl<E: EventDescriptor> EventDescriptor for Capture<E> {
    type EventType = E::EventType;

    const CAPTURE: bool = true;

    fn name(&self) -> Cow<'static, str> {
        self.inner.name()
    }
}

/// A custom event.
#[derive(Debug)]
pub struct Custom<E: FromWasmAbi = web_sys::Event> {
    name: Cow<'static, str>,
    _event_type: PhantomData<fn() -> E>,
}

impl<E: FromWasmAbi> Clone for Custom<E> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            _event_type: PhantomData,
        }
    }
}

impl<E: FromWasmAbi> EventDescriptor for Custom<E> {
    type EventType = E;

    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}

impl<E: FromWasmAbi> Custom<E> {
    /// Creates a custom event type that can be used within
    /// [`OnAttribute::on`](crate::view_tree::prelude::OnAttribute::on), for events
    /// which are not covered in the [`ev`](crate::view_tree::html::event) module.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            _event_type: PhantomData,
        }
    }
}

macro_rules! generate_event_types {
  {$(
    $( $event:ident )+ : $web_event:ident
  ),* $(,)?} => {
    blog_component_macros::paste! {
      $(
        #[doc = "The `" [< $($event)+ >] "` event, which receives [" $web_event "](web_sys::" $web_event ") as its argument."]
        #[derive(Copy, Clone, Debug)]
        #[allow(non_camel_case_types)]
        pub struct [<$( $event )+ >];

        impl EventDescriptor for [< $($event)+ >] {
          type EventType = web_sys::$web_event;

          #[inline(always)]
          fn name(&self) -> Cow<'static, str> {
            stringify!([< $($event)+ >]).into()
          }

        }
      )*
    }
  };

}

generate_event_types! {
  // =========================================================
  // WindowEventHandlersEventMap
  // =========================================================
  after print: Event,
  before print: Event,
  before unload: BeforeUnloadEvent,
  gamepad connected: GamepadEvent,
  gamepad disconnected: GamepadEvent,
  hash change: HashChangeEvent,
  language change: Event,
  message: MessageEvent,
  message error: MessageEvent,
  offline: Event,
  online: Event,
  page hide: PageTransitionEvent,
  page show: PageTransitionEvent,
  pop state: PopStateEvent,
  rejection handled: PromiseRejectionEvent,
  storage: StorageEvent,
  unhandled rejection: PromiseRejectionEvent,
  unload: Event,

  // =========================================================
  // GlobalEventHandlersEventMap
  // =========================================================
  abort: UiEvent,
  animation cancel: AnimationEvent,
  animation end: AnimationEvent,
  animation iteration: AnimationEvent,
  animation start: AnimationEvent,
  aux click: MouseEvent,
  before input: InputEvent,
  before toggle: Event, // web_sys does not include `ToggleEvent`
  blur: FocusEvent,
  can play: Event,
  can play through: Event,
  change: Event,
  click: MouseEvent,
  close: Event,
  composition end: CompositionEvent,
  composition start: CompositionEvent,
  composition update: CompositionEvent,
  context menu: MouseEvent,
  cue change: Event,
  dbl click: MouseEvent,
  drag: DragEvent,
  drag end: DragEvent,
  drag enter: DragEvent,
  drag leave: DragEvent,
  drag over: DragEvent,
  drag start: DragEvent,
  drop: DragEvent,
  duration change: Event,
  emptied: Event,
  ended: Event,
  error: ErrorEvent,
  focus: FocusEvent,
  focus in: FocusEvent,
  focus out: FocusEvent,
  form data: Event, // web_sys does not include `FormDataEvent`
  got pointer capture: PointerEvent,
  input: Event,
  invalid: Event,
  key down: KeyboardEvent,
  key press: KeyboardEvent,
  key up: KeyboardEvent,
  load: Event,
  loaded data: Event,
  loaded metadata: Event,
  load start: Event,
  lost pointer capture: PointerEvent,
  mouse down: MouseEvent,
  mouse enter: MouseEvent,
  mouse leave: MouseEvent,
  mouse move: MouseEvent,
  mouse out: MouseEvent,
  mouse over: MouseEvent,
  mouse up: MouseEvent,
  pause: Event,
  play: Event,
  playing: Event,
  pointer cancel: PointerEvent,
  pointer down: PointerEvent,
  pointer enter: PointerEvent,
  pointer leave: PointerEvent,
  pointer move: PointerEvent,
  pointer out: PointerEvent,
  pointer over: PointerEvent,
  pointer up: PointerEvent,
  progress: ProgressEvent,
  rate change: Event,
  reset: Event,
  resize: UiEvent,
  scroll: Event,
  scroll end: Event,
  security policy violation: SecurityPolicyViolationEvent,
  seeked: Event,
  seeking: Event,
  select: Event,
  selection change: Event,
  select start: Event,
  slot change: Event,
  stalled: Event,
  submit: SubmitEvent,
  suspend: Event,
  time update: Event,
  toggle: Event,
  touch cancel: TouchEvent,
  touch end: TouchEvent,
  touch move: TouchEvent,
  touch start: TouchEvent,
  transition cancel: TransitionEvent,
  transition end: TransitionEvent,
  transition run: TransitionEvent,
  transition start: TransitionEvent,
  volume change: Event,
  waiting: Event,
  webkit animation end: Event,
  webkit animation iteration: Event,
  webkit animation start: Event,
  webkit transition end: Event,
  wheel: WheelEvent,

  // =========================================================
  // WindowEventMap
  // =========================================================
  D O M Content Loaded: Event, // Hack for correct casing
  device motion: DeviceMotionEvent,
  device orientation: DeviceOrientationEvent,
  orientation change: Event,

  // =========================================================
  // DocumentAndElementEventHandlersEventMap
  // =========================================================
  copy: ClipboardEvent,
  cut: ClipboardEvent,
  paste: ClipboardEvent,

  // =========================================================
  // DocumentEventMap
  // =========================================================
  fullscreen change: Event,
  fullscreen error: Event,
  pointer lock change: Event,
  pointer lock error: Event,
  ready state change: Event,
  visibility change: Event,
}

// Export `web_sys` event types
use super::{
    attribute::{maybe_next_attr_erasure_macros::next_attr_output_type, NextAttribute},
    element::HasElementType,
};
#[doc(no_inline)]
pub use web_sys::{
    AnimationEvent, BeforeUnloadEvent, ClipboardEvent, CompositionEvent, CustomEvent,
    DeviceMotionEvent, DeviceOrientationEvent, DragEvent, ErrorEvent, Event, FocusEvent,
    GamepadEvent, HashChangeEvent, InputEvent, KeyboardEvent, MessageEvent, MouseEvent,
    PageTransitionEvent, PointerEvent, PopStateEvent, ProgressEvent, PromiseRejectionEvent,
    SecurityPolicyViolationEvent, StorageEvent, SubmitEvent, TouchEvent, TransitionEvent, UiEvent,
    WheelEvent,
};
