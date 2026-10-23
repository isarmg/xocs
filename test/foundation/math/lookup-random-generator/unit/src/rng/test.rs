
    use super::*;
    use crate::test::rng;
    use crate::rngs::mock::StepRng;
    #[cfg(feature = "alloc")] use alloc::boxed::Box;

    #[test]
    fn test_fill_bytes_default() {
        let mut r = StepRng::new(0x11_22_33_44_55_66_77_88, 0);

        // check every remainder mod 8, both in small and big vectors.
        let lengths = [0, 1, 2, 3, 4, 5, 6, 7, 80, 81, 82, 83, 84, 85, 86, 87];
        for &n in lengths.iter() {
            let mut buffer = [0u8; 87];
            let v = &mut buffer[0..n];
            r.fill_bytes(v);

            // use this to get nicer error messages.
            for (i, &byte) in v.iter().enumerate() {
                if byte == 0 {
                    panic!("byte {} of {} is zero", i, n)
                }
            }
        }
    }

    #[test]
    fn test_fill() {
        let x = 9041086907909331047; // a random u64
        let mut rng = StepRng::new(x, 0);

        // Convert to byte sequence and back to u64; byte-swap twice if BE.
        let mut array = [0u64; 2];
        rng.fill(&mut array[..]);
        assert_eq!(array, [x, x]);
        assert_eq!(rng.next_u64(), x);

        // Convert to bytes then u32 in LE order
        let mut array = [0u32; 2];
        rng.fill(&mut array[..]);
        assert_eq!(array, [x as u32, (x >> 32) as u32]);
        assert_eq!(rng.next_u32(), x as u32);

        // Check equivalence using wrapped arrays
        let mut warray = [Wrapping(0u32); 2];
        rng.fill(&mut warray[..]);
        assert_eq!(array[0], warray[0].0);
        assert_eq!(array[1], warray[1].0);

        // Check equivalence for generated floats
        let mut array = [0f32; 2];
        rng.fill(&mut array);
        let gen: [f32; 2] = rng.gen();
        assert_eq!(array, gen);
    }

    #[test]
    fn test_fill_empty() {
        let mut array = [0u32; 0];
        let mut rng = StepRng::new(0, 1);
        rng.fill(&mut array);
        rng.fill(&mut array[..]);
    }

    #[test]
    fn test_gen_range_int() {
        let mut r = rng(101);
        for _ in 0..1000 {
            let a = r.gen_range(-4711..17);
            assert!((-4711..17).contains(&a));
            let a: i8 = r.gen_range(-3..42);
            assert!((-3..42).contains(&a));
            let a: u16 = r.gen_range(10..99);
            assert!((10..99).contains(&a));
            let a: i32 = r.gen_range(-100..2000);
            assert!((-100..2000).contains(&a));
            let a: u32 = r.gen_range(12..=24);
            assert!((12..=24).contains(&a));

            assert_eq!(r.gen_range(0u32..1), 0u32);
            assert_eq!(r.gen_range(-12i64..-11), -12i64);
            assert_eq!(r.gen_range(3_000_000..3_000_001), 3_000_000);
        }
    }

    #[test]
    fn test_gen_range_float() {
        let mut r = rng(101);
        for _ in 0..1000 {
            let a = r.gen_range(-4.5..1.7);
            assert!((-4.5..1.7).contains(&a));
            let a = r.gen_range(-1.1..=-0.3);
            assert!((-1.1..=-0.3).contains(&a));

            assert_eq!(r.gen_range(0.0f32..=0.0), 0.);
            assert_eq!(r.gen_range(-11.0..=-11.0), -11.);
            assert_eq!(r.gen_range(3_000_000.0..=3_000_000.0), 3_000_000.);
        }
    }

    #[test]
    #[should_panic]
    fn test_gen_range_panic_int() {
        #![allow(clippy::reversed_empty_ranges)]
        let mut r = rng(102);
        r.gen_range(5..-2);
    }

    #[test]
    #[should_panic]
    fn test_gen_range_panic_usize() {
        #![allow(clippy::reversed_empty_ranges)]
        let mut r = rng(103);
        r.gen_range(5..2);
    }

    #[test]
    fn test_gen_bool() {
        #![allow(clippy::bool_assert_comparison)]

        let mut r = rng(105);
        for _ in 0..5 {
            assert_eq!(r.gen_bool(0.0), false);
            assert_eq!(r.gen_bool(1.0), true);
        }
    }

    #[test]
    fn test_rng_trait_object() {
        use crate::distributions::{Distribution, Standard};
        let mut rng = rng(109);
        let mut r = &mut rng as &mut dyn RngCore;
        r.next_u32();
        r.gen::<i32>();
        assert_eq!(r.gen_range(0..1), 0);
        let _c: u8 = Standard.sample(&mut r);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_rng_boxed_trait() {
        use crate::distributions::{Distribution, Standard};
        let rng = rng(110);
        let mut r = Box::new(rng) as Box<dyn RngCore>;
        r.next_u32();
        r.gen::<i32>();
        assert_eq!(r.gen_range(0..1), 0);
        let _c: u8 = Standard.sample(&mut r);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_gen_ratio_average() {
        const NUM: u32 = 3;
        const DENOM: u32 = 10;
        const N: u32 = 100_000;

        let mut sum: u32 = 0;
        let mut rng = rng(111);
        for _ in 0..N {
            if rng.gen_ratio(NUM, DENOM) {
                sum += 1;
            }
        }
        // Have Binomial(N, NUM/DENOM) distribution
        let expected = (NUM * N) / DENOM; // exact integer
        assert!(((sum - expected) as i32).abs() < 500);
    }
