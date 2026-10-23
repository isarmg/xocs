
    use super::*;
    use crate::RngCore;
    #[cfg(feature = "alloc")] use alloc::string::String;

    #[test]
    fn test_misc() {
        let rng: &mut dyn RngCore = &mut crate::test::rng(820);

        rng.sample::<char, _>(Standard);
        rng.sample::<bool, _>(Standard);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_chars() {
        use core::iter;
        let mut rng = crate::test::rng(805);

        // Test by generating a relatively large number of chars, so we also
        // take the rejection sampling path.
        let word: String = iter::repeat(())
            .map(|()| rng.gen::<char>())
            .take(1000)
            .collect();
        assert!(!word.is_empty());
    }

    #[test]
    fn test_alphanumeric() {
        let mut rng = crate::test::rng(806);

        // Test by generating a relatively large number of chars, so we also
        // take the rejection sampling path.
        let mut incorrect = false;
        for _ in 0..100 {
            let c: char = rng.sample(Alphanumeric).into();
            incorrect |= !(('0'..='9').contains(&c) ||
                           ('A'..='Z').contains(&c) ||
                           ('a'..='z').contains(&c) );
        }
        assert!(!incorrect);
    }

    #[test]
    fn value_stability() {
        fn test_samples<T: Copy + core::fmt::Debug + PartialEq, D: Distribution<T>>(
            distr: &D, zero: T, expected: &[T],
        ) {
            let mut rng = crate::test::rng(807);
            let mut buf = [zero; 5];
            for x in &mut buf {
                *x = rng.sample(&distr);
            }
            assert_eq!(&buf, expected);
        }

        test_samples(&Standard, 'a', &[
            '\u{8cdac}',
            '\u{a346a}',
            '\u{80120}',
            '\u{ed692}',
            '\u{35888}',
        ]);
        test_samples(&Alphanumeric, 0, &[104, 109, 101, 51, 77]);
        test_samples(&Standard, false, &[true, true, false, true, false]);
        test_samples(&Standard, None as Option<bool>, &[
            Some(true),
            None,
            Some(false),
            None,
            Some(false),
        ]);
        test_samples(&Standard, Wrapping(0i32), &[
            Wrapping(-2074640887),
            Wrapping(-1719949321),
            Wrapping(2018088303),
            Wrapping(-547181756),
            Wrapping(838957336),
        ]);

        // We test only sub-sets of tuple and array impls
        test_samples(&Standard, (), &[(), (), (), (), ()]);
        test_samples(&Standard, (false,), &[
            (true,),
            (true,),
            (false,),
            (true,),
            (false,),
        ]);
        test_samples(&Standard, (false, false), &[
            (true, true),
            (false, true),
            (false, false),
            (true, false),
            (false, false),
        ]);

        test_samples(&Standard, [0u8; 0], &[[], [], [], [], []]);
        test_samples(&Standard, [0u8; 3], &[
            [9, 247, 111],
            [68, 24, 13],
            [174, 19, 194],
            [172, 69, 213],
            [149, 207, 29],
        ]);
    }
