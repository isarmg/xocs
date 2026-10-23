#[inline(always)]
#[cfg(test)]
pub(crate) const fn pack(state: State, action: Action) -> u8 {
    ((action as u8) << 4) | state as u8
}
