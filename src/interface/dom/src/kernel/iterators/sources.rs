//! Iterators that are sources (produce elements from parameters,
//! not from another iterator).

use std::fmt;
use std::mem;

/// An iterator that infinitely applies function to value and yields results.
///
/// This `struct` is created by the [`iterate()`](crate::iterators::iterate) function.
/// See its documentation for more.
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iterate<St, F> {
    state: St,
    f: F,
}

impl<St, F> fmt::Debug for Iterate<St, F>
where
    St: fmt::Debug,
{
    debug_fmt_fields!(Iterate, state);
}

impl<St, F> Iterator for Iterate<St, F>
where
    F: FnMut(&St) -> St,
{
    type Item = St;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next_state = (self.f)(&self.state);
        Some(mem::replace(&mut self.state, next_state))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

/// Creates a new iterator that infinitely applies function to value and yields results.
///
/// ```
/// use blog_dom::iterators::iterate;
///
/// blog_dom::iterators::assert_equal(iterate(1, |i| i % 3 + 1).take(5), vec![1, 2, 3, 1, 2]);
/// ```
///
/// **Panics** if compute the next value does.
///
/// ```should_panic
/// # use blog_dom::iterators::iterate;
/// let mut it = iterate(25u32, |x| x - 10).take_while(|&x| x > 10);
/// assert_eq!(it.next(), Some(25)); // `Iterate` holds 15.
/// assert_eq!(it.next(), Some(15)); // `Iterate` holds 5.
/// it.next(); // `5 - 10` overflows.
/// ```
///
/// You can alternatively use [`core::iter::successors`] as it better describes a finite iterator.
pub fn iterate<St, F>(initial_value: St, f: F) -> Iterate<St, F>
where
    F: FnMut(&St) -> St,
{
    Iterate {
        state: initial_value,
        f,
    }
}
