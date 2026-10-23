/// Compare corresponding bytes in `x` and `y` for equality.
///
/// That is, this returns true if and only if `x.len() == y.len()` and
/// `x[i] == y[i]` for all `0 <= i < x.len()`.
///
/// Note that this isn't used. We only use it in tests as a convenient way
/// of testing `is_equal_raw`.
///
/// # Inlining
///
/// This routine is marked `inline(always)`. If you want to call this function
/// in a way that is not always inlined, you'll need to wrap a call to it in
/// another function that is marked as `inline(never)` or just `inline`.
///
/// # Motivation
///
/// Why not use slice equality instead? Well, slice equality usually results in
/// a call out to the current platform's `libc` which might not be inlineable
/// or have other overhead. This routine isn't guaranteed to be a win, but it
/// might be in some cases.
#[cfg(test)]
#[inline(always)]
fn is_equal(x: &[u8], y: &[u8]) -> bool {
    if x.len() != y.len() {
        return false;
    }
    // SAFETY: Our pointers are derived directly from borrowed slices which
    // uphold all of our safety guarantees except for length. We account for
    // length with the check above.
    unsafe { is_equal_raw(x.as_ptr(), y.as_ptr(), x.len()) }
}
