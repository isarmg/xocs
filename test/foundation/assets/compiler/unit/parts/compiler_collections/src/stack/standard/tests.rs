
    use super::*;

    macro_rules! assert_len_cap_last {
        ($stack:ident, $len:expr, $capacity:expr, $last:expr) => {
            assert_eq!($stack.len(), $len);
            assert_eq!($stack.capacity(), $capacity);
            assert_eq!($stack.last(), $last);
        };
    }

    #[test]
    fn new() {
        let stack = Stack::<bool>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        let stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        let stack = Stack::<[u8; 1024]>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        let stack = Stack::<[u8; 1025]>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);
    }

    #[test]
    fn with_capacity() {
        let stack = Stack::<u64>::with_capacity(16);
        assert_len_cap_last!(stack, 0, 16, None);
        assert_eq!(stack.capacity_bytes(), 128);
    }

    #[test]
    fn with_capacity_zero() {
        let stack = Stack::<u64>::with_capacity(0);
        assert_len_cap_last!(stack, 0, 0, None);
    }

    #[test]
    fn from_vec() {
        let vec = vec![10_u64, 20, 30];
        let stack = Stack::from(vec);
        assert_len_cap_last!(stack, 3, 3, Some(&30));
    }

    #[test]
    fn from_empty_vec() {
        let vec: Vec<u64> = vec![];
        let stack = Stack::from(vec);
        assert_len_cap_last!(stack, 0, 0, None);
    }

    #[test]
    fn from_iterator() {
        let arr = [10_u64, 20, 30];
        let stack = Stack::from_iter(arr);
        assert_len_cap_last!(stack, 3, 3, Some(&30));
    }

    #[test]
    fn collect_iterator() {
        let arr = [10_u64, 20, 30];
        let stack = arr.into_iter().collect::<Stack<_>>();
        assert_len_cap_last!(stack, 3, 3, Some(&30));
    }

    #[test]
    fn from_empty_iterator() {
        let arr: [u64; 0] = [];
        let stack = Stack::from_iter(arr.iter());
        assert_len_cap_last!(stack, 0, 0, None);
    }

    #[test]
    fn collect_empty_iterator() {
        let arr: [u64; 0] = [];
        let stack = arr.into_iter().collect::<Stack<_>>();
        assert_len_cap_last!(stack, 0, 0, None);
    }

    #[test]
    fn into_vec() {
        let stack = Stack::from(vec![10_u64, 20, 30]);
        assert_len_cap_last!(stack, 3, 3, Some(&30));
        let vec = stack.into_vec();
        assert_eq!(vec, vec![10, 20, 30]);
        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn into_empty_vec() {
        let stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        let vec = stack.into_vec();
        assert_eq!(vec, vec![]);
        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn into_vec_via_from_trait() {
        let stack = Stack::from(vec![10_u64, 20, 30]);
        assert_len_cap_last!(stack, 3, 3, Some(&30));
        let vec = Vec::from(stack);
        assert_eq!(vec, vec![10, 20, 30]);
        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn into_empty_vec_via_from_trait() {
        let stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        let vec = Vec::from(stack);
        assert_eq!(vec, vec![]);
        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn push_then_pop() {
        let mut stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        stack.push(10);
        assert_len_cap_last!(stack, 1, 4, Some(&10));
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, Some(&20));
        stack.push(30);
        assert_len_cap_last!(stack, 3, 4, Some(&30));

        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, Some(&40));
        assert_eq!(stack.capacity_bytes(), 32);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, Some(&50));
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(60);
        assert_len_cap_last!(stack, 6, 8, Some(&60));
        stack.push(70);
        assert_len_cap_last!(stack, 7, 8, Some(&70));

        stack.push(80);
        assert_len_cap_last!(stack, 8, 8, Some(&80));
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(90);
        assert_len_cap_last!(stack, 9, 16, Some(&90));
        assert_eq!(stack.capacity_bytes(), 128);

        assert_eq!(stack.pop(), Some(90));
        assert_len_cap_last!(stack, 8, 16, Some(&80));
        assert_eq!(stack.pop(), Some(80));
        assert_len_cap_last!(stack, 7, 16, Some(&70));
        assert_eq!(stack.pop(), Some(70));
        assert_len_cap_last!(stack, 6, 16, Some(&60));
        assert_eq!(stack.pop(), Some(60));
        assert_len_cap_last!(stack, 5, 16, Some(&50));
        assert_eq!(stack.pop(), Some(50));
        assert_len_cap_last!(stack, 4, 16, Some(&40));
        assert_eq!(stack.pop(), Some(40));
        assert_len_cap_last!(stack, 3, 16, Some(&30));
        assert_eq!(stack.pop(), Some(30));
        assert_len_cap_last!(stack, 2, 16, Some(&20));
        assert_eq!(stack.pop(), Some(20));
        assert_len_cap_last!(stack, 1, 16, Some(&10));
        assert_eq!(stack.pop(), Some(10));
        assert_len_cap_last!(stack, 0, 16, None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.capacity_bytes(), 128);
    }

    #[test]
    fn push_and_pop_mixed() {
        let mut stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.capacity_bytes(), 0);

        stack.push(10);
        assert_len_cap_last!(stack, 1, 4, Some(&10));
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, Some(&20));
        stack.push(30);
        assert_len_cap_last!(stack, 3, 4, Some(&30));

        assert_eq!(stack.pop(), Some(30));
        assert_len_cap_last!(stack, 2, 4, Some(&20));

        stack.push(31);
        assert_len_cap_last!(stack, 3, 4, Some(&31));
        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, Some(&40));
        assert_eq!(stack.capacity_bytes(), 32);

        assert_eq!(stack.pop(), Some(40));
        assert_len_cap_last!(stack, 3, 4, Some(&31));
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(41);
        assert_len_cap_last!(stack, 4, 4, Some(&41));
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, Some(&50));
        assert_eq!(stack.capacity_bytes(), 64);

        assert_eq!(stack.pop(), Some(50));
        assert_len_cap_last!(stack, 4, 8, Some(&41));
        assert_eq!(stack.pop(), Some(41));
        assert_len_cap_last!(stack, 3, 8, Some(&31));
        assert_eq!(stack.pop(), Some(31));
        assert_len_cap_last!(stack, 2, 8, Some(&20));

        stack.push(32);
        assert_len_cap_last!(stack, 3, 8, Some(&32));

        assert_eq!(stack.pop(), Some(32));
        assert_len_cap_last!(stack, 2, 8, Some(&20));
        assert_eq!(stack.pop(), Some(20));
        assert_len_cap_last!(stack, 1, 8, Some(&10));
        assert_eq!(stack.pop(), Some(10));
        assert_len_cap_last!(stack, 0, 8, None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(11);
        assert_len_cap_last!(stack, 1, 8, Some(&11));
        assert_eq!(stack.pop(), Some(11));
        assert_len_cap_last!(stack, 0, 8, None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.capacity_bytes(), 64);
    }

    #[test]
    fn last_mut() {
        let mut stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);
        assert_eq!(stack.last_mut(), None);

        stack.push(10);
        assert_len_cap_last!(stack, 1, 4, Some(&10));

        *stack.last_mut().unwrap() = 11;
        assert_len_cap_last!(stack, 1, 4, Some(&11));
        *stack.last_mut().unwrap() = 12;
        assert_len_cap_last!(stack, 1, 4, Some(&12));

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, Some(&20));
        *stack.last_mut().unwrap() = 21;
        assert_len_cap_last!(stack, 2, 4, Some(&21));
        *stack.last_mut().unwrap() = 22;
        assert_len_cap_last!(stack, 2, 4, Some(&22));
    }

    #[test]
    fn clear() {
        let mut stack = Stack::<u64>::new();
        assert_len_cap_last!(stack, 0, 0, None);

        stack.clear();
        assert_len_cap_last!(stack, 0, 0, None);

        stack.push(10);
        assert_len_cap_last!(stack, 1, 4, Some(&10));
        stack.clear();
        assert_len_cap_last!(stack, 0, 4, None);
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn drop() {
        use std::sync::{Mutex, OnceLock};

        static DROPS: OnceLock<Mutex<Vec<u32>>> = OnceLock::new();
        DROPS.get_or_init(|| Mutex::new(vec![]));

        fn drops() -> Vec<u32> {
            std::mem::take(DROPS.get().unwrap().lock().unwrap().as_mut())
        }

        #[derive(PartialEq, Debug)]
        struct Droppy(u32);

        impl Drop for Droppy {
            fn drop(&mut self) {
                DROPS.get().unwrap().lock().unwrap().push(self.0);
            }
        }

        {
            let mut stack = Stack::new();
            stack.push(Droppy(10));
            stack.push(Droppy(20));
            stack.push(Droppy(30));
            assert_eq!(stack.len(), 3);
            assert_eq!(stack.capacity(), 4);

            stack.pop();
            assert_eq!(drops(), &[30]);
            assert_eq!(drops(), []);

            stack.push(Droppy(31));
            stack.push(Droppy(40));
            stack.push(Droppy(50));
            assert_eq!(stack.len(), 5);
            assert_eq!(stack.capacity(), 8);
            assert_eq!(drops(), []);
        }

        assert_eq!(drops(), &[10, 20, 31, 40, 50]);
    }

    /// `Stack<T>` is `Send` / `Sync` only if `T` is.
    #[test]
    fn send_sync() {
        use std::rc::Rc;

        use crate::compiler_collections::types::implements;

        assert!(implements!(Stack<u32>: Send));
        assert!(implements!(Stack<u32>: Sync));
        assert!(implements!(Stack<Rc<u32>>: !Send));
        assert!(implements!(Stack<Rc<u32>>: !Sync));
    }
