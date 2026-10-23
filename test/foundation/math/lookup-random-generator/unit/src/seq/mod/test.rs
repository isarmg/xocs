
    use super::*;
    #[cfg(feature = "alloc")]
    use crate::Rng;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::vec::Vec;

    #[test]
    fn test_slice_choose() {
        let mut r = crate::test::rng(107);
        let chars = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
        ];
        let mut chosen = [0i32; 14];
        // The below all use a binomial distribution with n=1000, p=1/14.
        // binocdf(40, 1000, 1/14) ~= 2e-5; 1-binocdf(106, ..) ~= 2e-5
        for _ in 0..1000 {
            let picked = *chars.choose(&mut r).unwrap();
            chosen[(picked as usize) - ('a' as usize)] += 1;
        }
        for count in chosen.iter() {
            assert!(40 < *count && *count < 106);
        }

        chosen.iter_mut().for_each(|x| *x = 0);
        for _ in 0..1000 {
            *chosen.choose_mut(&mut r).unwrap() += 1;
        }
        for count in chosen.iter() {
            assert!(40 < *count && *count < 106);
        }

        let mut v: [isize; 0] = [];
        assert_eq!(v.choose(&mut r), None);
        assert_eq!(v.choose_mut(&mut r), None);
    }

    #[test]
    fn value_stability_slice() {
        let mut r = crate::test::rng(413);
        let chars = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n',
        ];
        let mut nums = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        assert_eq!(chars.choose(&mut r), Some(&'l'));
        assert_eq!(nums.choose_mut(&mut r), Some(&mut 10));

        #[cfg(feature = "alloc")]
        assert_eq!(
            &chars
                .choose_multiple(&mut r, 8)
                .cloned()
                .collect::<Vec<char>>(),
            &['d', 'm', 'b', 'n', 'c', 'k', 'h', 'e']
        );

        #[cfg(feature = "alloc")]
        assert_eq!(chars.choose_weighted(&mut r, |_| 1), Ok(&'f'));
        #[cfg(feature = "alloc")]
        assert_eq!(nums.choose_weighted_mut(&mut r, |_| 1), Ok(&mut 5));

        let mut r = crate::test::rng(414);
        nums.shuffle(&mut r);
        assert_eq!(nums, [9, 5, 3, 10, 7, 12, 8, 11, 6, 4, 0, 2, 1]);
        nums = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let res = nums.partial_shuffle(&mut r, 6);
        assert_eq!(res.0, &mut [7, 4, 8, 6, 9, 3]);
        assert_eq!(res.1, &mut [0, 1, 2, 12, 11, 5, 10]);
    }

    #[derive(Clone)]
    struct UnhintedIterator<I: Iterator + Clone> {
        iter: I,
    }
    impl<I: Iterator + Clone> Iterator for UnhintedIterator<I> {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }
    }

    #[derive(Clone)]
    struct ChunkHintedIterator<I: ExactSizeIterator + Iterator + Clone> {
        iter: I,
        chunk_remaining: usize,
        chunk_size: usize,
        hint_total_size: bool,
    }
    impl<I: ExactSizeIterator + Iterator + Clone> Iterator for ChunkHintedIterator<I> {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            if self.chunk_remaining == 0 {
                self.chunk_remaining = ::core::cmp::min(self.chunk_size, self.iter.len());
            }
            self.chunk_remaining = self.chunk_remaining.saturating_sub(1);

            self.iter.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (
                self.chunk_remaining,
                if self.hint_total_size {
                    Some(self.iter.len())
                } else {
                    None
                },
            )
        }
    }

    #[derive(Clone)]
    struct WindowHintedIterator<I: ExactSizeIterator + Iterator + Clone> {
        iter: I,
        window_size: usize,
        hint_total_size: bool,
    }
    impl<I: ExactSizeIterator + Iterator + Clone> Iterator for WindowHintedIterator<I> {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (
                ::core::cmp::min(self.iter.len(), self.window_size),
                if self.hint_total_size {
                    Some(self.iter.len())
                } else {
                    None
                },
            )
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_iterator_choose() {
        let r = &mut crate::test::rng(109);
        fn test_iter<R: Rng + ?Sized, Iter: Iterator<Item = usize> + Clone>(r: &mut R, iter: Iter) {
            let mut chosen = [0i32; 9];
            for _ in 0..1000 {
                let picked = iter.clone().choose(r).unwrap();
                chosen[picked] += 1;
            }
            for count in chosen.iter() {
                // Samples should follow Binomial(1000, 1/9)
                // Octave: binopdf(x, 1000, 1/9) gives the prob of *count == x
                // Note: have seen 153, which is unlikely but not impossible.
                assert!(
                    72 < *count && *count < 154,
                    "count not close to 1000/9: {}",
                    count
                );
            }
        }

        test_iter(r, 0..9);
        test_iter(r, [0, 1, 2, 3, 4, 5, 6, 7, 8].iter().cloned());
        #[cfg(feature = "alloc")]
        test_iter(r, (0..9).collect::<Vec<_>>().into_iter());
        test_iter(r, UnhintedIterator { iter: 0..9 });
        test_iter(
            r,
            ChunkHintedIterator {
                iter: 0..9,
                chunk_size: 4,
                chunk_remaining: 4,
                hint_total_size: false,
            },
        );
        test_iter(
            r,
            ChunkHintedIterator {
                iter: 0..9,
                chunk_size: 4,
                chunk_remaining: 4,
                hint_total_size: true,
            },
        );
        test_iter(
            r,
            WindowHintedIterator {
                iter: 0..9,
                window_size: 2,
                hint_total_size: false,
            },
        );
        test_iter(
            r,
            WindowHintedIterator {
                iter: 0..9,
                window_size: 2,
                hint_total_size: true,
            },
        );

        assert_eq!((0..0).choose(r), None);
        assert_eq!(UnhintedIterator { iter: 0..0 }.choose(r), None);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_iterator_choose_stable() {
        let r = &mut crate::test::rng(109);
        fn test_iter<R: Rng + ?Sized, Iter: Iterator<Item = usize> + Clone>(r: &mut R, iter: Iter) {
            let mut chosen = [0i32; 9];
            for _ in 0..1000 {
                let picked = iter.clone().choose_stable(r).unwrap();
                chosen[picked] += 1;
            }
            for count in chosen.iter() {
                // Samples should follow Binomial(1000, 1/9)
                // Octave: binopdf(x, 1000, 1/9) gives the prob of *count == x
                // Note: have seen 153, which is unlikely but not impossible.
                assert!(
                    72 < *count && *count < 154,
                    "count not close to 1000/9: {}",
                    count
                );
            }
        }

        test_iter(r, 0..9);
        test_iter(r, [0, 1, 2, 3, 4, 5, 6, 7, 8].iter().cloned());
        #[cfg(feature = "alloc")]
        test_iter(r, (0..9).collect::<Vec<_>>().into_iter());
        test_iter(r, UnhintedIterator { iter: 0..9 });
        test_iter(
            r,
            ChunkHintedIterator {
                iter: 0..9,
                chunk_size: 4,
                chunk_remaining: 4,
                hint_total_size: false,
            },
        );
        test_iter(
            r,
            ChunkHintedIterator {
                iter: 0..9,
                chunk_size: 4,
                chunk_remaining: 4,
                hint_total_size: true,
            },
        );
        test_iter(
            r,
            WindowHintedIterator {
                iter: 0..9,
                window_size: 2,
                hint_total_size: false,
            },
        );
        test_iter(
            r,
            WindowHintedIterator {
                iter: 0..9,
                window_size: 2,
                hint_total_size: true,
            },
        );

        assert_eq!((0..0).choose(r), None);
        assert_eq!(UnhintedIterator { iter: 0..0 }.choose(r), None);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_iterator_choose_stable_stability() {
        fn test_iter(iter: impl Iterator<Item = usize> + Clone) -> [i32; 9] {
            let r = &mut crate::test::rng(109);
            let mut chosen = [0i32; 9];
            for _ in 0..1000 {
                let picked = iter.clone().choose_stable(r).unwrap();
                chosen[picked] += 1;
            }
            chosen
        }

        let reference = test_iter(0..9);
        assert_eq!(
            test_iter([0, 1, 2, 3, 4, 5, 6, 7, 8].iter().cloned()),
            reference
        );

        #[cfg(feature = "alloc")]
        assert_eq!(test_iter((0..9).collect::<Vec<_>>().into_iter()), reference);
        assert_eq!(test_iter(UnhintedIterator { iter: 0..9 }), reference);
        assert_eq!(
            test_iter(ChunkHintedIterator {
                iter: 0..9,
                chunk_size: 4,
                chunk_remaining: 4,
                hint_total_size: false,
            }),
            reference
        );
        assert_eq!(
            test_iter(ChunkHintedIterator {
                iter: 0..9,
                chunk_size: 4,
                chunk_remaining: 4,
                hint_total_size: true,
            }),
            reference
        );
        assert_eq!(
            test_iter(WindowHintedIterator {
                iter: 0..9,
                window_size: 2,
                hint_total_size: false,
            }),
            reference
        );
        assert_eq!(
            test_iter(WindowHintedIterator {
                iter: 0..9,
                window_size: 2,
                hint_total_size: true,
            }),
            reference
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_shuffle() {
        let mut r = crate::test::rng(108);
        let empty: &mut [isize] = &mut [];
        empty.shuffle(&mut r);
        let mut one = [1];
        one.shuffle(&mut r);
        let b: &[_] = &[1];
        assert_eq!(one, b);

        let mut two = [1, 2];
        two.shuffle(&mut r);
        assert!(two == [1, 2] || two == [2, 1]);

        fn move_last(slice: &mut [usize], pos: usize) {
            // use slice[pos..].rotate_left(1); once we can use that
            let last_val = slice[pos];
            for i in pos..slice.len() - 1 {
                slice[i] = slice[i + 1];
            }
            *slice.last_mut().unwrap() = last_val;
        }
        let mut counts = [0i32; 24];
        for _ in 0..10000 {
            let mut arr: [usize; 4] = [0, 1, 2, 3];
            arr.shuffle(&mut r);
            let mut permutation = 0usize;
            let mut pos_value = counts.len();
            for i in 0..4 {
                pos_value /= 4 - i;
                let pos = arr.iter().position(|&x| x == i).unwrap();
                assert!(pos < (4 - i));
                permutation += pos * pos_value;
                move_last(&mut arr, pos);
                assert_eq!(arr[3], i);
            }
            for (i, &a) in arr.iter().enumerate() {
                assert_eq!(a, i);
            }
            counts[permutation] += 1;
        }
        for count in counts.iter() {
            // Binomial(10000, 1/24) with average 416.667
            // Octave: binocdf(n, 10000, 1/24)
            // 99.9% chance samples lie within this range:
            assert!(352 <= *count && *count <= 483, "count: {}", count);
        }
    }

    #[test]
    fn test_partial_shuffle() {
        let mut r = crate::test::rng(118);

        let mut empty: [u32; 0] = [];
        let res = empty.partial_shuffle(&mut r, 10);
        assert_eq!((res.0.len(), res.1.len()), (0, 0));

        let mut v = [1, 2, 3, 4, 5];
        let res = v.partial_shuffle(&mut r, 2);
        assert_eq!((res.0.len(), res.1.len()), (2, 3));
        assert!(res.0[0] != res.0[1]);
        // First elements are only modified if selected, so at least one isn't modified:
        assert!(res.1[0] == 1 || res.1[1] == 2 || res.1[2] == 3);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_sample_iter() {
        let min_val = 1;
        let max_val = 100;

        let mut r = crate::test::rng(401);
        let vals = (min_val..max_val).collect::<Vec<i32>>();
        let small_sample = vals.iter().choose_multiple(&mut r, 5);
        let large_sample = vals.iter().choose_multiple(&mut r, vals.len() + 5);

        assert_eq!(small_sample.len(), 5);
        assert_eq!(large_sample.len(), vals.len());
        // no randomization happens when amount >= len
        assert_eq!(large_sample, vals.iter().collect::<Vec<_>>());

        assert!(small_sample
            .iter()
            .all(|e| { **e >= min_val && **e <= max_val }));
    }

    #[test]
    #[cfg(feature = "alloc")]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_weighted() {
        let mut r = crate::test::rng(406);
        const N_REPS: u32 = 3000;
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

        // choose_weighted
        fn get_weight<T>(item: &(u32, T)) -> u32 {
            item.0
        }
        let mut chosen = [0i32; 14];
        let mut items = [(0u32, 0usize); 14]; // (weight, index)
        for (i, item) in items.iter_mut().enumerate() {
            *item = (weights[i], i);
        }
        for _ in 0..N_REPS {
            let item = items.choose_weighted(&mut r, get_weight).unwrap();
            chosen[item.1] += 1;
        }
        verify(chosen);

        // choose_weighted_mut
        let mut items = [(0u32, 0i32); 14]; // (weight, count)
        for (i, item) in items.iter_mut().enumerate() {
            *item = (weights[i], 0);
        }
        for _ in 0..N_REPS {
            items.choose_weighted_mut(&mut r, get_weight).unwrap().1 += 1;
        }
        for (ch, item) in chosen.iter_mut().zip(items.iter()) {
            *ch = item.1;
        }
        verify(chosen);

        // Check error cases
        let empty_slice = &mut [10][0..0];
        assert_eq!(
            empty_slice.choose_weighted(&mut r, |_| 1),
            Err(WeightedError::NoItem)
        );
        assert_eq!(
            empty_slice.choose_weighted_mut(&mut r, |_| 1),
            Err(WeightedError::NoItem)
        );
        assert_eq!(
            ['x'].choose_weighted_mut(&mut r, |_| 0),
            Err(WeightedError::AllWeightsZero)
        );
        assert_eq!(
            [0, -1].choose_weighted_mut(&mut r, |x| *x),
            Err(WeightedError::InvalidWeight)
        );
        assert_eq!(
            [-1, 0].choose_weighted_mut(&mut r, |x| *x),
            Err(WeightedError::InvalidWeight)
        );
    }

    #[test]
    fn value_stability_choose() {
        fn choose<I: Iterator<Item = u32>>(iter: I) -> Option<u32> {
            let mut rng = crate::test::rng(411);
            iter.choose(&mut rng)
        }

        assert_eq!(choose([].iter().cloned()), None);
        assert_eq!(choose(0..100), Some(33));
        assert_eq!(choose(UnhintedIterator { iter: 0..100 }), Some(40));
        assert_eq!(
            choose(ChunkHintedIterator {
                iter: 0..100,
                chunk_size: 32,
                chunk_remaining: 32,
                hint_total_size: false,
            }),
            Some(39)
        );
        assert_eq!(
            choose(ChunkHintedIterator {
                iter: 0..100,
                chunk_size: 32,
                chunk_remaining: 32,
                hint_total_size: true,
            }),
            Some(39)
        );
        assert_eq!(
            choose(WindowHintedIterator {
                iter: 0..100,
                window_size: 32,
                hint_total_size: false,
            }),
            Some(90)
        );
        assert_eq!(
            choose(WindowHintedIterator {
                iter: 0..100,
                window_size: 32,
                hint_total_size: true,
            }),
            Some(90)
        );
    }

    #[test]
    fn value_stability_choose_stable() {
        fn choose<I: Iterator<Item = u32>>(iter: I) -> Option<u32> {
            let mut rng = crate::test::rng(411);
            iter.choose_stable(&mut rng)
        }

        assert_eq!(choose([].iter().cloned()), None);
        assert_eq!(choose(0..100), Some(40));
        assert_eq!(choose(UnhintedIterator { iter: 0..100 }), Some(40));
        assert_eq!(
            choose(ChunkHintedIterator {
                iter: 0..100,
                chunk_size: 32,
                chunk_remaining: 32,
                hint_total_size: false,
            }),
            Some(40)
        );
        assert_eq!(
            choose(ChunkHintedIterator {
                iter: 0..100,
                chunk_size: 32,
                chunk_remaining: 32,
                hint_total_size: true,
            }),
            Some(40)
        );
        assert_eq!(
            choose(WindowHintedIterator {
                iter: 0..100,
                window_size: 32,
                hint_total_size: false,
            }),
            Some(40)
        );
        assert_eq!(
            choose(WindowHintedIterator {
                iter: 0..100,
                window_size: 32,
                hint_total_size: true,
            }),
            Some(40)
        );
    }

    #[test]
    fn value_stability_choose_multiple() {
        fn do_test<I: Iterator<Item = u32>>(iter: I, v: &[u32]) {
            let mut rng = crate::test::rng(412);
            let mut buf = [0u32; 8];
            assert_eq!(iter.choose_multiple_fill(&mut rng, &mut buf), v.len());
            assert_eq!(&buf[0..v.len()], v);
        }

        do_test(0..4, &[0, 1, 2, 3]);
        do_test(0..8, &[0, 1, 2, 3, 4, 5, 6, 7]);
        do_test(0..100, &[58, 78, 80, 92, 43, 8, 96, 7]);

        #[cfg(feature = "alloc")]
        {
            fn do_test<I: Iterator<Item = u32>>(iter: I, v: &[u32]) {
                let mut rng = crate::test::rng(412);
                assert_eq!(iter.choose_multiple(&mut rng, v.len()), v);
            }

            do_test(0..4, &[0, 1, 2, 3]);
            do_test(0..8, &[0, 1, 2, 3, 4, 5, 6, 7]);
            do_test(0..100, &[58, 78, 80, 92, 43, 8, 96, 7]);
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_multiple_weighted_edge_cases() {
        use super::*;

        let mut rng = crate::test::rng(413);

        // Case 1: One of the weights is 0
        let choices = [('a', 2), ('b', 1), ('c', 0)];
        for _ in 0..100 {
            let result = choices
                .choose_multiple_weighted(&mut rng, 2, |item| item.1)
                .unwrap()
                .collect::<Vec<_>>();

            assert_eq!(result.len(), 2);
            assert!(!result.iter().any(|val| val.0 == 'c'));
        }

        // Case 2: All of the weights are 0
        let choices = [('a', 0), ('b', 0), ('c', 0)];

        assert_eq!(
            choices
                .choose_multiple_weighted(&mut rng, 2, |item| item.1)
                .unwrap()
                .count(),
            2
        );

        // Case 3: Negative weights
        let choices = [('a', -1), ('b', 1), ('c', 1)];
        assert_eq!(
            choices
                .choose_multiple_weighted(&mut rng, 2, |item| item.1)
                .unwrap_err(),
            WeightedError::InvalidWeight
        );

        // Case 4: Empty list
        let choices = [];
        assert_eq!(
            choices
                .choose_multiple_weighted(&mut rng, 0, |_: &()| 0)
                .unwrap()
                .count(),
            0
        );

        // Case 5: NaN weights
        let choices = [('a', <f64>::NAN), ('b', 1.0), ('c', 1.0)];
        assert_eq!(
            choices
                .choose_multiple_weighted(&mut rng, 2, |item| item.1)
                .unwrap_err(),
            WeightedError::InvalidWeight
        );

        // Case 6: +infinity weights
        let choices = [('a', <f64>::INFINITY), ('b', 1.0), ('c', 1.0)];
        for _ in 0..100 {
            let result = choices
                .choose_multiple_weighted(&mut rng, 2, |item| item.1)
                .unwrap()
                .collect::<Vec<_>>();
            assert_eq!(result.len(), 2);
            assert!(result.iter().any(|val| val.0 == 'a'));
        }

        // Case 7: -infinity weights
        let choices = [('a', <f64>::NEG_INFINITY), ('b', 1.0), ('c', 1.0)];
        assert_eq!(
            choices
                .choose_multiple_weighted(&mut rng, 2, |item| item.1)
                .unwrap_err(),
            WeightedError::InvalidWeight
        );

        // Case 8: -0 weights
        let choices = [('a', -0.0), ('b', 1.0), ('c', 1.0)];
        assert!(choices
            .choose_multiple_weighted(&mut rng, 2, |item| item.1)
            .is_ok());
    }

    #[test]
    #[cfg(feature = "std")]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_multiple_weighted_distributions() {
        use super::*;

        // The theoretical probabilities of the different outcomes are:
        // AB: 0.5  * 0.5  = 0.250
        // AC: 0.5  * 0.5  = 0.250
        // BA: 0.25 * 0.67 = 0.167
        // BC: 0.25 * 0.33 = 0.082
        // CA: 0.25 * 0.67 = 0.167
        // CB: 0.25 * 0.33 = 0.082
        let choices = [('a', 2), ('b', 1), ('c', 1)];
        let mut rng = crate::test::rng(414);

        let mut results = [0i32; 3];
        let expected_results = [4167, 4167, 1666];
        for _ in 0..10000 {
            let result = choices
                .choose_multiple_weighted(&mut rng, 2, |item| item.1)
                .unwrap()
                .collect::<Vec<_>>();

            assert_eq!(result.len(), 2);

            match (result[0].0, result[1].0) {
                ('a', 'b') | ('b', 'a') => {
                    results[0] += 1;
                }
                ('a', 'c') | ('c', 'a') => {
                    results[1] += 1;
                }
                ('b', 'c') | ('c', 'b') => {
                    results[2] += 1;
                }
                (_, _) => panic!("unexpected result"),
            }
        }

        let mut diffs = results
            .iter()
            .zip(&expected_results)
            .map(|(a, b)| (a - b).abs());
        assert!(!diffs.any(|deviation| deviation > 100));
    }
