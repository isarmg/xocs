
    use super::*;

    #[cfg(feature = "serde1")]
    #[test]
    fn test_weightedindex_serde1() {
        let weighted_index = WeightedIndex::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap();

        let ser_weighted_index = bincode::serialize(&weighted_index).unwrap();
        let de_weighted_index: WeightedIndex<i32> =
            bincode::deserialize(&ser_weighted_index).unwrap();

        assert_eq!(
            de_weighted_index.cumulative_weights,
            weighted_index.cumulative_weights
        );
        assert_eq!(de_weighted_index.total_weight, weighted_index.total_weight);
    }

    #[test]
    fn test_accepting_nan() {
        assert_eq!(
            WeightedIndex::new(&[<f32>::NAN, 0.5]).unwrap_err(),
            WeightedError::InvalidWeight,
        );
        assert_eq!(
            WeightedIndex::new(&[<f32>::NAN]).unwrap_err(),
            WeightedError::InvalidWeight,
        );
        assert_eq!(
            WeightedIndex::new(&[0.5, <f32>::NAN]).unwrap_err(),
            WeightedError::InvalidWeight,
        );

        assert_eq!(
            WeightedIndex::new(&[0.5, 7.0])
                .unwrap()
                .update_weights(&[(0, &<f32>::NAN)])
                .unwrap_err(),
            WeightedError::InvalidWeight,
        )
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_weightedindex() {
        let mut r = crate::test::rng(700);
        const N_REPS: u32 = 5000;
        let weights = [1u32, 2, 3, 0, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7];
        let total_weight = weights.iter().sum::<u32>() as f32;

        let verify = |result: [i32; 14]| {
            for (i, count) in result.iter().enumerate() {
                let exp = (weights[i] * N_REPS) as f32 / total_weight;
                let mut err = (*count as f32 - exp).abs();
                if err != 0.0 {
                    err /= exp;
                }
                assert!(err <= 0.25);
            }
        };

        // WeightedIndex from vec
        let mut chosen = [0i32; 14];
        let distr = WeightedIndex::new(weights.to_vec()).unwrap();
        for _ in 0..N_REPS {
            chosen[distr.sample(&mut r)] += 1;
        }
        verify(chosen);

        // WeightedIndex from slice
        chosen = [0i32; 14];
        let distr = WeightedIndex::new(&weights[..]).unwrap();
        for _ in 0..N_REPS {
            chosen[distr.sample(&mut r)] += 1;
        }
        verify(chosen);

        // WeightedIndex from iterator
        chosen = [0i32; 14];
        let distr = WeightedIndex::new(weights.iter()).unwrap();
        for _ in 0..N_REPS {
            chosen[distr.sample(&mut r)] += 1;
        }
        verify(chosen);

        for _ in 0..5 {
            assert_eq!(WeightedIndex::new(&[0, 1]).unwrap().sample(&mut r), 1);
            assert_eq!(WeightedIndex::new(&[1, 0]).unwrap().sample(&mut r), 0);
            assert_eq!(
                WeightedIndex::new(&[0, 0, 0, 0, 10, 0])
                    .unwrap()
                    .sample(&mut r),
                4
            );
        }

        assert_eq!(
            WeightedIndex::new(&[10][0..0]).unwrap_err(),
            WeightedError::NoItem
        );
        assert_eq!(
            WeightedIndex::new(&[0]).unwrap_err(),
            WeightedError::AllWeightsZero
        );
        assert_eq!(
            WeightedIndex::new(&[10, 20, -1, 30]).unwrap_err(),
            WeightedError::InvalidWeight
        );
        assert_eq!(
            WeightedIndex::new(&[-10, 20, 1, 30]).unwrap_err(),
            WeightedError::InvalidWeight
        );
        assert_eq!(
            WeightedIndex::new(&[-10]).unwrap_err(),
            WeightedError::InvalidWeight
        );
    }

    #[test]
    fn test_update_weights() {
        let data = [
            (
                &[10u32, 2, 3, 4][..],
                &[(1, &100), (2, &4)][..], // positive change
                &[10, 100, 4, 4][..],
            ),
            (
                &[1u32, 2, 3, 0, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7][..],
                &[(2, &1), (5, &1), (13, &100)][..], // negative change and last element
                &[1u32, 2, 1, 0, 5, 1, 7, 1, 2, 3, 4, 5, 6, 100][..],
            ),
        ];

        for (weights, update, expected_weights) in data.iter() {
            let total_weight = weights.iter().sum::<u32>();
            let mut distr = WeightedIndex::new(weights.to_vec()).unwrap();
            assert_eq!(distr.total_weight, total_weight);

            distr.update_weights(update).unwrap();
            let expected_total_weight = expected_weights.iter().sum::<u32>();
            let expected_distr = WeightedIndex::new(expected_weights.to_vec()).unwrap();
            assert_eq!(distr.total_weight, expected_total_weight);
            assert_eq!(distr.total_weight, expected_distr.total_weight);
            assert_eq!(distr.cumulative_weights, expected_distr.cumulative_weights);
        }
    }

    #[test]
    fn value_stability() {
        fn test_samples<X: SampleUniform + PartialOrd, I>(
            weights: I,
            buf: &mut [usize],
            expected: &[usize],
        ) where
            I: IntoIterator,
            I::Item: SampleBorrow<X>,
            X: for<'a> ::core::ops::AddAssign<&'a X> + Clone + Default,
        {
            assert_eq!(buf.len(), expected.len());
            let distr = WeightedIndex::new(weights).unwrap();
            let mut rng = crate::test::rng(701);
            for r in buf.iter_mut() {
                *r = rng.sample(&distr);
            }
            assert_eq!(buf, expected);
        }

        let mut buf = [0; 10];
        test_samples(
            &[1i32, 1, 1, 1, 1, 1, 1, 1, 1],
            &mut buf,
            &[0, 6, 2, 6, 3, 4, 7, 8, 2, 5],
        );
        test_samples(
            &[0.7f32, 0.1, 0.1, 0.1],
            &mut buf,
            &[0, 0, 0, 1, 0, 0, 2, 3, 0, 0],
        );
        test_samples(
            &[1.0f64, 0.999, 0.998, 0.997],
            &mut buf,
            &[2, 2, 1, 3, 2, 1, 3, 3, 2, 1],
        );
    }

    #[test]
    fn weighted_index_distributions_can_be_compared() {
        assert_eq!(WeightedIndex::new(&[1, 2]), WeightedIndex::new(&[1, 2]));
    }
