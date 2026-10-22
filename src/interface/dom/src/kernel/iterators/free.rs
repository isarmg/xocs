//! Free functions that create iterator adaptors or call iterator methods.
//!
//! The benefit of free functions is that they accept any [`IntoIterator`] as
//! argument, so the resulting code may be easier to read.

use std::fmt::Display;
use std::iter::{self, Zip};

type VecIntoIter<T> = alloc::vec::IntoIter<T>;

use alloc::string::String;

use crate::iterators::intersperse::{Intersperse, IntersperseWith};
use crate::iterators::Itertools;

pub use crate::iterators::adaptors::{interleave, put_back};

pub use crate::iterators::kmerge_impl::kmerge;
pub use crate::iterators::merge_join::{merge, merge_join_by};

pub use crate::iterators::multipeek_impl::multipeek;

pub use crate::iterators::peek_nth::peek_nth;

pub use crate::iterators::put_back_n_impl::put_back_n;

pub use crate::iterators::rciter_impl::rciter;
pub use crate::iterators::zip_eq_impl::zip_eq;

/// Iterate `iterable` with a particular value inserted between each element.
///
/// [`IntoIterator`] enabled version of [`Iterator::intersperse`].
///
/// ```
/// use blog_dom::iterators::intersperse;
///
/// blog_dom::iterators::assert_equal(intersperse(0..3, 8), vec![0, 8, 1, 8, 2]);
/// ```
pub fn intersperse<I>(iterable: I, element: I::Item) -> Intersperse<I::IntoIter>
where
    I: IntoIterator,
    <I as IntoIterator>::Item: Clone,
{
    Itertools::intersperse(iterable.into_iter(), element)
}

/// Iterate `iterable` with a particular value created by a function inserted
/// between each element.
///
/// [`IntoIterator`] enabled version of [`Iterator::intersperse_with`].
///
/// ```
/// use blog_dom::iterators::intersperse_with;
///
/// let mut i = 10;
/// blog_dom::iterators::assert_equal(intersperse_with(0..3, || { i -= 1; i }), vec![0, 9, 1, 8, 2]);
/// assert_eq!(i, 8);
/// ```
pub fn intersperse_with<I, F>(iterable: I, element: F) -> IntersperseWith<I::IntoIter, F>
where
    I: IntoIterator,
    F: FnMut() -> I::Item,
{
    Itertools::intersperse_with(iterable.into_iter(), element)
}

/// Iterate `iterable` with a running index.
///
/// [`IntoIterator`] enabled version of [`Iterator::enumerate`].
///
/// ```
/// use blog_dom::iterators::enumerate;
///
/// for (i, elt) in enumerate(&[1, 2, 3]) {
///     /* loop body */
///     # let _ = (i, elt);
/// }
/// ```
pub fn enumerate<I>(iterable: I) -> iter::Enumerate<I::IntoIter>
where
    I: IntoIterator,
{
    iterable.into_iter().enumerate()
}

/// Iterate `iterable` in reverse.
///
/// [`IntoIterator`] enabled version of [`Iterator::rev`].
///
/// ```
/// use blog_dom::iterators::rev;
///
/// for elt in rev(&[1, 2, 3]) {
///     /* loop body */
///     # let _ = elt;
/// }
/// ```
pub fn rev<I>(iterable: I) -> iter::Rev<I::IntoIter>
where
    I: IntoIterator,
    I::IntoIter: DoubleEndedIterator,
{
    iterable.into_iter().rev()
}

/// Converts the arguments to iterators and zips them.
///
/// [`IntoIterator`] enabled version of [`Iterator::zip`].
///
/// ## Example
///
/// ```
/// use blog_dom::iterators::zip;
///
/// let mut result: Vec<(i32, char)> = Vec::new();
///
/// for (a, b) in zip(&[1, 2, 3, 4, 5], &['a', 'b', 'c']) {
///     result.push((*a, *b));
/// }
/// assert_eq!(result, vec![(1, 'a'),(2, 'b'),(3, 'c')]);
/// ```
#[deprecated(
    note = "Use [std::iter::zip](https://doc.rust-lang.org/std/iter/fn.zip.html) instead",
    since = "0.10.4"
)]
pub fn zip<I, J>(i: I, j: J) -> Zip<I::IntoIter, J::IntoIter>
where
    I: IntoIterator,
    J: IntoIterator,
{
    i.into_iter().zip(j)
}

/// Takes two iterables and creates a new iterator over both in sequence.
///
/// [`IntoIterator`] enabled version of [`Iterator::chain`].
///
/// ## Example
/// ```
/// use blog_dom::iterators::chain;
///
/// let mut result:Vec<i32> = Vec::new();
///
/// for element in chain(&[1, 2, 3], &[4]) {
///     result.push(*element);
/// }
/// assert_eq!(result, vec![1, 2, 3, 4]);
/// ```
pub fn chain<I, J>(
    i: I,
    j: J,
) -> iter::Chain<<I as IntoIterator>::IntoIter, <J as IntoIterator>::IntoIter>
where
    I: IntoIterator,
    J: IntoIterator<Item = I::Item>,
{
    i.into_iter().chain(j)
}

/// Create an iterator that clones each element from `&T` to `T`.
///
/// [`IntoIterator`] enabled version of [`Iterator::cloned`].
///
/// ```
/// use blog_dom::iterators::cloned;
///
/// assert_eq!(cloned(b"abc").next(), Some(b'a'));
/// ```
pub fn cloned<'a, I, T>(iterable: I) -> iter::Cloned<I::IntoIter>
where
    I: IntoIterator<Item = &'a T>,
    T: Clone + 'a,
{
    iterable.into_iter().cloned()
}

/// Perform a fold operation over the iterable.
///
/// [`IntoIterator`] enabled version of [`Iterator::fold`].
///
/// ```
/// use blog_dom::iterators::fold;
///
/// assert_eq!(fold(&[1., 2., 3.], 0., |a, &b| f32::max(a, b)), 3.);
/// ```
pub fn fold<I, B, F>(iterable: I, init: B, f: F) -> B
where
    I: IntoIterator,
    F: FnMut(B, I::Item) -> B,
{
    iterable.into_iter().fold(init, f)
}

/// Test whether the predicate holds for all elements in the iterable.
///
/// [`IntoIterator`] enabled version of [`Iterator::all`].
///
/// ```
/// use blog_dom::iterators::all;
///
/// assert!(all(&[1, 2, 3], |elt| *elt > 0));
/// ```
pub fn all<I, F>(iterable: I, f: F) -> bool
where
    I: IntoIterator,
    F: FnMut(I::Item) -> bool,
{
    iterable.into_iter().all(f)
}

/// Test whether the predicate holds for any elements in the iterable.
///
/// [`IntoIterator`] enabled version of [`Iterator::any`].
///
/// ```
/// use blog_dom::iterators::any;
///
/// assert!(any(&[0, -1, 2], |elt| *elt > 0));
/// ```
pub fn any<I, F>(iterable: I, f: F) -> bool
where
    I: IntoIterator,
    F: FnMut(I::Item) -> bool,
{
    iterable.into_iter().any(f)
}

/// Return the maximum value of the iterable.
///
/// [`IntoIterator`] enabled version of [`Iterator::max`].
///
/// ```
/// use blog_dom::iterators::max;
///
/// assert_eq!(max(0..10), Some(9));
/// ```
pub fn max<I>(iterable: I) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: Ord,
{
    iterable.into_iter().max()
}

/// Return the minimum value of the iterable.
///
/// [`IntoIterator`] enabled version of [`Iterator::min`].
///
/// ```
/// use blog_dom::iterators::min;
///
/// assert_eq!(min(0..10), Some(0));
/// ```
pub fn min<I>(iterable: I) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: Ord,
{
    iterable.into_iter().min()
}

/// Combine all iterator elements into one `String`, separated by `sep`.
///
/// [`IntoIterator`] enabled version of [`Itertools::join`].
///
/// ```
/// use blog_dom::iterators::join;
///
/// assert_eq!(join(&[1, 2, 3], ", "), "1, 2, 3");
/// ```
pub fn join<I>(iterable: I, sep: &str) -> String
where
    I: IntoIterator,
    I::Item: Display,
{
    iterable.into_iter().join(sep)
}

/// Sort all iterator elements into a new iterator in ascending order.
///
/// [`IntoIterator`] enabled version of [`Itertools::sorted`].
///
/// ```
/// use blog_dom::iterators::sorted;
/// use blog_dom::iterators::assert_equal;
///
/// assert_equal(sorted("rust".chars()), "rstu".chars());
/// ```
pub fn sorted<I>(iterable: I) -> VecIntoIter<I::Item>
where
    I: IntoIterator,
    I::Item: Ord,
{
    iterable.into_iter().sorted()
}

/// Sort all iterator elements into a new iterator in ascending order.
/// This sort is unstable (i.e., may reorder equal elements).
///
/// [`IntoIterator`] enabled version of [`Itertools::sorted_unstable`].
///
/// ```
/// use blog_dom::iterators::sorted_unstable;
/// use blog_dom::iterators::assert_equal;
///
/// assert_equal(sorted_unstable("rust".chars()), "rstu".chars());
/// ```
pub fn sorted_unstable<I>(iterable: I) -> VecIntoIter<I::Item>
where
    I: IntoIterator,
    I::Item: Ord,
{
    iterable.into_iter().sorted_unstable()
}
