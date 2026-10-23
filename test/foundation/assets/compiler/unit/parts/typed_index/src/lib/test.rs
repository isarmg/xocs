
    use super::*;

    define_index_type! {
        pub struct TestIdx = u32;
    }

    #[test]
    fn test_resize() {
        let mut v = IndexVec::<TestIdx, u32>::with_capacity(10);
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());

        v.push(1);
        assert_eq!(v.len(), 1);

        v.resize(5, 1);
        assert_eq!(v.len(), 5);
        assert_eq!(v.as_slice(), &[1, 1, 1, 1, 1]);

        v.shrink_to_fit();
        assert_eq!(v.len(), 5);
    }

    #[test]
    fn test_push_pop() {
        let mut v = IndexVec::<TestIdx, u32>::new();
        v.push(1);
        assert_eq!(v.pop(), Some(1));
    }

    #[test]
    fn test_clear() {
        let mut v: IndexVec<TestIdx, u32> = [1, 2, 3].into_iter().collect();
        assert_eq!(v.len(), 3);

        v.clear();
        assert_eq!(v.len(), 0);
        assert_eq!(v.as_slice(), &[]);
        assert_eq!(v, IndexVec::<TestIdx, u32>::new());
    }
