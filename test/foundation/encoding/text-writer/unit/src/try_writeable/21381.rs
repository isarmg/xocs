#[cfg(test)]
impl_try_writeable_delegate!(DelegatedTryMessage<'_>, |&self| &self.0, Error = usize);
