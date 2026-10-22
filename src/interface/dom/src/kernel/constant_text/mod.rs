#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Utilities for const concatenation of string slices.

pub(crate) const MAX_TEMPLATE_SIZE: usize = 4096;

/// Converts a zero-terminated buffer of bytes into a UTF-8 string.
pub const fn str_from_buffer(buf: &[u8; MAX_TEMPLATE_SIZE]) -> &str {
    match core::ffi::CStr::from_bytes_until_nul(buf) {
        Ok(cstr) => match cstr.to_str() {
            Ok(str) => str,
            Err(_) => panic!("TEMPLATE FAILURE"),
        },
        Err(_) => panic!("TEMPLATE FAILURE"),
    }
}

/// Concatenates any number of static strings into a single array.
// credit to Rainer Stropek, "Constant fun," Rust Linz, June 2022
pub const fn const_concat(strs: &'static [&'static str]) -> [u8; MAX_TEMPLATE_SIZE] {
    let mut buffer = [0; MAX_TEMPLATE_SIZE];
    let mut position = 0;
    let mut remaining = strs;

    while let [current, tail @ ..] = remaining {
        let x = current.as_bytes();
        let mut i = 0;

        // have it iterate over bytes manually, because, again,
        // no mutable refernces in const fns
        while i < x.len() {
            buffer[position] = x[i];
            position += 1;
            i += 1;
        }

        remaining = tail;
    }

    buffer
}

/// Converts a zero-terminated buffer of bytes into a UTF-8 string with the given prefix.
pub const fn const_concat_with_prefix(
    strs: &'static [&'static str],
    prefix: &'static str,
    suffix: &'static str,
) -> [u8; MAX_TEMPLATE_SIZE] {
    let mut buffer = [0; MAX_TEMPLATE_SIZE];
    let mut position = 0;
    let mut remaining = strs;

    while let [current, tail @ ..] = remaining {
        let x = current.as_bytes();
        let mut i = 0;

        // have it iterate over bytes manually, because, again,
        // no mutable refernces in const fns
        while i < x.len() {
            buffer[position] = x[i];
            position += 1;
            i += 1;
        }

        remaining = tail;
    }

    if buffer[0] == 0 {
        buffer
    } else {
        let mut new_buf = [0; MAX_TEMPLATE_SIZE];
        let prefix = prefix.as_bytes();
        let suffix = suffix.as_bytes();
        let mut position = 0;
        let mut i = 0;
        while i < prefix.len() {
            new_buf[position] = prefix[i];
            position += 1;
            i += 1;
        }
        i = 0;
        while i < buffer.len() {
            if buffer[i] == 0 {
                break;
            }
            new_buf[position] = buffer[i];
            position += 1;
            i += 1;
        }
        i = 0;
        while i < suffix.len() {
            new_buf[position] = suffix[i];
            position += 1;
            i += 1;
        }

        new_buf
    }
}

/// Joins non-empty template fragments, separating them with the given string.
///
/// Empty fragments contribute no class or style text, so they do not introduce
/// leading, trailing, or repeated separators. The buffer reserves its final
/// byte for the zero terminator expected by [`str_from_buffer`].
pub const fn const_concat_with_separator(
    strs: &[&str],
    separator: &'static str,
) -> [u8; MAX_TEMPLATE_SIZE] {
    let mut buffer = [0; MAX_TEMPLATE_SIZE];
    let mut position = 0;
    let mut remaining = strs;
    let mut has_fragment = false;

    while let [current, tail @ ..] = remaining {
        let bytes = current.as_bytes();
        if !bytes.is_empty() {
            if has_fragment {
                let separator = separator.as_bytes();
                let mut index = 0;
                while index < separator.len() {
                    assert!(
                        position < MAX_TEMPLATE_SIZE - 1,
                        "template exceeds fixed buffer capacity"
                    );
                    buffer[position] = separator[index];
                    position += 1;
                    index += 1;
                }
            }
            let mut index = 0;
            while index < bytes.len() {
                assert!(
                    position < MAX_TEMPLATE_SIZE - 1,
                    "template exceeds fixed buffer capacity"
                );
                buffer[position] = bytes[index];
                position += 1;
                index += 1;
            }
            has_fragment = true;
        }
        remaining = tail;
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separators_preserve_fragments_and_skip_empty_attribute_text() {
        for (parts, separator, expected) in [
            (&["a", "b"][..], ",", "a,b"),
            (&["a", "b"][..], "::", "a::b"),
            (&["", "red", "", "blue", ""][..], " ", "red blue"),
            (
                &["", "color:red", "", "display:block", ""][..],
                ";",
                "color:red;display:block",
            ),
            (&["", ""][..], ";", ""),
            (&[][..], ";", ""),
            (&["中", "文"][..], "・", "中・文"),
            (&["one", "two"][..], "", "onetwo"),
        ] {
            assert_eq!(
                str_from_buffer(&const_concat_with_separator(parts, separator)),
                expected
            );
        }
        const JOINED: [u8; MAX_TEMPLATE_SIZE] =
            const_concat_with_separator(&["left", "right"], "::");
        assert_eq!(str_from_buffer(&JOINED), "left::right");
    }

    #[test]
    fn joined_templates_reserve_the_zero_terminator_at_capacity() {
        let text = "x".repeat(MAX_TEMPLATE_SIZE - 1);
        let result = const_concat_with_separator(&[&text], ",");
        assert_eq!(str_from_buffer(&result), text);
        let text = "x".repeat(MAX_TEMPLATE_SIZE - 2);
        let result = const_concat_with_separator(&[&text, "y"], "");
        assert_eq!(str_from_buffer(&result).len(), MAX_TEMPLATE_SIZE - 1);
        assert!(
            std::panic::catch_unwind(|| const_concat_with_separator(&[&text, "y"], ",")).is_err()
        );
        let too_long = "x".repeat(MAX_TEMPLATE_SIZE);
        assert!(
            std::panic::catch_unwind(|| const_concat_with_separator(&[&too_long], "")).is_err()
        );
    }
}
