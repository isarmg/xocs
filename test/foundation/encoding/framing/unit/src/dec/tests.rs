

    use super::*;

    #[test]
    fn decode_malforemd() {
        let malformed_buf: [u8; 32] = [
            68, 69, 65, 68, 66, 69, 69, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let mut dest_buf: [u8; 32] = [0; 32];
        if let Err(DecodeError::InvalidFrame { decoded_bytes }) =
            decode(&malformed_buf, &mut dest_buf)
        {
            assert_eq!(decoded_bytes, 7);
        } else {
            panic!("decoding worked when it should not have");
        }
    }

    #[test]
    fn decode_empty() {
        #[cfg(feature = "alloc")]
        matches!(decode_vec(&[]).unwrap_err(), DecodeError::EmptyFrame);
        matches!(
            decode_in_place(&mut []).unwrap_err(),
            DecodeError::EmptyFrame
        );
        matches!(
            decode(&[], &mut [0; 256]).unwrap_err(),
            DecodeError::EmptyFrame
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn decode_target_buf_too_small() {
        let encoded = &[3, 10, 11, 2, 12];
        let expected_decoded_len = 4;
        for i in 0..expected_decoded_len - 1 {
            let mut dest = alloc::vec![0; i];
            let result = decode(encoded, &mut dest);
            assert_eq!(result, Err(DecodeError::TargetBufTooSmall));
        }
    }
