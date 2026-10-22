use crate::reactivity::{
    graph::{AnySubscriber, Observer, WithObserver},
    owner::Owner,
};
use futures::Future;
use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    /// A [`Future`] wrapper that sets the [`Owner`] and [`Observer`] before polling the inner
    /// `Future`.
    #[derive(Clone)]
    #[allow(missing_docs)]
    pub struct ScopedFuture<Fut> {
        pub owner: Owner,
        pub observer: Option<AnySubscriber>,
        #[pin]
        pub fut: Fut,
    }
}

impl<Fut> ScopedFuture<Fut> {
    /// Wraps the given `Future` by taking the current [`Owner`] and [`Observer`] and re-setting
    /// them as the active owner and observer every time the inner `Future` is polled.
    pub fn new(fut: Fut) -> Self {
        let owner = Owner::current().unwrap_or_default();
        let observer = Observer::get();
        Self {
            owner,
            observer,
            fut,
        }
    }

    /// Wraps the given `Future` by taking the current [`Owner`] re-setting it as the
    /// active owner every time the inner `Future` is polled. Always untracks, i.e., clears
    /// the active [`Observer`] when polled.
    pub fn new_untracked(fut: Fut) -> Self {
        let owner = Owner::current().unwrap_or_default();
        Self {
            owner,
            observer: None,
            fut,
        }
    }

    #[doc(hidden)]
    #[track_caller]
    pub fn new_untracked_with_diagnostics(fut: Fut) -> ScopedFutureUntrackedWithDiagnostics<Fut> {
        let owner = Owner::current().unwrap_or_default();
        ScopedFutureUntrackedWithDiagnostics {
            owner,
            observer: None,
            fut,
        }
    }
}

impl<Fut: Future> Future for ScopedFuture<Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.owner.with(|| {
            #[cfg(debug_assertions)]
            let _maybe_guard = if this.observer.is_none() {
                Some(crate::reactivity::diagnostics::SpecialNonReactiveZone::enter())
            } else {
                None
            };
            this.observer.with_observer(|| this.fut.poll(cx))
        })
    }
}

pin_project! {
    /// A [`Future`] wrapper that sets the [`Owner`] and [`Observer`] before polling the inner
    /// `Future`, output of [`ScopedFuture::new_untracked_with_diagnostics`].
    ///
    /// In leptos 0.9 this will be replaced with `ScopedFuture` itself.
    #[derive(Clone)]
    pub struct ScopedFutureUntrackedWithDiagnostics<Fut> {
        owner: Owner,
        observer: Option<AnySubscriber>,
        #[pin]
        fut: Fut,
    }
}

impl<Fut: Future> Future for ScopedFutureUntrackedWithDiagnostics<Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.owner
            .with(|| this.observer.with_observer(|| this.fut.poll(cx)))
    }
}
