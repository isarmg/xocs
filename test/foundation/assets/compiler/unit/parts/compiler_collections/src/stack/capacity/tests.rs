
    use super::*;

    const ISIZE_MAX: usize = isize::MAX as usize;
    const ISIZE_MAX_PLUS_ONE: usize = ISIZE_MAX + 1;

    #[test]
    fn bool() {
        struct TestStack;
        impl StackCapacity<bool> for TestStack {}
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, ISIZE_MAX);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 16);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 16);
    }

    #[test]
    fn u64() {
        struct TestStack;
        impl StackCapacity<u64> for TestStack {}
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 8);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 8);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 8);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 32);
    }

    #[test]
    fn u32_pair() {
        struct TestStack;
        impl StackCapacity<[u32; 2]> for TestStack {}
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 8);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 8);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 32);
    }

    #[test]
    fn u32_triple() {
        struct TestStack;
        impl StackCapacity<[u32; 3]> for TestStack {}
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 12);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 12);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 4);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 48);
    }

    #[test]
    fn large_low_alignment() {
        struct TestStack;
        impl StackCapacity<[u16; 1000]> for TestStack {}
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 2000);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 2000);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 2);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 1);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 2000);
    }

    #[test]
    fn large_high_alignment() {
        #[repr(align(4096))]
        #[expect(dead_code)]
        struct TestItem(u8);

        struct TestStack;
        impl StackCapacity<TestItem> for TestStack {}
        assert_eq!(TestStack::MAX_CAPACITY, ISIZE_MAX / 4096);
        assert_eq!(TestStack::MAX_CAPACITY_BYTES, TestStack::MAX_CAPACITY * 4096);
        assert!(TestStack::MAX_CAPACITY_BYTES <= ISIZE_MAX_PLUS_ONE - 4096);
        assert_eq!(TestStack::DEFAULT_CAPACITY, 1);
        assert_eq!(TestStack::DEFAULT_CAPACITY_BYTES, 4096);
    }
