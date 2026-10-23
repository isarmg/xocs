
    use super::*;
    use crate::rngs::mock::StepRng;

    #[test]
    #[cfg(feature = "serde1")]
    fn test_serialization_uniform_duration() {
        let distr = UniformDuration::new(Duration::from_secs(10), Duration::from_secs(60));
        let de_distr: UniformDuration =
            bincode::deserialize(&bincode::serialize(&distr).unwrap()).unwrap();
        assert_eq!(distr.offset, de_distr.offset);
        match (distr.mode, de_distr.mode) {
            (
                UniformDurationMode::Small {
                    secs: a_secs,
                    nanos: a_nanos,
                },
                UniformDurationMode::Small { secs, nanos },
            ) => {
                assert_eq!(a_secs, secs);

                assert_eq!(a_nanos.0.low, nanos.0.low);
                assert_eq!(a_nanos.0.range, nanos.0.range);
                assert_eq!(a_nanos.0.z, nanos.0.z);
            }
            (
                UniformDurationMode::Medium { nanos: a_nanos },
                UniformDurationMode::Medium { nanos },
            ) => {
                assert_eq!(a_nanos.0.low, nanos.0.low);
                assert_eq!(a_nanos.0.range, nanos.0.range);
                assert_eq!(a_nanos.0.z, nanos.0.z);
            }
            (
                UniformDurationMode::Large {
                    max_secs: a_max_secs,
                    max_nanos: a_max_nanos,
                    secs: a_secs,
                },
                UniformDurationMode::Large {
                    max_secs,
                    max_nanos,
                    secs,
                },
            ) => {
                assert_eq!(a_max_secs, max_secs);
                assert_eq!(a_max_nanos, max_nanos);

                assert_eq!(a_secs.0.low, secs.0.low);
                assert_eq!(a_secs.0.range, secs.0.range);
                assert_eq!(a_secs.0.z, secs.0.z);
            }
            _ => panic!("`UniformDurationMode` was not serialized/deserialized correctly"),
        }
    }

    #[test]
    #[cfg(feature = "serde1")]
    fn test_uniform_serialization() {
        let unit_box: Uniform<i32> = Uniform::new(-1, 1);
        let de_unit_box: Uniform<i32> =
            bincode::deserialize(&bincode::serialize(&unit_box).unwrap()).unwrap();

        assert_eq!(unit_box.0.low, de_unit_box.0.low);
        assert_eq!(unit_box.0.range, de_unit_box.0.range);
        assert_eq!(unit_box.0.z, de_unit_box.0.z);

        let unit_box: Uniform<f32> = Uniform::new(-1., 1.);
        let de_unit_box: Uniform<f32> =
            bincode::deserialize(&bincode::serialize(&unit_box).unwrap()).unwrap();

        assert_eq!(unit_box.0.low, de_unit_box.0.low);
        assert_eq!(unit_box.0.scale, de_unit_box.0.scale);
    }

    #[should_panic]
    #[test]
    fn test_uniform_bad_limits_equal_int() {
        Uniform::new(10, 10);
    }

    #[test]
    fn test_uniform_good_limits_equal_int() {
        let mut rng = crate::test::rng(804);
        let dist = Uniform::new_inclusive(10, 10);
        for _ in 0..20 {
            assert_eq!(rng.sample(dist), 10);
        }
    }

    #[should_panic]
    #[test]
    fn test_uniform_bad_limits_flipped_int() {
        Uniform::new(10, 5);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_integers() {
        let mut rng = crate::test::rng(251);
        macro_rules! t {
            ($ty:ident, $v:expr, $le:expr, $lt:expr) => {{
                for &(low, high) in $v.iter() {
                    let my_uniform = Uniform::new(low, high);
                    for _ in 0..1000 {
                        let v: $ty = rng.sample(my_uniform);
                        assert!($le(low, v) && $lt(v, high));
                    }

                    let my_uniform = Uniform::new_inclusive(low, high);
                    for _ in 0..1000 {
                        let v: $ty = rng.sample(my_uniform);
                        assert!($le(low, v) && $le(v, high));
                    }

                    let my_uniform = Uniform::new(&low, high);
                    for _ in 0..1000 {
                        let v: $ty = rng.sample(my_uniform);
                        assert!($le(low, v) && $lt(v, high));
                    }

                    let my_uniform = Uniform::new_inclusive(&low, &high);
                    for _ in 0..1000 {
                        let v: $ty = rng.sample(my_uniform);
                        assert!($le(low, v) && $le(v, high));
                    }

                    for _ in 0..1000 {
                        let v = <$ty as SampleUniform>::Sampler::sample_single(low, high, &mut rng);
                        assert!($le(low, v) && $lt(v, high));
                    }

                    for _ in 0..1000 {
                        let v = <$ty as SampleUniform>::Sampler::sample_single_inclusive(low, high, &mut rng);
                        assert!($le(low, v) && $le(v, high));
                    }
                }
            }};

            // scalar bulk
            ($($ty:ident),*) => {{
                $(t!(
                    $ty,
                    [(0, 10), (10, 127), ($ty::MIN, $ty::MAX)],
                    |x, y| x <= y,
                    |x, y| x < y
                );)*
            }};

            // simd bulk
            ($($ty:ident),* => $scalar:ident) => {{
                $(t!(
                    $ty,
                    [
                        ($ty::splat(0), $ty::splat(10)),
                        ($ty::splat(10), $ty::splat(127)),
                        ($ty::splat($scalar::MIN), $ty::splat($scalar::MAX)),
                    ],
                    |x: $ty, y| x.le(y).all(),
                    |x: $ty, y| x.lt(y).all()
                );)*
            }};
        }
        t!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, i128, u128);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_char() {
        let mut rng = crate::test::rng(891);
        let mut max = core::char::from_u32(0).unwrap();
        for _ in 0..100 {
            let c = rng.gen_range('A'..='Z');
            assert!(('A'..='Z').contains(&c));
            max = max.max(c);
        }
        assert_eq!(max, 'Z');
        let d = Uniform::new(
            core::char::from_u32(0xD7F0).unwrap(),
            core::char::from_u32(0xE010).unwrap(),
        );
        for _ in 0..100 {
            let c = d.sample(&mut rng);
            assert!((c as u32) < 0xD800 || (c as u32) > 0xDFFF);
        }
    }

    #[test]
    #[cfg(feature = "serde1")]
    fn test_char_bad_deser() {
        let json = r#"{"sampler":{"low":4294967200,"range":0,"z":0}}"#;
        let result = serde_json::from_str::<Uniform<char>>(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.classify(), serde_json::error::Category::Data);

        #[cfg(feature = "alloc")]
        {
            assert_eq!(
                alloc::string::ToString::to_string(&err),
                "bad sampler range for UniformChar at line 1 column 46"
            );
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_floats() {
        let mut rng = crate::test::rng(252);
        let mut zero_rng = StepRng::new(0, 0);
        let mut max_rng = StepRng::new(0xffff_ffff_ffff_ffff, 0);
        macro_rules! t {
            ($ty:ty, $f_scalar:ident, $bits_shifted:expr) => {{
                let v: &[($f_scalar, $f_scalar)] = &[
                    (0.0, 100.0),
                    (-1e35, -1e25),
                    (1e-35, 1e-25),
                    (-1e35, 1e35),
                    (<$f_scalar>::from_bits(0), <$f_scalar>::from_bits(3)),
                    (-<$f_scalar>::from_bits(10), -<$f_scalar>::from_bits(1)),
                    (-<$f_scalar>::from_bits(5), 0.0),
                    (-<$f_scalar>::from_bits(7), -0.0),
                    (0.1 * <$f_scalar>::MAX, <$f_scalar>::MAX),
                    (-<$f_scalar>::MAX * 0.2, <$f_scalar>::MAX * 0.7),
                ];
                for &(low_scalar, high_scalar) in v.iter() {
                    for lane in 0..<$ty>::lanes() {
                        let low = <$ty>::splat(0.0 as $f_scalar).replace(lane, low_scalar);
                        let high = <$ty>::splat(1.0 as $f_scalar).replace(lane, high_scalar);
                        let my_uniform = Uniform::new(low, high);
                        let my_incl_uniform = Uniform::new_inclusive(low, high);
                        for _ in 0..100 {
                            let v = rng.sample(my_uniform).extract(lane);
                            assert!(low_scalar <= v && v < high_scalar);
                            let v = rng.sample(my_incl_uniform).extract(lane);
                            assert!(low_scalar <= v && v <= high_scalar);
                            let v =
                                <$ty as SampleUniform>::Sampler::sample_single(low, high, &mut rng)
                                    .extract(lane);
                            assert!(low_scalar <= v && v < high_scalar);
                        }

                        assert_eq!(
                            rng.sample(Uniform::new_inclusive(low, low)).extract(lane),
                            low_scalar
                        );

                        assert_eq!(zero_rng.sample(my_uniform).extract(lane), low_scalar);
                        assert_eq!(zero_rng.sample(my_incl_uniform).extract(lane), low_scalar);
                        assert_eq!(
                            <$ty as SampleUniform>::Sampler::sample_single(
                                low,
                                high,
                                &mut zero_rng
                            )
                            .extract(lane),
                            low_scalar
                        );
                        assert!(max_rng.sample(my_uniform).extract(lane) < high_scalar);
                        assert!(max_rng.sample(my_incl_uniform).extract(lane) <= high_scalar);

                        // Don't run this test for really tiny differences between high and low
                        // since for those rounding might result in selecting high for a very
                        // long time.
                        if (high_scalar - low_scalar) > 0.0001 {
                            let mut lowering_max_rng = StepRng::new(
                                0xffff_ffff_ffff_ffff,
                                (-1i64 << $bits_shifted) as u64,
                            );
                            assert!(
                                <$ty as SampleUniform>::Sampler::sample_single(
                                    low,
                                    high,
                                    &mut lowering_max_rng
                                )
                                .extract(lane)
                                    < high_scalar
                            );
                        }
                    }
                }

                assert_eq!(
                    rng.sample(Uniform::new_inclusive(<$f_scalar>::MAX, <$f_scalar>::MAX)),
                    <$f_scalar>::MAX
                );
                assert_eq!(
                    rng.sample(Uniform::new_inclusive(-<$f_scalar>::MAX, -<$f_scalar>::MAX)),
                    -<$f_scalar>::MAX
                );
            }};
        }

        t!(f32, f32, 32 - 23);
        t!(f64, f64, 64 - 52);
    }

    #[test]
    #[should_panic]
    fn test_float_overflow() {
        let _ = Uniform::from(<f64>::MIN..<f64>::MAX);
    }

    #[test]
    #[should_panic]
    fn test_float_overflow_single() {
        let mut rng = crate::test::rng(252);
        rng.gen_range(<f64>::MIN..<f64>::MAX);
    }

    #[test]
    #[cfg(all(feature = "std", not(target_arch = "wasm32"),))]
    fn test_float_assertions() {
        use super::SampleUniform;
        use std::panic::catch_unwind;
        fn range<T: SampleUniform>(low: T, high: T) {
            let mut rng = crate::test::rng(253);
            T::Sampler::sample_single(low, high, &mut rng);
        }

        macro_rules! t {
            ($ty:ident, $f_scalar:ident) => {{
                let v: &[($f_scalar, $f_scalar)] = &[
                    (<$f_scalar>::NAN, 0.0),
                    (1.0, <$f_scalar>::NAN),
                    (<$f_scalar>::NAN, <$f_scalar>::NAN),
                    (1.0, 0.5),
                    (<$f_scalar>::MAX, -<$f_scalar>::MAX),
                    (<$f_scalar>::INFINITY, <$f_scalar>::INFINITY),
                    (<$f_scalar>::NEG_INFINITY, <$f_scalar>::NEG_INFINITY),
                    (<$f_scalar>::NEG_INFINITY, 5.0),
                    (5.0, <$f_scalar>::INFINITY),
                    (<$f_scalar>::NAN, <$f_scalar>::INFINITY),
                    (<$f_scalar>::NEG_INFINITY, <$f_scalar>::NAN),
                    (<$f_scalar>::NEG_INFINITY, <$f_scalar>::INFINITY),
                ];
                for &(low_scalar, high_scalar) in v.iter() {
                    for lane in 0..<$ty>::lanes() {
                        let low = <$ty>::splat(0.0 as $f_scalar).replace(lane, low_scalar);
                        let high = <$ty>::splat(1.0 as $f_scalar).replace(lane, high_scalar);
                        assert!(catch_unwind(|| range(low, high)).is_err());
                        assert!(catch_unwind(|| Uniform::new(low, high)).is_err());
                        assert!(catch_unwind(|| Uniform::new_inclusive(low, high)).is_err());
                        assert!(catch_unwind(|| range(low, low)).is_err());
                        assert!(catch_unwind(|| Uniform::new(low, low)).is_err());
                    }
                }
            }};
        }

        t!(f32, f32);
        t!(f64, f64);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_durations() {
        let mut rng = crate::test::rng(253);

        let v = &[
            (Duration::new(10, 50000), Duration::new(100, 1234)),
            (Duration::new(0, 100), Duration::new(1, 50)),
            (
                Duration::new(0, 0),
                Duration::new(u64::max_value(), 999_999_999),
            ),
        ];
        for &(low, high) in v.iter() {
            let my_uniform = Uniform::new(low, high);
            for _ in 0..1000 {
                let v = rng.sample(my_uniform);
                assert!(low <= v && v < high);
            }
        }
    }

    #[test]
    fn test_custom_uniform() {
        use crate::distributions::uniform::{
            SampleBorrow, SampleUniform, UniformFloat, UniformSampler,
        };
        #[derive(Clone, Copy, PartialEq, PartialOrd)]
        struct MyF32 {
            x: f32,
        }
        #[derive(Clone, Copy, Debug)]
        struct UniformMyF32(UniformFloat<f32>);
        impl UniformSampler for UniformMyF32 {
            type X = MyF32;

            fn new<B1, B2>(low: B1, high: B2) -> Self
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                UniformMyF32(UniformFloat::<f32>::new(low.borrow().x, high.borrow().x))
            }

            fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                UniformSampler::new(low, high)
            }

            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
                MyF32 {
                    x: self.0.sample(rng),
                }
            }
        }
        impl SampleUniform for MyF32 {
            type Sampler = UniformMyF32;
        }

        let (low, high) = (MyF32 { x: 17.0f32 }, MyF32 { x: 22.0f32 });
        let uniform = Uniform::new(low, high);
        let mut rng = crate::test::rng(804);
        for _ in 0..100 {
            let x: MyF32 = rng.sample(uniform);
            assert!(low <= x && x < high);
        }
    }

    #[test]
    fn test_uniform_from_std_range() {
        let r = Uniform::from(2u32..7);
        assert_eq!(r.0.low, 2);
        assert_eq!(r.0.range, 5);
        assert_eq!(r.0.max(), 6);
        let r = Uniform::from(2.0f64..7.0);
        assert_eq!(r.0.low, 2.0);
        assert_eq!(r.0.scale, 5.0);
    }

    #[test]
    fn test_uniform_from_std_range_inclusive() {
        let r = Uniform::from(2u32..=6);
        assert_eq!(r.0.low, 2);
        assert_eq!(r.0.range, 5);
        assert_eq!(r.0.max(), 6);
        let r = Uniform::from(2.0f64..=7.0);
        assert_eq!(r.0.low, 2.0);
        assert!(r.0.scale > 5.0);
        assert!(r.0.scale < 5.0 + 1e-14);
    }

    #[test]
    fn value_stability() {
        fn test_samples<T: SampleUniform + Copy + core::fmt::Debug + PartialEq>(
            lb: T,
            ub: T,
            expected_single: &[T],
            expected_multiple: &[T],
        ) where
            Uniform<T>: Distribution<T>,
        {
            let mut rng = crate::test::rng(897);
            let mut buf = [lb; 3];

            for x in &mut buf {
                *x = T::Sampler::sample_single(lb, ub, &mut rng);
            }
            assert_eq!(&buf, expected_single);

            let distr = Uniform::new(lb, ub);
            for x in &mut buf {
                *x = rng.sample(&distr);
            }
            assert_eq!(&buf, expected_multiple);
        }

        // We test on a sub-set of types; possibly we should do more.
        // TODO: SIMD types

        test_samples(11u8, 219, &[17, 66, 214], &[181, 93, 165]);
        test_samples(11u32, 219, &[17, 66, 214], &[181, 93, 165]);

        test_samples(
            0f32,
            1e-2f32,
            &[0.0003070104, 0.0026630748, 0.00979833],
            &[0.008194133, 0.00398172, 0.007428536],
        );
        test_samples(
            -1e10f64,
            1e10f64,
            &[-4673848682.871551, 6388267422.932352, 4857075081.198343],
            &[1173375212.1808167, 1917642852.109581, 2365076174.3153973],
        );

        test_samples(
            Duration::new(2, 0),
            Duration::new(4, 0),
            &[
                Duration::new(2, 532615131),
                Duration::new(3, 638826742),
                Duration::new(3, 485707508),
            ],
            &[
                Duration::new(3, 117337521),
                Duration::new(3, 191764285),
                Duration::new(3, 236507617),
            ],
        );
    }

    #[test]
    fn uniform_distributions_can_be_compared() {
        assert_eq!(Uniform::new(1.0, 2.0), Uniform::new(1.0, 2.0));

        // To cover UniformInt
        assert_eq!(
            Uniform::new(1 as u32, 2 as u32),
            Uniform::new(1 as u32, 2 as u32)
        );
    }
