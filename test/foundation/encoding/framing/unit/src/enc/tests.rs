
    #[cfg(feature = "alloc")]
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn encode_target_buf_too_small() {
        let source = &[10, 11, 0, 12];
        let expected = &[3, 10, 11, 2, 12];
        for len in 0..expected.len() {
            let mut dest = alloc::vec![0; len];
            matches!(
                try_encode(source, &mut dest).unwrap_err(),
                DestBufTooSmallError
            );
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    #[should_panic]
    fn encode_target_buf_too_small_panicking() {
        let source = &[10, 11, 0, 12];
        let expected = &[3, 10, 11, 2, 12];
        encode(source, &mut alloc::vec![0; expected.len() - 1]);
    }
