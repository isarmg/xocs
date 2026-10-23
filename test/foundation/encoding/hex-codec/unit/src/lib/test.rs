
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::ToString;
    use pretty_assertions::assert_eq;

    #[test]
    #[cfg(feature = "alloc")]
    fn test_gen_iter() {
        let result = vec![(0, 1), (2, 3)];

        assert_eq!(generate_iter(5).collect::<Vec<_>>(), result);
    }

    #[test]
    fn test_encode_to_slice() {
        let mut output_1 = [0; 4 * 2];
        encode_to_slice(b"kiwi", &mut output_1).unwrap();
        assert_eq!(&output_1, b"6b697769");

        let mut output_2 = [0; 5 * 2];
        encode_to_slice(b"kiwis", &mut output_2).unwrap();
        assert_eq!(&output_2, b"6b69776973");

        let mut output_3 = [0; 100];

        assert_eq!(
            encode_to_slice(b"kiwis", &mut output_3),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    fn test_decode_to_slice() {
        let mut output_1 = [0; 4];
        decode_to_slice(b"6b697769", &mut output_1).unwrap();
        assert_eq!(&output_1, b"kiwi");

        let mut output_2 = [0; 5];
        decode_to_slice(b"6b69776973", &mut output_2).unwrap();
        assert_eq!(&output_2, b"kiwis");

        let mut output_3 = [0; 4];

        assert_eq!(
            decode_to_slice(b"6", &mut output_3),
            Err(FromHexError::OddLength)
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_encode() {
        assert_eq!(encode("foobar"), "666f6f626172");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_decode() {
        assert_eq!(
            decode("666f6f626172"),
            Ok(String::from("foobar").into_bytes())
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_okay_str() {
        assert_eq!(Vec::from_hex("666f6f626172").unwrap(), b"foobar");
        assert_eq!(Vec::from_hex("666F6F626172").unwrap(), b"foobar");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_okay_bytes() {
        assert_eq!(Vec::from_hex(b"666f6f626172").unwrap(), b"foobar");
        assert_eq!(Vec::from_hex(b"666F6F626172").unwrap(), b"foobar");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_invalid_length() {
        assert_eq!(Vec::from_hex("1").unwrap_err(), FromHexError::OddLength);
        assert_eq!(
            Vec::from_hex("666f6f6261721").unwrap_err(),
            FromHexError::OddLength
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_invalid_char() {
        assert_eq!(
            Vec::from_hex("66ag").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: 'g', index: 3 }
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_empty() {
        assert_eq!(Vec::from_hex("").unwrap(), b"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex_whitespace() {
        assert_eq!(
            Vec::from_hex("666f 6f62617").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: ' ', index: 4 }
        );
    }

    #[test]
    pub fn test_from_hex_array() {
        assert_eq!(
            <[u8; 6] as FromHex>::from_hex("666f6f626172"),
            Ok([0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72])
        );

        assert_eq!(
            <[u8; 5] as FromHex>::from_hex("666f6f626172"),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_to_hex() {
        assert_eq!(
            [0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72].encode_hex::<String>(),
            "666f6f626172".to_string(),
        );

        assert_eq!(
            [0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72].encode_hex_upper::<String>(),
            "666F6F626172".to_string(),
        );
    }
