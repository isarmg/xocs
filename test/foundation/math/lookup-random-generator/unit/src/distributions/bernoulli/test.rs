
    use super::Bernoulli;
    use crate::distributions::Distribution;
    use crate::Rng;

    #[test]
    #[cfg(feature = "serde1")]
    fn test_serializing_deserializing_bernoulli() {
        let coin_flip = Bernoulli::new(0.5).unwrap();
        let de_coin_flip: Bernoulli =
            bincode::deserialize(&bincode::serialize(&coin_flip).unwrap()).unwrap();

        assert_eq!(coin_flip.p_int, de_coin_flip.p_int);
    }

    #[test]
    fn test_trivial() {
        // We prefer to be explicit here.
        #![allow(clippy::bool_assert_comparison)]

        let mut r = crate::test::rng(1);
        let always_false = Bernoulli::new(0.0).unwrap();
        let always_true = Bernoulli::new(1.0).unwrap();
        for _ in 0..5 {
            assert_eq!(r.sample::<bool, _>(&always_false), false);
            assert_eq!(r.sample::<bool, _>(&always_true), true);
            assert_eq!(Distribution::<bool>::sample(&always_false, &mut r), false);
            assert_eq!(Distribution::<bool>::sample(&always_true, &mut r), true);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_average() {
        const P: f64 = 0.3;
        const NUM: u32 = 3;
        const DENOM: u32 = 10;
        let d1 = Bernoulli::new(P).unwrap();
        let d2 = Bernoulli::from_ratio(NUM, DENOM).unwrap();
        const N: u32 = 100_000;

        let mut sum1: u32 = 0;
        let mut sum2: u32 = 0;
        let mut rng = crate::test::rng(2);
        for _ in 0..N {
            if d1.sample(&mut rng) {
                sum1 += 1;
            }
            if d2.sample(&mut rng) {
                sum2 += 1;
            }
        }
        let avg1 = (sum1 as f64) / (N as f64);
        assert!((avg1 - P).abs() < 5e-3);

        let avg2 = (sum2 as f64) / (N as f64);
        assert!((avg2 - (NUM as f64) / (DENOM as f64)).abs() < 5e-3);
    }

    #[test]
    fn value_stability() {
        let mut rng = crate::test::rng(3);
        let distr = Bernoulli::new(0.4532).unwrap();
        let mut buf = [false; 10];
        for x in &mut buf {
            *x = rng.sample(&distr);
        }
        assert_eq!(
            buf,
            [true, false, false, true, false, false, true, true, true, true]
        );
    }

    #[test]
    fn bernoulli_distributions_can_be_compared() {
        assert_eq!(Bernoulli::new(1.0), Bernoulli::new(1.0));
    }
