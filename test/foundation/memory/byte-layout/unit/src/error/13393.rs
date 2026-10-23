#[cfg(test)]
impl<Src, Dst> AlignmentError<Src, Dst> {
    // A convenience constructor so that test code doesn't need to write
    // `unsafe`.
    fn new_checked(src: Src) -> AlignmentError<Src, Dst> {
        assert_ne!(core::mem::align_of::<Dst>(), 1);
        // SAFETY: The preceding assertion guarantees that `Dst`'s alignment
        // requirement is greater than one.
        unsafe { AlignmentError::new_unchecked(src) }
    }
}
