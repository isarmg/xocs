//! Body types.
//!
//! The erased body used by the file service and its optional fallback.

#![allow(missing_docs)]

use bytes::Buf;
use http_body::Body;
use pin_project_lite::pin_project;

macro_rules! body_methods {
    () => {
        #[inline]
        fn poll_frame(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
            self.project().inner.poll_frame(cx)
        }

        #[inline]
        fn is_end_stream(&self) -> bool {
            Body::is_end_stream(&self.inner)
        }

        #[inline]
        fn size_hint(&self) -> http_body::SizeHint {
            Body::size_hint(&self.inner)
        }
    };
}

pin_project! {
    pub struct UnsyncBoxBody<D, E> {
        #[pin]
        pub(crate) inner: http_body::combinators::UnsyncBoxBody<D, E>
    }
}

impl<D, E> Default for UnsyncBoxBody<D, E>
where
    D: Buf + 'static,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<D, E> UnsyncBoxBody<D, E> {
    #[allow(dead_code)]
    pub(crate) fn new(inner: http_body::combinators::UnsyncBoxBody<D, E>) -> Self {
        Self { inner }
    }
}

impl<D, E> Body for UnsyncBoxBody<D, E>
where
    D: Buf,
{
    type Data = D;
    type Error = E;

    body_methods!();
}
