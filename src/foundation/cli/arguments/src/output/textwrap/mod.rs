//! Fork of `textwrap` crate
//!
//! Benefits of forking:
//! - Pull in only what we need rather than relying on the compiler to remove what we don't need
//! - `LineWrapper` is able to incrementally wrap which will help with `StyledStr`

pub(crate) mod core;





pub(crate) fn wrap(content: &str, _hard_width: usize) -> String {
    content.to_owned()
}
