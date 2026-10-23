#[cfg(test)]
thread_local! {
    static DROP_INVOCATIONS: Cell<usize> = const { Cell::new(0) };
}
