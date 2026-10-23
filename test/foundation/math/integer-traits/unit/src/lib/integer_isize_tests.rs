macro_rules! integer_isize_tests {
    ($T:ty, $test_mod:ident) => {
        #[cfg(test)]
        mod $test_mod {
            use crate::Integer;
            use core::mem;

            /// Checks that the division rule holds for:
            ///
            /// - `n`: numerator (dividend)
            /// - `d`: denominator (divisor)
            /// - `qr`: quotient and remainder
            #[cfg(test)]
            fn test_division_rule((n, d): ($T, $T), (q, r): ($T, $T)) {
                assert_eq!(d * q + r, n);
            }

            #[test]
            fn test_div_rem() {
                fn test_nd_dr(nd: ($T, $T), qr: ($T, $T)) {
                    let (n, d) = nd;
                    let separate_div_rem = (n / d, n % d);
                    let combined_div_rem = n.div_rem(&d);

                    test_division_rule(nd, qr);

                    assert_eq!(separate_div_rem, qr);
                    assert_eq!(combined_div_rem, qr);
                }

                test_nd_dr((8, 3), (2, 2));
                test_nd_dr((8, -3), (-2, 2));
                test_nd_dr((-8, 3), (-2, -2));
                test_nd_dr((-8, -3), (2, -2));

                test_nd_dr((1, 2), (0, 1));
                test_nd_dr((1, -2), (0, 1));
                test_nd_dr((-1, 2), (0, -1));
                test_nd_dr((-1, -2), (0, -1));
            }

            #[test]
            fn test_div_mod_floor() {
                fn test_nd_dm(nd: ($T, $T), dm: ($T, $T)) {
                    let (n, d) = nd;
                    let separate_div_mod_floor =
                        (Integer::div_floor(&n, &d), Integer::mod_floor(&n, &d));
                    let combined_div_mod_floor = Integer::div_mod_floor(&n, &d);

                    test_division_rule(nd, dm);

                    assert_eq!(separate_div_mod_floor, dm);
                    assert_eq!(combined_div_mod_floor, dm);
                }

                test_nd_dm((8, 3), (2, 2));
                test_nd_dm((8, -3), (-3, -1));
                test_nd_dm((-8, 3), (-3, 1));
                test_nd_dm((-8, -3), (2, -2));

                test_nd_dm((1, 2), (0, 1));
                test_nd_dm((1, -2), (-1, -1));
                test_nd_dm((-1, 2), (-1, 1));
                test_nd_dm((-1, -2), (0, -1));
            }

            #[test]
            fn test_gcd() {
                assert_eq!((10 as $T).gcd(&2), 2 as $T);
                assert_eq!((10 as $T).gcd(&3), 1 as $T);
                assert_eq!((0 as $T).gcd(&3), 3 as $T);
                assert_eq!((3 as $T).gcd(&3), 3 as $T);
                assert_eq!((56 as $T).gcd(&42), 14 as $T);
                assert_eq!((3 as $T).gcd(&-3), 3 as $T);
                assert_eq!((-6 as $T).gcd(&3), 3 as $T);
                assert_eq!((-4 as $T).gcd(&-2), 2 as $T);
            }

            #[test]
            fn test_gcd_cmp_with_euclidean() {
                fn euclidean_gcd(mut m: $T, mut n: $T) -> $T {
                    while m != 0 {
                        mem::swap(&mut m, &mut n);
                        m %= n;
                    }

                    n.abs()
                }

                // gcd(-128, b) = 128 is not representable as positive value
                // for i8
                for i in -127..=127 {
                    for j in -127..=127 {
                        assert_eq!(euclidean_gcd(i, j), i.gcd(&j));
                    }
                }
            }

            #[test]
            fn test_gcd_min_val() {
                let min = <$T>::MIN;
                let max = <$T>::MAX;
                let max_pow2 = max / 2 + 1;
                assert_eq!(min.gcd(&max), 1 as $T);
                assert_eq!(max.gcd(&min), 1 as $T);
                assert_eq!(min.gcd(&max_pow2), max_pow2);
                assert_eq!(max_pow2.gcd(&min), max_pow2);
                assert_eq!(min.gcd(&42), 2 as $T);
                assert_eq!((42 as $T).gcd(&min), 2 as $T);
            }

            #[test]
            #[should_panic]
            fn test_gcd_min_val_min_val() {
                let min = <$T>::MIN;
                assert!(min.gcd(&min) >= 0);
            }

            #[test]
            #[should_panic]
            fn test_gcd_min_val_0() {
                let min = <$T>::MIN;
                assert!(min.gcd(&0) >= 0);
            }

            #[test]
            #[should_panic]
            fn test_gcd_0_min_val() {
                let min = <$T>::MIN;
                assert!((0 as $T).gcd(&min) >= 0);
            }

            #[test]
            fn test_lcm() {
                assert_eq!((1 as $T).lcm(&0), 0 as $T);
                assert_eq!((0 as $T).lcm(&1), 0 as $T);
                assert_eq!((1 as $T).lcm(&1), 1 as $T);
                assert_eq!((-1 as $T).lcm(&1), 1 as $T);
                assert_eq!((1 as $T).lcm(&-1), 1 as $T);
                assert_eq!((-1 as $T).lcm(&-1), 1 as $T);
                assert_eq!((8 as $T).lcm(&9), 72 as $T);
                assert_eq!((11 as $T).lcm(&5), 55 as $T);
            }

            #[test]
            fn test_gcd_lcm() {
                use core::iter::once;
                for i in once(0)
                    .chain((1..).take(127).flat_map(|a| once(a).chain(once(-a))))
                    .chain(once(-128))
                {
                    for j in once(0)
                        .chain((1..).take(127).flat_map(|a| once(a).chain(once(-a))))
                        .chain(once(-128))
                    {
                        assert_eq!(i.gcd_lcm(&j), (i.gcd(&j), i.lcm(&j)));
                    }
                }
            }

            #[test]
            fn test_extended_gcd_lcm() {
                use crate::ExtendedGcd;
                use core::fmt::Debug;
                use num_traits::NumAssign;

                fn check<A: Copy + Debug + Integer + NumAssign>(a: A, b: A) {
                    let ExtendedGcd { gcd, x, y, .. } = a.extended_gcd(&b);
                    assert_eq!(gcd, x * a + y * b);
                }

                use core::iter::once;
                for i in once(0)
                    .chain((1..).take(127).flat_map(|a| once(a).chain(once(-a))))
                    .chain(once(-128))
                {
                    for j in once(0)
                        .chain((1..).take(127).flat_map(|a| once(a).chain(once(-a))))
                        .chain(once(-128))
                    {
                        check(i, j);
                        let (ExtendedGcd { gcd, .. }, lcm) = i.extended_gcd_lcm(&j);
                        assert_eq!((gcd, lcm), (i.gcd(&j), i.lcm(&j)));
                    }
                }
            }

            #[test]
            fn test_even() {
                assert_eq!((-4 as $T).is_even(), true);
                assert_eq!((-3 as $T).is_even(), false);
                assert_eq!((-2 as $T).is_even(), true);
                assert_eq!((-1 as $T).is_even(), false);
                assert_eq!((0 as $T).is_even(), true);
                assert_eq!((1 as $T).is_even(), false);
                assert_eq!((2 as $T).is_even(), true);
                assert_eq!((3 as $T).is_even(), false);
                assert_eq!((4 as $T).is_even(), true);
            }

            #[test]
            fn test_odd() {
                assert_eq!((-4 as $T).is_odd(), false);
                assert_eq!((-3 as $T).is_odd(), true);
                assert_eq!((-2 as $T).is_odd(), false);
                assert_eq!((-1 as $T).is_odd(), true);
                assert_eq!((0 as $T).is_odd(), false);
                assert_eq!((1 as $T).is_odd(), true);
                assert_eq!((2 as $T).is_odd(), false);
                assert_eq!((3 as $T).is_odd(), true);
                assert_eq!((4 as $T).is_odd(), false);
            }

            #[test]
            fn test_multiple_of_one_limits() {
                for x in &[<$T>::MIN, <$T>::MAX] {
                    for one in &[1, -1] {
                        assert_eq!(Integer::next_multiple_of(x, one), *x);
                        assert_eq!(Integer::prev_multiple_of(x, one), *x);
                    }
                }
            }
        }
    };
}
