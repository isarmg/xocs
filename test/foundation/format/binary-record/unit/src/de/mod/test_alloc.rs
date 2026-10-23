
    extern crate alloc;

    use super::*;

    use alloc::vec;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct ZSTStruct;

    #[test]
    fn zst_vec() {
        assert_eq!(from_bytes(&[3]), Ok(vec![ZSTStruct, ZSTStruct, ZSTStruct]));

        assert_eq!(
            from_bytes(&[4]),
            Ok(vec![ZSTStruct, ZSTStruct, ZSTStruct, ZSTStruct])
        );
    }

    #[test]
    fn vec() {
        assert_eq!(
            from_bytes::<Vec<u8>>(&[8, 255, 255, 255, 0, 0, 0, 0, 0]),
            Ok(vec![255, 255, 255, 0, 0, 0, 0, 0])
        );

        // This won't actually prove anything since tests will likely always be
        // run on devices with larger amounts of memory, but it can't hurt.
        assert_eq!(
            from_bytes::<Vec<u8>>(&[(1 << 7) | 8, 255, 255, 255, 0, 0, 0, 0, 0]),
            Err(Error::DeserializeUnexpectedEnd)
        );
    }
