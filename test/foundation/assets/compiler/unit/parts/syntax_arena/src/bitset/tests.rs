
    use super::*;

    #[test]
    fn basic() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(64, &allocator);
        assert_eq!(bs.to_string(), "00000000");
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "10000011");
        bs.set_bit(9);
        assert_eq!(bs.to_string(), "00000010_10000011");
        bs.set_bit(63);
        assert_eq!(
            bs.to_string(),
            "10000000_00000000_00000000_00000000_00000000_00000000_00000010_10000011"
        );

        let mut bs = BitSet::new_in(65, &allocator);
        assert_eq!(bs.to_string(), "00000000");
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "10000011");
        bs.set_bit(8);
        assert_eq!(bs.to_string(), "00000001_10000011");
        bs.set_bit(15);
        assert_eq!(bs.to_string(), "10000001_10000011");
        bs.set_bit(63);
        assert_eq!(
            bs.to_string(),
            "10000000_00000000_00000000_00000000_00000000_00000000_10000001_10000011"
        );
        bs.set_bit(64);
        assert_eq!(
            bs.to_string(),
            "00000001_10000000_00000000_00000000_00000000_00000000_00000000_10000001_10000011"
        );
    }

    #[test]
    fn clone_in() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(16, &allocator);
        assert_eq!(bs.to_string(), "00000000");
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "10000011");
        bs.set_bit(8);
        assert_eq!(bs.to_string(), "00000001_10000011");

        let mut bs2 = bs.clone_in(&allocator);
        bs2.set_bit(15);
        assert_eq!(bs2.to_string(), "10000001_10000011");
    }

    #[test]
    fn unset_bit() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(16, &allocator);
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        bs.set_bit(8);
        assert_eq!(bs.to_string(), "00000001_10000011");
        bs.unset_bit(1);
        assert_eq!(bs.to_string(), "00000001_10000001");
        bs.unset_bit(8);
        assert_eq!(bs.to_string(), "10000001");
        bs.unset_bit(0);
        assert_eq!(bs.to_string(), "10000000");
        bs.unset_bit(7);
        assert_eq!(bs.to_string(), "00000000");
    }

    #[test]
    fn union() {
        let allocator = Allocator::default();
        let mut bs1 = BitSet::new_in(16, &allocator);
        bs1.set_bit(0);
        bs1.set_bit(3);
        bs1.set_bit(8);
        assert_eq!(bs1.to_string(), "00000001_00001001");

        let mut bs2 = BitSet::new_in(16, &allocator);
        bs2.set_bit(1);
        bs2.set_bit(3);
        bs2.set_bit(9);
        assert_eq!(bs2.to_string(), "00000010_00001010");

        bs1.union(&bs2);
        assert_eq!(bs1.to_string(), "00000011_00001011");
    }

    #[test]
    fn clear() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(128, &allocator);
        bs.set_bit(0);
        bs.set_bit(7);
        bs.set_bit(64);
        bs.set_bit(127);
        assert!(bs.has_bit(0));
        assert!(bs.has_bit(7));
        assert!(bs.has_bit(64));
        assert!(bs.has_bit(127));

        bs.clear();
        assert_eq!(bs.to_string(), "00000000");
        assert!(!bs.has_bit(0));
        assert!(!bs.has_bit(7));
        assert!(!bs.has_bit(64));
        assert!(!bs.has_bit(127));

        // Can set bits again after clear
        bs.set_bit(42);
        assert!(bs.has_bit(42));
    }

    #[test]
    fn ones() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(3, &allocator);
        bs.set_bit(0);
        bs.set_bit(2);
        assert_eq!(bs.ones().collect::<Vec<_>>(), [0, 2]);
    }

    #[test]
    fn capacity_and_is_empty() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(128, &allocator);
        assert_eq!(bs.capacity(), 128);
        assert!(bs.is_empty());
        bs.set_bit(100);
        assert!(!bs.is_empty());
        bs.unset_bit(100);
        assert!(bs.is_empty());

        // Zero-capacity bitset is always empty.
        let bs0 = BitSet::new_in(0, &allocator);
        assert_eq!(bs0.capacity(), 0);
        assert!(bs0.is_empty());
        assert!(!bs0.contains(0));

        // `contains` is bounds-safe: out-of-range is `false`, not a panic.
        bs.set_bit(100);
        assert!(bs.contains(100));
        assert!(!bs.contains(99));
        assert!(!bs.contains(128));
        assert!(!bs.contains(usize::MAX));
    }
