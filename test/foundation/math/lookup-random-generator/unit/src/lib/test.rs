
    use super::*;

    /// Construct a deterministic RNG with the given seed
    pub fn rng(seed: u64) -> impl RngCore {
        // For tests, we want a statistically good, fast, reproducible RNG.
        // PCG32 will do fine, and will be easy to embed if we ever need to.
        const INC: u64 = 11634580027462260723;
        rand_pcg::Pcg32::new(seed, INC)
    }

    #[test]
    #[cfg(all(feature = "std", feature = "std_rng"))]
    fn test_random() {
        let _n: usize = random();
        let _f: f32 = random();
        let _o: Option<Option<i8>> = random();
        #[allow(clippy::type_complexity)]
        let _many: (
            (),
            (usize, isize, Option<(u32, (bool,))>),
            (u8, i8, u16, i16, u32, i32, u64, i64),
            (f32, (f64, (f64,))),
        ) = random();
    }
