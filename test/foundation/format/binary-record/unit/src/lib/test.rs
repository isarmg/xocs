
    #[test]
    fn varint_boundary_canon() {
        let x = u32::MAX;
        let mut buf = [0u8; 5];
        let used = crate::to_slice(&x, &mut buf).unwrap();
        let deser: u32 = crate::from_bytes(used).unwrap();
        assert_eq!(deser, u32::MAX);
        assert_eq!(used, &mut [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        let deser: Result<u32, crate::Error> = crate::from_bytes(&[0xFF, 0xFF, 0xFF, 0xFF, 0x1F]);
        assert_eq!(deser, Err(crate::Error::DeserializeBadVarint));
    }

    #[test]
    fn signed_int128() {
        let x = -19490127978232325886905073712831_i128;
        let mut buf = [0u8; 32];
        let used = crate::to_slice(&x, &mut buf).unwrap();
        let deser: i128 = crate::from_bytes(used).unwrap();
        assert_eq!(deser, x);
    }
