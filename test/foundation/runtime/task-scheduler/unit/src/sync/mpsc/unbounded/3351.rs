#[cfg(all(test, not(loom)))]
pub(crate) fn unbounded_channel_from_index<T>(
    start_index: usize,
) -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = chan::channel_from_index(start_index, Semaphore(AtomicUsize::new(0)));

    let tx = UnboundedSender::new(tx);
    let rx = UnboundedReceiver::new(rx);

    (tx, rx)
}
