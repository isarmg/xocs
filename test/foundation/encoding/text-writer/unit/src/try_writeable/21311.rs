#[cfg(test)]
struct DelegatedTryMessage<'s>(Result<&'s str, usize>);
