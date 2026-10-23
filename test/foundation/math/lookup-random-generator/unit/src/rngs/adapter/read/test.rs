
    use std::println;

    use super::ReadRng;
    use crate::RngCore;

    #[test]
    fn test_reader_rng_u64() {
        // transmute from the target to avoid endianness concerns.
        #[rustfmt::skip]
        let v = [0u8, 0, 0, 0, 0, 0, 0, 1,
                 0,   4, 0, 0, 3, 0, 0, 2,
                 5,   0, 0, 0, 0, 0, 0, 0];
        let mut rng = ReadRng::new(&v[..]);

        assert_eq!(rng.next_u64(), 1 << 56);
        assert_eq!(rng.next_u64(), (2 << 56) + (3 << 32) + (4 << 8));
        assert_eq!(rng.next_u64(), 5);
    }

    #[test]
    fn test_reader_rng_u32() {
        let v = [0u8, 0, 0, 1, 0, 0, 2, 0, 3, 0, 0, 0];
        let mut rng = ReadRng::new(&v[..]);

        assert_eq!(rng.next_u32(), 1 << 24);
        assert_eq!(rng.next_u32(), 2 << 16);
        assert_eq!(rng.next_u32(), 3);
    }

    #[test]
    fn test_reader_rng_fill_bytes() {
        let v = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut w = [0u8; 8];

        let mut rng = ReadRng::new(&v[..]);
        rng.fill_bytes(&mut w);

        assert!(v == w);
    }

    #[test]
    fn test_reader_rng_insufficient_bytes() {
        let v = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut w = [0u8; 9];

        let mut rng = ReadRng::new(&v[..]);

        let result = rng.try_fill_bytes(&mut w);
        assert!(result.is_err());
        println!("Error: {}", result.unwrap_err());
    }
