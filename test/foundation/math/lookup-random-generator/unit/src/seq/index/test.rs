
    use super::*;

    #[test]
    #[cfg(feature = "serde1")]
    fn test_serialization_index_vec() {
        let some_index_vec = IndexVec::from(vec![254_usize, 234, 2, 1]);
        let de_some_index_vec: IndexVec =
            bincode::deserialize(&bincode::serialize(&some_index_vec).unwrap()).unwrap();
        match (some_index_vec, de_some_index_vec) {
            (IndexVec::U32(a), IndexVec::U32(b)) => {
                assert_eq!(a, b);
            }
            (IndexVec::USize(a), IndexVec::USize(b)) => {
                assert_eq!(a, b);
            }
            _ => {
                panic!("failed to seralize/deserialize `IndexVec`")
            }
        }
    }

    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[test]
    fn test_sample_boundaries() {
        let mut r = crate::test::rng(404);

        assert_eq!(sample_inplace(&mut r, 0, 0).len(), 0);
        assert_eq!(sample_inplace(&mut r, 1, 0).len(), 0);
        assert_eq!(sample_inplace(&mut r, 1, 1).into_vec(), vec![0]);

        assert_eq!(sample_rejection(&mut r, 1u32, 0).len(), 0);

        assert_eq!(sample_floyd(&mut r, 0, 0).len(), 0);
        assert_eq!(sample_floyd(&mut r, 1, 0).len(), 0);
        assert_eq!(sample_floyd(&mut r, 1, 1).into_vec(), vec![0]);

        // These algorithms should be fast with big numbers. Test average.
        let sum: usize = sample_rejection(&mut r, 1 << 25, 10u32).into_iter().sum();
        assert!(1 << 25 < sum && sum < (1 << 25) * 25);

        let sum: usize = sample_floyd(&mut r, 1 << 25, 10).into_iter().sum();
        assert!(1 << 25 < sum && sum < (1 << 25) * 25);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_sample_alg() {
        let seed_rng = crate::test::rng;

        // We can't test which algorithm is used directly, but Floyd's alg
        // should produce different results from the others. (Also, `inplace`
        // and `cached` currently use different sizes thus produce different results.)

        // A small length and relatively large amount should use inplace
        let (length, amount): (usize, usize) = (100, 50);
        let v1 = sample(&mut seed_rng(420), length, amount);
        let v2 = sample_inplace(&mut seed_rng(420), length as u32, amount as u32);
        assert!(v1.iter().all(|e| e < length));
        assert_eq!(v1, v2);

        // Test Floyd's alg does produce different results
        let v3 = sample_floyd(&mut seed_rng(420), length as u32, amount as u32);
        assert!(v1 != v3);

        // A large length and small amount should use Floyd
        let (length, amount): (usize, usize) = (1 << 20, 50);
        let v1 = sample(&mut seed_rng(421), length, amount);
        let v2 = sample_floyd(&mut seed_rng(421), length as u32, amount as u32);
        assert!(v1.iter().all(|e| e < length));
        assert_eq!(v1, v2);

        // A large length and larger amount should use cache
        let (length, amount): (usize, usize) = (1 << 20, 600);
        let v1 = sample(&mut seed_rng(422), length, amount);
        let v2 = sample_rejection(&mut seed_rng(422), length as u32, amount as u32);
        assert!(v1.iter().all(|e| e < length));
        assert_eq!(v1, v2);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_sample_weighted() {
        let seed_rng = crate::test::rng;
        for &(amount, len) in &[(0, 10), (5, 10), (10, 10)] {
            let v = sample_weighted(&mut seed_rng(423), len, |i| i as f64, amount).unwrap();
            match v {
                IndexVec::U32(mut indices) => {
                    assert_eq!(indices.len(), amount);
                    indices.sort_unstable();
                    indices.dedup();
                    assert_eq!(indices.len(), amount);
                    for &i in &indices {
                        assert!((i as usize) < len);
                    }
                }
                IndexVec::USize(_) => panic!("expected `IndexVec::U32`"),
            }
        }
    }

    #[test]
    fn value_stability_sample() {
        let do_test = |length, amount, values: &[u32]| {
            let mut buf = [0u32; 8];
            let mut rng = crate::test::rng(410);

            let res = sample(&mut rng, length, amount);
            let len = res.len().min(buf.len());
            for (x, y) in res.into_iter().zip(buf.iter_mut()) {
                *y = x as u32;
            }
            assert_eq!(
                &buf[0..len],
                values,
                "failed sampling {}, {}",
                length,
                amount
            );
        };

        do_test(10, 6, &[8, 0, 3, 5, 9, 6]); // floyd
        do_test(25, 10, &[18, 15, 14, 9, 0, 13, 5, 24]); // floyd
        do_test(300, 8, &[30, 283, 150, 1, 73, 13, 285, 35]); // floyd
        do_test(300, 80, &[31, 289, 248, 154, 5, 78, 19, 286]); // inplace
        do_test(300, 180, &[31, 289, 248, 154, 5, 78, 19, 286]); // inplace

        do_test(
            1_000_000,
            8,
            &[103717, 963485, 826422, 509101, 736394, 807035, 5327, 632573],
        ); // floyd
        do_test(
            1_000_000,
            180,
            &[103718, 963490, 826426, 509103, 736396, 807036, 5327, 632573],
        ); // rejection
    }
