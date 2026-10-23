
    use alloc::string::String;
    use alloc::vec::Vec;
    #[cfg(feature = "std")]
    use std::io::Cursor;

    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::tests::{rand_bytes, rand_unicode};
    use crate::CompactString;

    const MAX_SIZE: usize = core::mem::size_of::<String>();

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_buffers_roundtrip(#[strategy(rand_unicode())] word: String) {
        let mut buf = Cursor::new(word.as_bytes());
        let compact = CompactString::from_utf8_buf(&mut buf).unwrap();

        proptest::prop_assert_eq!(&word, &compact);
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_allocated_properly(#[strategy(rand_unicode())] word: String) {
        let mut buf = Cursor::new(word.as_bytes());
        let compact = CompactString::from_utf8_buf(&mut buf).unwrap();

        if word.len() <= MAX_SIZE {
            proptest::prop_assert!(!compact.is_heap_allocated())
        } else {
            proptest::prop_assert!(compact.is_heap_allocated())
        }
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_only_accept_valid_utf8(#[strategy(rand_bytes())] bytes: Vec<u8>) {
        let mut buf = Cursor::new(bytes.as_slice());

        let compact_result = CompactString::from_utf8_buf(&mut buf);
        let str_result = core::str::from_utf8(bytes.as_slice());

        match (compact_result, str_result) {
            (Ok(c), Ok(s)) => prop_assert_eq!(c, s),
            (Err(c_err), Err(s_err)) => prop_assert_eq!(c_err, s_err),
            _ => panic!("CompactString and core::str read UTF-8 differently?"),
        }
    }
