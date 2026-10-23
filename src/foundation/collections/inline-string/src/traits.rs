use alloc::string::String;
use core::fmt::{self, Write};
use core::num;

use castaway::{match_type, LifetimeFree};

use super::repr::{IntoRepr, Repr};
use crate::{CompactString, ToCompactStringError, UnwrapWithMsg};

/// A trait for converting a value to a `CompactString`.
///
/// This trait is automatically implemented for any type which implements the
/// [`fmt::Display`] trait. As such, [`ToCompactString`] shouldn't be implemented directly:
/// [`fmt::Display`] should be implemented instead, and you get the [`ToCompactString`]
/// implementation for free.
pub trait ToCompactString {
    /// Converts the given value to a [`CompactString`].
    ///
    /// # Panics
    ///
    /// Panics if the system runs out of memory and it cannot hold the whole string,
    /// or if [`Display::fmt()`][core::fmt::Display::fmt] returns an error.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use compact_str::ToCompactString;
    /// # use compact_str::CompactString;
    ///
    /// let i = 5;
    /// let five = CompactString::new("5");
    ///
    /// assert_eq!(i.to_compact_string(), five);
    /// ```
    #[inline]
    #[track_caller]
    fn to_compact_string(&self) -> CompactString {
        self.try_to_compact_string().unwrap_with_msg()
    }

    /// Fallible version of [`ToCompactString::to_compact_string()`]
    ///
    /// This method won't panic if the system is out-of-memory, but return a
    /// [`ReserveError`][crate::ReserveError].
    /// Otherwise it behaves the same as [`ToCompactString::to_compact_string()`].
    fn try_to_compact_string(&self) -> Result<CompactString, ToCompactStringError>;
}

/// # Safety
///
/// * [`CompactString`] does not contain any lifetime
/// * [`CompactString`] is 'static
/// * [`CompactString`] is a container to `u8`, which is `LifetimeFree`.
unsafe impl LifetimeFree for CompactString {}
unsafe impl LifetimeFree for Repr {}

impl ToCompactString for str {
    #[inline]
    fn to_compact_string(&self) -> CompactString {
        CompactString::new(self)
    }

    #[inline]
    fn try_to_compact_string(&self) -> Result<CompactString, ToCompactStringError> {
        Ok(CompactString(Repr::new(self)?))
    }
}

/// # Panics
///
/// In this implementation, the `to_compact_string` method panics if the `Display` implementation
/// returns an error. This indicates an incorrect `Display` implementation since
/// `std::fmt::Write for CompactString` never returns an error itself.
///
/// # Note
///
/// We use the [`castaway`] crate to provide zero-cost specialization for several types, those are:
/// * `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
/// * `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
/// * `NonZeroU*`, `NonZeroI*`
/// * `bool`
/// * `char`
/// * `String`, `CompactString`
/// * `f32`, `f64`
///     * For floats we use [`zmij`] crate which sometimes provides different formatting than [`std`]
impl<T: fmt::Display> ToCompactString for T {
    #[inline]
    fn try_to_compact_string(&self) -> Result<CompactString, ToCompactStringError> {
        let repr = match_type!(self, {
            &u8 as s => s.into_repr()?,
            &i8 as s => s.into_repr()?,
            &u16 as s => s.into_repr()?,
            &i16 as s => s.into_repr()?,
            &u32 as s => s.into_repr()?,
            &i32 as s => s.into_repr()?,
            &u64 as s => s.into_repr()?,
            &i64 as s => s.into_repr()?,
            &u128 as s => s.into_repr()?,
            &i128 as s => s.into_repr()?,
            &usize as s => s.into_repr()?,
            &isize as s => s.into_repr()?,
            &f32 as s => s.into_repr()?,
            &f64 as s => s.into_repr()?,
            &bool as s => s.into_repr()?,
            &char as s => s.into_repr()?,
            &String as s => Repr::new(s)?,
            &CompactString as s => Repr::new(s)?,
            &num::NonZeroU8 as s => s.into_repr()?,
            &num::NonZeroI8 as s => s.into_repr()?,
            &num::NonZeroU16 as s => s.into_repr()?,
            &num::NonZeroI16 as s => s.into_repr()?,
            &num::NonZeroU32 as s => s.into_repr()?,
            &num::NonZeroI32 as s => s.into_repr()?,
            &num::NonZeroU64 as s => s.into_repr()?,
            &num::NonZeroI64 as s => s.into_repr()?,
            &num::NonZeroUsize as s => s.into_repr()?,
            &num::NonZeroIsize as s => s.into_repr()?,
            &num::NonZeroU128 as s => s.into_repr()?,
            &num::NonZeroI128 as s => s.into_repr()?,
            s => {
                let mut c = CompactString::const_new("");
                write!(c, "{}", s)?;
                return Ok(c);
            }
        });

        Ok(CompactString(repr))
    }
}

/// A trait that provides convenience methods for creating a [`CompactString`] from a collection of
/// items. It is implemented for all types that can be converted into an iterator, and that iterator
/// yields types that can be converted into a `str`.
///
/// i.e. `C: IntoIterator<Item = AsRef<str>>`.
///
/// # Concatenate and Join
/// Two methods that this trait provides are `concat_compact(...)` and `join_compact(...)`
/// ```
/// use compact_str::CompactStringExt;
///
/// let words = vec!["☀️", "🌕", "🌑", "☀️"];
///
/// // directly concatenate all the words together
/// let concat = words.iter().concat_compact();
/// assert_eq!(concat, "☀️🌕🌑☀️");
///
/// // join the words, with a separator
/// let join = words.iter().join_compact(" ➡️ ");
/// assert_eq!(join, "☀️ ➡️ 🌕 ➡️ 🌑 ➡️ ☀️");
/// ```
pub trait CompactStringExt {
    /// Concatenates all the items of a collection into a [`CompactString`]
    ///
    /// # Example
    /// ```
    /// use compact_str::CompactStringExt;
    ///
    /// let items = ["hello", " ", "world", "!"];
    /// let compact = items.concat_compact();
    ///
    /// assert_eq!(compact, "hello world!");
    /// ```
    fn concat_compact(self) -> CompactString;

    /// Joins all the items of a collection, placing a separator between them, forming a
    /// [`CompactString`]
    ///
    /// # Example
    /// ```
    /// use compact_str::CompactStringExt;
    ///
    /// let fruits = vec!["apples", "oranges", "bananas"];
    /// let compact = fruits.join_compact(", ");
    ///
    /// assert_eq!(compact, "apples, oranges, bananas");
    /// ```
    fn join_compact<S: AsRef<str>>(self, separator: S) -> CompactString;
}

impl<I, C> CompactStringExt for C
where
    I: AsRef<str>,
    C: IntoIterator<Item = I>,
{
    fn concat_compact(self) -> CompactString {
        self.into_iter()
            .fold(CompactString::const_new(""), |mut s, item| {
                s.push_str(item.as_ref());
                s
            })
    }

    fn join_compact<S: AsRef<str>>(self, separator: S) -> CompactString {
        let mut compact_string = CompactString::const_new("");

        let mut iter = self.into_iter().peekable();
        let sep = separator.as_ref();

        while let Some(item) = iter.next() {
            compact_string.push_str(item.as_ref());
            if iter.peek().is_some() {
                compact_string.push_str(sep);
            }
        }

        compact_string
    }
}

#[path = "../../../../../test/foundation/collections/inline-string/unit/src/traits/tests.rs"]
#[cfg(test)]
mod tests;
