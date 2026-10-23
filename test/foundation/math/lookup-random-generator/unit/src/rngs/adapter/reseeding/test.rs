
    use super::ReseedingRng;
    use crate::rngs::mock::StepRng;
    use crate::rngs::std::Core;
    use crate::{Rng, SeedableRng};

    #[test]
    fn test_reseeding() {
        let mut zero = StepRng::new(0, 0);
        let rng = Core::from_rng(&mut zero).unwrap();
        let thresh = 1; // reseed every time the buffer is exhausted
        let mut reseeding = ReseedingRng::new(rng, thresh, zero);

        // RNG buffer size is [u32; 64]
        // Debug is only implemented up to length 32 so use two arrays
        let mut buf = ([0u32; 32], [0u32; 32]);
        reseeding.fill(&mut buf.0);
        reseeding.fill(&mut buf.1);
        let seq = buf;
        for _ in 0..10 {
            reseeding.fill(&mut buf.0);
            reseeding.fill(&mut buf.1);
            assert_eq!(buf, seq);
        }
    }

    #[test]
    fn test_clone_reseeding() {
        #![allow(clippy::redundant_clone)]

        let mut zero = StepRng::new(0, 0);
        let rng = Core::from_rng(&mut zero).unwrap();
        let mut rng1 = ReseedingRng::new(rng, 32 * 4, zero);

        let first: u32 = rng1.gen();
        for _ in 0..10 {
            let _ = rng1.gen::<u32>();
        }

        let mut rng2 = rng1.clone();
        assert_eq!(first, rng2.gen::<u32>());
    }
