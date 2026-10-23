//! Functions for wrapping text.

use alloc::borrow::{Cow, ToOwned};
use alloc::vec::Vec;

use crate::Options;
use crate::core::{Word, break_words, display_width};
use crate::word_splitters::split_words;

/// Wrap a line of text at a given width.
///
/// The result is a vector of lines. Each line is of type [`Cow<'_,
/// str>`](Cow), which means that the line will borrow from the input
/// `&str` if possible. The lines do not have trailing whitespace,
/// including a final `'\n'`. Please use [`fill()`](crate::fill()) if
/// you need a [`String`] instead.
///
/// The easiest way to use this function is to pass an integer for
/// `width_or_options`:
///
/// ```
/// use blog_text_line_layout::wrap;
///
/// let lines = wrap("Memory safety without garbage collection.", 15);
/// assert_eq!(lines, &[
///     "Memory safety",
///     "without garbage",
///     "collection.",
/// ]);
/// ```
///
/// If you need to customize the wrapping, you can pass an [`Options`]
/// instead of an `usize`:
///
/// ```
/// use blog_text_line_layout::{wrap, Options};
///
/// let options = Options::new(15)
///     .initial_indent("- ")
///     .subsequent_indent("  ");
/// let lines = wrap("Memory safety without garbage collection.", &options);
/// assert_eq!(lines, &[
///     "- Memory safety",
///     "  without",
///     "  garbage",
///     "  collection.",
/// ]);
/// ```
///
/// # Optimal-Fit Wrapping
///
/// By default, `wrap` will try to ensure an even right margin by
/// finding breaks that avoid short lines. We call this an
/// “optimal-fit algorithm” since the line breaks are computed by
/// considering all possible line breaks. The alternative is a
/// “first-fit algorithm” which simply accumulates words until they no
/// longer fit on the line.
///
/// As an example, using the first-fit algorithm to wrap the famous
/// Hamlet quote “To be, or not to be: that is the question” in a
/// narrow column with room for only 10 characters looks like this:
///
/// ```
/// # use blog_text_line_layout::{WrapAlgorithm::FirstFit, Options, wrap};
/// #
/// # let lines = wrap("To be, or not to be: that is the question",
/// #                  Options::new(10).wrap_algorithm(FirstFit));
/// # assert_eq!(lines.join("\n") + "\n", "\
/// To be, or
/// not to be:
/// that is
/// the
/// question
/// # ");
/// ```
///
/// Notice how the second to last line is quite narrow because
/// “question” was too large to fit? The greedy first-fit algorithm
/// doesn’t look ahead, so it has no other option than to put
/// “question” onto its own line.
///
/// With the optimal-fit wrapping algorithm, the previous lines are
/// shortened slightly in order to make the word “is” move onto the
/// second to last line:
///
/// ```
/// # #[cfg(feature = "smawk")] {
/// # use blog_text_line_layout::{Options, WrapAlgorithm, wrap};
/// #
/// # let lines = wrap(
/// #     "To be, or not to be: that is the question",
/// #     Options::new(10).wrap_algorithm(WrapAlgorithm::new_optimal_fit())
/// # );
/// # assert_eq!(lines.join("\n") + "\n", "\
/// To be,
/// or not to
/// be: that
/// is the
/// question
/// # "); }
/// ```
///
/// Please see [`WrapAlgorithm`](crate::WrapAlgorithm) for details on
/// the choices.
///
/// # Examples
///
/// The returned iterator yields lines of type `Cow<'_, str>`. If
/// possible, the wrapped lines will borrow from the input string. For
/// example, with hanging indentation, the first line can borrow from
/// the input, but the subsequent lines become owned strings:
///
/// ```
/// use std::borrow::Cow::{Borrowed, Owned};
/// use blog_text_line_layout::{wrap, Options};
///
/// let options = Options::new(15).subsequent_indent("....");
/// let lines = wrap("Wrapping text all day long.", &options);
/// let annotated = lines
///     .iter()
///     .map(|line| match line {
///         Borrowed(text) => format!("[Borrowed] {}", text),
///         Owned(text) => format!("[Owned]    {}", text),
///     })
///     .collect::<Vec<_>>();
/// assert_eq!(
///     annotated,
///     &[
///         "[Borrowed] Wrapping text",
///         "[Owned]    ....all day",
///         "[Owned]    ....long.",
///     ]
/// );
/// ```
///
/// ## Leading and Trailing Whitespace
///
/// As a rule, leading whitespace (indentation) is preserved and
/// trailing whitespace is discarded.
///
/// In more detail, when wrapping words into lines, words are found
/// by splitting the input text on space characters. One or more
/// spaces (shown here as “␣”) are attached to the end of each word:
///
/// ```text
/// "Foo␣␣␣bar␣baz" -> ["Foo␣␣␣", "bar␣", "baz"]
/// ```
///
/// These words are then put into lines. The interword whitespace is
/// preserved, unless the lines are wrapped so that the `"Foo␣␣␣"`
/// word falls at the end of a line:
///
/// ```
/// use blog_text_line_layout::wrap;
///
/// assert_eq!(wrap("Foo   bar baz", 10), vec!["Foo   bar", "baz"]);
/// assert_eq!(wrap("Foo   bar baz", 8), vec!["Foo", "bar baz"]);
/// ```
///
/// Notice how the trailing whitespace is removed in both cases: in the
/// first example, `"bar␣"` becomes `"bar"` and in the second case
/// `"Foo␣␣␣"` becomes `"Foo"`.
///
/// Leading whitespace is preserved when the following word fits on
/// the first line. To understand this, consider how words are found
/// in a text with leading spaces:
///
/// ```text
/// "␣␣foo␣bar" -> ["␣␣", "foo␣", "bar"]
/// ```
///
/// When put into lines, the indentation is preserved if `"foo"` fits
/// on the first line, otherwise you end up with an empty line:
///
/// ```
/// use blog_text_line_layout::wrap;
///
/// assert_eq!(wrap("  foo bar", 8), vec!["  foo", "bar"]);
/// assert_eq!(wrap("  foo bar", 4), vec!["", "foo", "bar"]);
/// ```
pub fn wrap<'a, Opt>(text: &str, width_or_options: Opt) -> Vec<Cow<'_, str>>
where
    Opt: Into<Options<'a>>,
{
    let options: Options = width_or_options.into();
    let line_ending_str = options.line_ending.as_str();

    let mut lines = Vec::new();
    for line in text.split(line_ending_str) {
        wrap_single_line(line, &options, &mut lines);
    }

    lines
}

pub(crate) fn wrap_single_line<'a>(
    line: &'a str,
    options: &Options<'_>,
    lines: &mut Vec<Cow<'a, str>>,
) {
    let indent = if lines.is_empty() {
        options.initial_indent
    } else {
        options.subsequent_indent
    };
    if line.len() < options.width && indent.is_empty() {
        if options.preserve_trailing_space {
            lines.push(Cow::from(line));
        } else {
            lines.push(Cow::from(line.trim_end_matches(' ')));
        }
    } else {
        wrap_single_line_slow_path(line, options, lines)
    }
}

/// Wrap a single line of text.
///
/// This is taken when `line` is longer than `options.width`.
pub(crate) fn wrap_single_line_slow_path<'a>(
    line: &'a str,
    options: &Options<'_>,
    lines: &mut Vec<Cow<'a, str>>,
) {
    let initial_width = options
        .width
        .saturating_sub(display_width(options.initial_indent));
    let subsequent_width = options
        .width
        .saturating_sub(display_width(options.subsequent_indent));
    let line_widths = [initial_width, subsequent_width];

    let words = options.word_separator.find_words(line);
    let split_words = split_words(words, &options.word_splitter);
    let broken_words = if options.break_words {
        let mut broken_words = break_words(split_words, line_widths[1]);
        if !options.initial_indent.is_empty() {
            // Without this, the first word will always go into the
            // first line. However, since we break words based on the
            // _second_ line width, it can be wrong to unconditionally
            // put the first word onto the first line. An empty
            // zero-width word fixed this.
            broken_words.insert(0, Word::from(""));
        }
        broken_words
    } else {
        split_words.collect::<Vec<_>>()
    };

    let wrapped_words = options.wrap_algorithm.wrap(&broken_words, &line_widths);

    let mut idx = 0;
    for words in wrapped_words {
        let last_word = match words.last() {
            None => {
                lines.push(Cow::from(""));
                continue;
            }
            Some(word) => word,
        };

        // We assume here that all words are contiguous in `line`.
        // That is, the sum of their lengths should add up to the
        // length of `line`.
        let len = words
            .iter()
            .map(|word| word.len() + word.whitespace.len())
            .sum::<usize>();

        // The result is owned if we have indentation, otherwise we
        // can simply borrow an empty string.
        let mut result = if lines.is_empty() && !options.initial_indent.is_empty() {
            Cow::Owned(options.initial_indent.to_owned())
        } else if !lines.is_empty() && !options.subsequent_indent.is_empty() {
            Cow::Owned(options.subsequent_indent.to_owned())
        } else {
            // We can use an empty string here since string
            // concatenation for `Cow` preserves a borrowed value when
            // either side is empty.
            Cow::from("")
        };

        if last_word.penalty.is_empty() && options.preserve_trailing_space {
            result += &line[idx..idx + len];
        } else {
            result += &line[idx..idx + len - last_word.whitespace.len()];
        }

        if !last_word.penalty.is_empty() {
            result.to_mut().push_str(last_word.penalty);
        }

        lines.push(result);

        // Advance by the length of `result`, plus the length of
        // `last_word.whitespace` -- even if we had a penalty, we need
        // to skip over the whitespace.
        idx += len;
    }
}

#[path = "../../../../../test/foundation/text/line-layout/unit/src/wrap/tests.rs"]
#[cfg(test)]
mod tests;
