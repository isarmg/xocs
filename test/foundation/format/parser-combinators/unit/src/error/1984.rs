#[cfg(test)]
pub(crate) type TestResult<I, O> = ModalResult<O, InputError<I>>;
