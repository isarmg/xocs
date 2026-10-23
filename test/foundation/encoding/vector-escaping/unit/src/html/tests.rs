
    use super::*;
    use crate::{HTML_ESCAPE_TAB, HTML_NEED_ESCAPED};

    #[test]
    fn test_html_capacity_overflow() {
        assert_eq!(html_capacity(usize::MAX), Err(InputTooLarge));
        assert_eq!(html_capacity(usize::MAX / 6), Err(InputTooLarge));
        assert_eq!(html_capacity(10), Ok(92));
        assert_eq!(html_capacity(0), Ok(32));
    }

    #[test]
    fn test_html_escape_tables() {
        let expected: [(u8, &str); 6] = [
            (b'"', "&quot;"),
            (b'&', "&amp;"),
            (b'\'', "&#x27;"),
            (b'/', "&#x2f;"),
            (b'<', "&lt;"),
            (b'>', "&gt;"),
        ];
        for i in 0..256usize {
            match expected.iter().find(|&&(ch, _)| ch as usize == i) {
                Some(&(_, esc)) => {
                    let (len, bytes) = &HTML_ESCAPE_TAB[i];
                    assert_eq!(*len as usize, esc.len(), "len for byte 0x{i:02x}");
                    assert_eq!(&bytes[..esc.len()], esc.as_bytes(), "bytes for 0x{i:02x}");
                    assert!(
                        bytes[esc.len()..].iter().all(|&b| b == 0),
                        "padding for byte 0x{i:02x}"
                    );
                    assert_eq!(HTML_NEED_ESCAPED[i], 1, "need_escaped for byte 0x{i:02x}");
                }
                None => {
                    assert_eq!(HTML_ESCAPE_TAB[i], (0, [0; 8]), "entry for byte 0x{i:02x}");
                    assert_eq!(HTML_NEED_ESCAPED[i], 0, "need_escaped for byte 0x{i:02x}");
                }
            }
        }
    }
