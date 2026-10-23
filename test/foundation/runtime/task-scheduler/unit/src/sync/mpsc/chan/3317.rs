#[cfg(all(test, not(loom)))]
pub(crate) fn channel_from_index<T, S: Semaphore>(
    start_index: usize,
    semaphore: S,
) -> (Tx<T, S>, Rx<T, S>) {
    let (tx, rx) = list::channel_from_index(start_index);
    channel_from_list(tx, rx, semaphore)
}
