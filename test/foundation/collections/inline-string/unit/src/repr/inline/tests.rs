
    #[test]
    #[ignore] // we run this in CI, but unless you're compiling in release, this takes a while
    fn test_unused_utf8_bytes() {
        use rayon::prelude::*;

        // test to validate for all char the first and last bytes are never within a specified range
        // note: according to the UTF-8 spec it shouldn't be, but we double check that here
        (0..u32::MAX).into_par_iter().for_each(|i| {
            if let Ok(c) = char::try_from(i) {
                let mut buf = [0_u8; 4];
                c.encode_utf8(&mut buf);

                // check ranges for first byte
                match buf[0] {
                    x @ 128..=191 => panic!("first byte within 128..=191, {}", x),
                    x @ 248..=255 => panic!("first byte within 248..=255, {}", x),
                    _ => (),
                }

                // check ranges for last byte
                if let x @ 192..=255 = buf[c.len_utf8() - 1] {
                    panic!("last byte within 192..=255, {}", x)
                }
            }
        })
    }

    #[cfg(feature = "smallvec")]
    mod smallvec {
        use alloc::string::String;

        use quickcheck_macros::quickcheck;

        use crate::repr::{InlineBuffer, MAX_SIZE};

        #[test]
        fn test_into_array() {
            let s = "hello world!";

            let inline = unsafe { InlineBuffer::new(s) };
            let (array, length) = inline.into_array();

            assert_eq!(s.len(), length);

            // all bytes after the length should be 0
            assert!(array[length..].iter().all(|b| *b == 0));

            // taking a string slice should give back the same string as the original
            let ex_s = unsafe { core::str::from_utf8_unchecked(&array[..length]) };
            assert_eq!(s, ex_s);
        }

        #[quickcheck]
        #[cfg_attr(miri, ignore)]
        fn quickcheck_into_array(s: String) {
            let mut total_length = 0;
            let s: String = s
                .chars()
                .take_while(|c| {
                    total_length += c.len_utf8();
                    total_length < MAX_SIZE
                })
                .collect();

            let inline = unsafe { InlineBuffer::new(&s) };
            let (array, length) = inline.into_array();
            assert_eq!(s.len(), length);

            // all bytes after the length should be 0
            assert!(array[length..].iter().all(|b| *b == 0));

            // taking a string slice should give back the same string as the original
            let ex_s = unsafe { core::str::from_utf8_unchecked(&array[..length]) };
            assert_eq!(s, ex_s);
        }
    }
