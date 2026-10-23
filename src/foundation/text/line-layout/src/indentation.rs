//! Functions related to adding and removing indentation from lines of
//! text.
//!
//! The functions here can be used to uniformly indent or dedent
//! (unindent) word-wrapped lines of text.

use alloc::string::String;

/// Indent each line by the given prefix.
///
/// # Examples
///
/// ```
/// use blog_text_line_layout::indent;
///
/// assert_eq!(indent("First line.\nSecond line.\n", "  "),
///            "  First line.\n  Second line.\n");
/// ```
///
/// When indenting, trailing whitespace is stripped from the prefix.
/// This means that empty lines remain empty afterwards:
///
/// ```
/// use blog_text_line_layout::indent;
///
/// assert_eq!(indent("First line.\n\n\nSecond line.\n", "  "),
///            "  First line.\n\n\n  Second line.\n");
/// ```
///
/// Notice how `"\n\n\n"` remained as `"\n\n\n"`.
///
/// This feature is useful when you want to indent text and have a
/// space between your prefix and the text. In this case, you _don't_
/// want a trailing space on empty lines:
///
/// ```
/// use blog_text_line_layout::indent;
///
/// assert_eq!(indent("foo = 123\n\nprint(foo)\n", "# "),
///            "# foo = 123\n#\n# print(foo)\n");
/// ```
///
/// Notice how `"\n\n"` became `"\n#\n"` instead of `"\n# \n"` which
/// would have trailing whitespace.
///
/// Leading and trailing whitespace from the text itself is kept
/// unchanged:
///
/// ```
/// use blog_text_line_layout::indent;
///
/// assert_eq!(indent(" \t  Foo   ", "->"), "-> \t  Foo   ");
/// ```
pub fn indent(s: &str, prefix: &str) -> String {
    // We know we'll need more than s.len() bytes for the output, but
    // without counting '\n' characters (which is somewhat slow), we
    // don't know exactly how much. However, we can preemptively do
    // the first doubling of the output size.
    let mut result = String::with_capacity(2 * s.len());
    let trimmed_prefix = prefix.trim_end();
    for (idx, line) in s.split_terminator('\n').enumerate() {
        if idx > 0 {
            result.push('\n');
        }
        if line.trim().is_empty() {
            result.push_str(trimmed_prefix);
        } else {
            result.push_str(prefix);
        }
        result.push_str(line);
    }
    if s.ends_with('\n') {
        // split_terminator will have eaten the final '\n'.
        result.push('\n');
    }
    result
}

/// Removes common leading whitespace from each line.
///
/// This function will look at each non-empty line and determine the
/// maximum amount of whitespace that can be removed from all lines:
///
/// ```
/// use blog_text_line_layout::dedent;
///
/// assert_eq!(dedent("
///     1st line
///       2nd line
///     3rd line
/// "), "
/// 1st line
///   2nd line
/// 3rd line
/// ");
/// ```
pub fn dedent(s: &str) -> String {
    let mut prefix = "";
    let mut lines = s.lines();

    // We first search for a non-empty line to find a prefix.
    for line in &mut lines {
        let mut whitespace_idx = line.len();
        for (idx, ch) in line.char_indices() {
            if !ch.is_whitespace() {
                whitespace_idx = idx;
                break;
            }
        }

        // Check if the line had anything but whitespace
        if whitespace_idx < line.len() {
            prefix = &line[..whitespace_idx];
            break;
        }
    }

    // We then continue looking through the remaining lines to
    // possibly shorten the prefix.
    for line in &mut lines {
        let mut whitespace_idx = line.len();
        for ((idx, a), b) in line.char_indices().zip(prefix.chars()) {
            if a != b {
                whitespace_idx = idx;
                break;
            }
        }

        // Check if the line had anything but whitespace and if we
        // have found a shorter prefix
        if whitespace_idx < line.len() && whitespace_idx < prefix.len() {
            prefix = &line[..whitespace_idx];
        }
    }

    // We now go over the lines a second time to build the result.
    let mut result = String::new();
    for line in s.lines() {
        if line.starts_with(prefix) && line.chars().any(|c| !c.is_whitespace()) {
            let (_, tail) = line.split_at(prefix.len());
            result.push_str(tail);
        }
        result.push('\n');
    }

    if result.ends_with('\n') && !s.ends_with('\n') {
        let new_len = result.len() - 1;
        result.truncate(new_len);
    }

    result
}

#[path = "../../../../../test/foundation/text/line-layout/unit/src/indentation/tests.rs"]
#[cfg(test)]
mod tests;
