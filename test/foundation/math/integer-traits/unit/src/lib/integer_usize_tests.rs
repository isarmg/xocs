macro_rules! integer_usize_tests {
    ($T:ty, $test_mod:ident) => {
        #[cfg(test)]
        mod $test_mod {
            use crate::Integer;
            use core::mem;

            #[test]
            fn test_div_mod_floor() {
                assert_eq!(<$T as Integer>::div_floor(&10, &3), 3 as $T);
                assert_eq!(<$T as Integer>::mod_floor(&10, &3), 1 as $T);
                assert_eq!(<$T as Integer>::div_mod_floor(&10, &3), (3 as $T, 1 as $T));
                assert_eq!(<$T as Integer>::div_floor(&5, &5), 1 as $T);
                assert_eq!(<$T as Integer>::mod_floor(&5, &5), 0 as $T);
                assert_eq!(<$T as Integer>::div_mod_floor(&5, &5), (1 as $T, 0 as $T));
                assert_eq!(<$T as Integer>::div_floor(&3, &7), 0 as $T);
                assert_eq!(<$T as Integer>::div_floor(&3, &7), 0 as $T);
                assert_eq!(<$T as Integer>::mod_floor(&3, &7), 3 as $T);
                assert_eq!(<$T as Integer>::div_mod_floor(&3, &7), (0 as $T, 3 as $T));
            }

            #[test]
            fn test_gcd() {
                assert_eq!((10 as $T).gcd(&2), 2 as $T);
                assert_eq!((10 as $T).gcd(&3), 1 as $T);
                assert_eq!((0 as $T).gcd(&3), 3 as $T);
                assert_eq!((3 as $T).gcd(&3), 3 as $T);
                assert_eq!((56 as $T).gcd(&42), 14 as $T);
            }

            #[test]
            fn test_gcd_cmp_with_euclidean() {
                fn euclidean_gcd(mut m: $T, mut n: $T) -> $T {
                    while m != 0 {
                        mem::swap(&mut m, &mut n);
                        m %= n;
                    }
                    n
                }

                for i in 0..=255 {
                    for j in 0..=255 {
                        assert_eq!(euclidean_gcd(i, j), i.gcd(&j));
                    }
                }
            }

            #[test]
            fn test_lcm() {
                assert_eq!((1 as $T).lcm(&0), 0 as $T);
                assert_eq!((0 as $T).lcm(&1), 0 as $T);
                assert_eq!((1 as $T).lcm(&1), 1 as $T);
                assert_eq!((8 as $T).lcm(&9), 72 as $T);
                assert_eq!((11 as $T).lcm(&5), 55 as $T);
                assert_eq!((15 as $T).lcm(&17), 255 as $T);
            }

            #[test]
            fn test_gcd_lcm() {
                for i in (0..).take(256) {
                    for j in (0..).take(256) {
                        assert_eq!(i.gcd_lcm(&j), (i.gcd(&j), i.lcm(&j)));
                    }
                }
            }

            #[test]
            fn test_is_multiple_of() {
                assert!(<$T as Integer>::is_multiple_of(&(0 as $T), &(0 as $T)));
                assert!(<$T as Integer>::is_multiple_of(&(6 as $T), &(6 as $T)));
                assert!(<$T as Integer>::is_multiple_of(&(6 as $T), &(3 as $T)));
                assert!(<$T as Integer>::is_multiple_of(&(6 as $T), &(1 as $T)));

                assert!(!<$T as Integer>::is_multiple_of(&(42 as $T), &(5 as $T)));
                assert!(!<$T as Integer>::is_multiple_of(&(5 as $T), &(3 as $T)));
                assert!(!<$T as Integer>::is_multiple_of(&(42 as $T), &(0 as $T)));
            }

            #[test]
            fn test_even() {
                assert_eq!((0 as $T).is_even(), true);
                assert_eq!((1 as $T).is_even(), false);
                assert_eq!((2 as $T).is_even(), true);
                assert_eq!((3 as $T).is_even(), false);
                assert_eq!((4 as $T).is_even(), true);
            }

            #[test]
            fn test_odd() {
                assert_eq!((0 as $T).is_odd(), false);
                assert_eq!((1 as $T).is_odd(), true);
                assert_eq!((2 as $T).is_odd(), false);
                assert_eq!((3 as $T).is_odd(), true);
                assert_eq!((4 as $T).is_odd(), false);
            }
        }
    };
}
