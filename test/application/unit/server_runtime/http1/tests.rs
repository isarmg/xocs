use super::*;
use tokio::io::AsyncWriteExt;

#[tokio::test(start_paused = true)]
async fn write_idle_timeout_interrupts_a_stalled_peer() {
    let (stream, _peer) = tokio::io::duplex(1);
    let mut stream = WriteIdleTimeout::new(stream, Duration::from_secs(30));
    let error = stream.write_all(b"ab").await.unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
}

#[test]
fn accept_backoff_is_bounded_and_resets() {
    let mut backoff = AcceptBackoff::default();
    assert_eq!(backoff.failure_delay(), Duration::from_millis(50));
    assert_eq!(backoff.failure_delay(), Duration::from_millis(100));
    for _ in 0..20 {
        assert!(backoff.failure_delay() <= Duration::from_secs(1));
    }
    backoff.reset();
    assert_eq!(backoff.failure_delay(), Duration::from_millis(50));
}

#[tokio::test]
async fn backoff_is_interrupted_by_shutdown() {
    let stop = CancellationToken::new();
    stop.cancel();
    assert!(
        !wait_after_accept_error(
            "example-product",
            "test",
            &stop,
            &mut AcceptBackoff::default(),
            &io::Error::from_raw_os_error(24)
        )
        .await
    );
}
