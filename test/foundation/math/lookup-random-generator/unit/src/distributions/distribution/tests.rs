
    use crate::distributions::{Distribution, Uniform};
    use crate::Rng;

    #[test]
    fn test_distributions_iter() {
        use crate::distributions::Open01;
        let mut rng = crate::test::rng(210);
        let distr = Open01;
        let mut iter = Distribution::<f32>::sample_iter(distr, &mut rng);
        let mut sum: f32 = 0.;
        for _ in 0..100 {
            sum += iter.next().unwrap();
        }
        assert!(0. < sum && sum < 100.);
    }

    #[test]
    fn test_distributions_map() {
        let dist = Uniform::new_inclusive(0, 5).map(|val| val + 15);

        let mut rng = crate::test::rng(212);
        let val = dist.sample(&mut rng);
        assert!((15..=20).contains(&val));
    }

    #[test]
    fn test_make_an_iter() {
        fn ten_dice_rolls_other_than_five<R: Rng>(rng: &mut R) -> impl Iterator<Item = i32> + '_ {
            Uniform::new_inclusive(1, 6)
                .sample_iter(rng)
                .filter(|x| *x != 5)
                .take(10)
        }

        let mut rng = crate::test::rng(211);
        let mut count = 0;
        for val in ten_dice_rolls_other_than_five(&mut rng) {
            assert!((1..=6).contains(&val) && val != 5);
            count += 1;
        }
        assert_eq!(count, 10);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_dist_string() {
        use crate::distributions::{Alphanumeric, DistString, Standard};
        use core::str;
        let mut rng = crate::test::rng(213);

        let s1 = Alphanumeric.sample_string(&mut rng, 20);
        assert_eq!(s1.len(), 20);
        assert_eq!(str::from_utf8(s1.as_bytes()), Ok(s1.as_str()));

        let s2 = Standard.sample_string(&mut rng, 20);
        assert_eq!(s2.chars().count(), 20);
        assert_eq!(str::from_utf8(s2.as_bytes()), Ok(s2.as_str()));
    }
