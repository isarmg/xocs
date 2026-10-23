use crate::error::ParserError;
use crate::stream::Stream;
use crate::Parser;

/// Preserve a named parser boundary without runtime tracing.
///
/// This product uses structured parse errors through [`Parser::context`];
/// terminal trace rendering is outside its maintained parser contract.
///
/// Note that [`Parser::context`] also provides high level trace information.
///
///
/// # Example
///
/// ```rust
/// # use winnow::{error::ErrMode, error::Needed};
/// # use winnow::token::take_while;
/// # use winnow::stream::AsChar;
/// # use winnow::prelude::*;
/// use winnow::combinator::trace;
///
/// fn short_alpha<'s>(s: &mut &'s [u8]) -> ModalResult<&'s [u8]> {
///   trace("short_alpha",
///     take_while(3..=6, AsChar::is_alpha)
///   ).parse_next(s)
/// }
///
/// assert_eq!(short_alpha.parse_peek(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(short_alpha.parse_peek(b"lengthy"), Ok((&b"y"[..], &b"length"[..])));
/// assert_eq!(short_alpha.parse_peek(b"latin"), Ok((&b""[..], &b"latin"[..])));
/// assert!(short_alpha.parse_peek(b"ed").is_err());
/// assert!(short_alpha.parse_peek(b"12345").is_err());
/// ```
#[inline(always)]
pub fn trace<I: Stream, O, E: ParserError<I>>(
    _name: impl core::fmt::Display,
    parser: impl Parser<I, O, E>,
) -> impl Parser<I, O, E> {
    parser
}

#[inline(always)]
pub(crate) fn trace_result<T, I: Stream, E: ParserError<I>>(
    _name: impl core::fmt::Display,
    _result: &Result<T, E>,
) {
}

pub(crate) struct DisplayDebug<D>(pub(crate) D);

impl<D: core::fmt::Debug> core::fmt::Display for DisplayDebug<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
