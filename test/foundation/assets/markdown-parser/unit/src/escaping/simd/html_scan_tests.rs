
        #[test]
        fn multichunk() {
            let mut vec = Vec::new();
            unsafe {
                super::foreach_special_simd("&aXaaaa.a'aa9a<>aab&".as_bytes(), 0, |ix| {
                    #[allow(clippy::unit_arg)]
                    Ok::<_, std::fmt::Error>(vec.push(ix))
                })
                .unwrap();
            }
            assert_eq!(vec, vec![0, 9, 14, 15, 19]);
        }

        // only match these bytes, and when we match them, match them VECTOR_SIZE times
        #[test]
        fn only_right_bytes_matched() {
            for b in 0..255u8 {
                let right_byte = b == b'&' || b == b'<' || b == b'>' || b == b'"' || b == b'\'';
                let vek = vec![b; super::VECTOR_SIZE];
                let mut match_count = 0;
                unsafe {
                    super::foreach_special_simd(&vek, 0, |_| {
                        match_count += 1;
                        Ok::<_, std::fmt::Error>(())
                    })
                    .unwrap();
                }
                assert!((match_count > 0) == (match_count == super::VECTOR_SIZE));
                assert_eq!(
                    (match_count == super::VECTOR_SIZE),
                    right_byte,
                    "match_count: {}, byte: {:?}",
                    match_count,
                    b as char
                );
            }
        }
    
