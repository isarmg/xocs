
    use super::*;

    #[test]
    fn test_integers() {
        let mut rng = crate::test::rng(806);

        rng.sample::<isize, _>(Standard);
        rng.sample::<i8, _>(Standard);
        rng.sample::<i16, _>(Standard);
        rng.sample::<i32, _>(Standard);
        rng.sample::<i64, _>(Standard);
        rng.sample::<i128, _>(Standard);

        rng.sample::<usize, _>(Standard);
        rng.sample::<u8, _>(Standard);
        rng.sample::<u16, _>(Standard);
        rng.sample::<u32, _>(Standard);
        rng.sample::<u64, _>(Standard);
        rng.sample::<u128, _>(Standard);
    }

    #[test]
    fn value_stability() {
        fn test_samples<T: Copy + core::fmt::Debug + PartialEq>(zero: T, expected: &[T])
        where Standard: Distribution<T> {
            let mut rng = crate::test::rng(807);
            let mut buf = [zero; 3];
            for x in &mut buf {
                *x = rng.sample(Standard);
            }
            assert_eq!(&buf, expected);
        }

        test_samples(0u8, &[9, 247, 111]);
        test_samples(0u16, &[32265, 42999, 38255]);
        test_samples(0u32, &[2220326409, 2575017975, 2018088303]);
        test_samples(0u64, &[
            11059617991457472009,
            16096616328739788143,
            1487364411147516184,
        ]);
        test_samples(0u128, &[
            296930161868957086625409848350820761097,
            145644820879247630242265036535529306392,
            111087889832015897993126088499035356354,
        ]);
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
        test_samples(0usize, &[2220326409, 2575017975, 2018088303]);
        #[cfg(target_pointer_width = "64")]
        test_samples(0usize, &[
            11059617991457472009,
            16096616328739788143,
            1487364411147516184,
        ]);

        test_samples(0i8, &[9, -9, 111]);
        // Skip further i* types: they are simple reinterpretation of u* samples
    }
