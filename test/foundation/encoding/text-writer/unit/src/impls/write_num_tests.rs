macro_rules! write_num_tests {
    ($u:ty, $i:ty, $test:ident $(,$random_call:ident)?) => {
        #[test]
        #[allow(trivial_numeric_casts)]
        fn $test() {
            use $crate::assert_writeable_eq;
            assert_writeable_eq!(&(0 as $u), "0");
            assert_writeable_eq!(&(0 as $i), "0");
            assert_writeable_eq!(&(-0 as $i), "0");
            assert_writeable_eq!(&(1 as $u), "1");
            assert_writeable_eq!(&(1 as $i), "1");
            assert_writeable_eq!(&(-1 as $i), "-1");
            assert_writeable_eq!(&(9 as $u), "9");
            assert_writeable_eq!(&(9 as $i), "9");
            assert_writeable_eq!(&(-9 as $i), "-9");
            assert_writeable_eq!(&(10 as $u), "10");
            assert_writeable_eq!(&(10 as $i), "10");
            assert_writeable_eq!(&(-10 as $i), "-10");
            assert_writeable_eq!(&(99 as $u), "99");
            assert_writeable_eq!(&(99 as $i), "99");
            assert_writeable_eq!(&(-99 as $i), "-99");
            assert_writeable_eq!(&(100 as $u), "100");
            assert_writeable_eq!(&(-100 as $i), "-100");
            assert_writeable_eq!(&<$u>::MAX, <$u>::MAX.to_string());
            assert_writeable_eq!(&<$i>::MAX, <$i>::MAX.to_string());
            assert_writeable_eq!(&<$i>::MIN, <$i>::MIN.to_string());

            $(

                use rand::{rngs::SmallRng, Rng, SeedableRng};
                let mut rng = SmallRng::seed_from_u64(4); // chosen by fair dice roll.
                                                          // guaranteed to be random.
                for _ in 0..1000 {
                    let rand = rng.$random_call::<$u>();
                    assert_writeable_eq!(rand, rand.to_string());
                }
            )?
        }
    };
}
