//! Consumer regressions for the maintained HTTP body's scheduling contract.

use bytes::Bytes;
use http_body::{Body, BodyExt, Frame, Full, Limited, StreamBody};
use std::{
    convert::Infallible,
    future::Future,
    pin::{Pin, pin},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

#[derive(Default)]
struct WakeCounter(AtomicUsize);

impl Wake for WakeCounter {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

struct ReadyFrames {
    remaining: usize,
    polls: Arc<AtomicUsize>,
    trailers: bool,
}

impl Body for ReadyFrames {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        self.polls.fetch_add(1, Ordering::Relaxed);
        if self.remaining == 0 {
            return Poll::Ready(None);
        }
        self.remaining -= 1;
        Poll::Ready(Some(Ok(if self.trailers {
            Frame::trailers(http::HeaderMap::new())
        } else {
            Frame::data(Bytes::new())
        })))
    }
}

#[test]
fn collection_yields_and_wakes_for_ready_frames_even_without_data_bytes() {
    for trailers in [false, true] {
        let polls = Arc::new(AtomicUsize::new(0));
        let wake = Arc::new(WakeCounter::default());
        let waker = Waker::from(wake.clone());
        let mut cx = Context::from_waker(&waker);
        let mut collection = pin!(
            Limited::new(
                ReadyFrames {
                    remaining: 4096,
                    polls: polls.clone(),
                    trailers,
                },
                0,
            )
            .collect()
        );

        assert!(collection.as_mut().poll(&mut cx).is_pending());
        assert!((1..4096).contains(&polls.load(Ordering::Relaxed)));
        assert_eq!(wake.0.load(Ordering::Relaxed), 1);

        // The wake must allow the consumer to finish without losing progress.
        let collected = futures::executor::block_on(collection).unwrap();
        assert_eq!(collected.trailers().is_some(), trailers);
        assert!(collected.to_bytes().is_empty());
    }
}

#[tokio::test(start_paused = true)]
async fn bounded_axum_collection_can_be_cancelled_at_an_elapsed_deadline() {
    let body = axum::body::Body::new(ReadyFrames {
        remaining: 4096,
        polls: Arc::new(AtomicUsize::new(0)),
        trailers: false,
    });
    let result =
        tokio::time::timeout_at(tokio::time::Instant::now(), axum::body::to_bytes(body, 0)).await;

    assert!(
        result.is_err(),
        "ready empty frames must not bypass the deadline"
    );
}

#[tokio::test]
async fn collection_preserves_data_and_trailers_across_yields() {
    let data = (0..=255).map(|byte| Frame::data(Bytes::from(vec![byte])));
    let mut trailers = http::HeaderMap::new();
    trailers.insert("x-content-check", "complete".parse().unwrap());
    let frames = data
        .chain([Frame::trailers(trailers.clone())])
        .map(Ok::<_, Infallible>);
    let body = StreamBody::new(futures::stream::iter(frames));
    let collected = Limited::new(body, 256).collect().await.unwrap();

    assert_eq!(collected.trailers(), Some(&trailers));
    assert_eq!(
        collected.to_bytes().as_ref(),
        &(0..=255).collect::<Vec<u8>>()
    );
}

#[test]
fn a_small_ready_body_finishes_without_a_spurious_yield() {
    let wake = Arc::new(WakeCounter::default());
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut collection = pin!(Full::new(Bytes::from_static(b"hello")).collect());

    let Poll::Ready(Ok(collected)) = collection.as_mut().poll(&mut cx) else {
        panic!("a small ready body should finish in one poll");
    };
    assert_eq!(collected.to_bytes(), "hello");
    assert_eq!(wake.0.load(Ordering::Relaxed), 0);
}
