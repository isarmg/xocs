
    use super::*;

    #[test]
    fn reference() {
        let mut rng = Xoshiro128PlusPlus::from_seed(
            [1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128plusplus.c
        let expected = [
            641, 1573767, 3222811527, 3517856514, 836907274, 4247214768,
            3867114732, 1355841295, 495546011, 621204420,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }
