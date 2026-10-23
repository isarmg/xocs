
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
        let stack = NonEmptyStack::new(true);
        assert_len_cap_last!(stack, 1, 16, &true);
        assert_eq!(stack.capacity_bytes(), 16);

        let stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.capacity_bytes(), 32);

        let stack = NonEmptyStack::new([10u8; 1024]);
        assert_len_cap_last!(stack, 1, 4, &[10; 1024]);
        assert_eq!(stack.capacity_bytes(), 4096);

        let stack = NonEmptyStack::new([10u8; 1025]);
        assert_len_cap_last!(stack, 1, 1, &[10; 1025]);
        assert_eq!(stack.capacity_bytes(), 1025);
    }

    #[test]
    fn with_capacity() {
        let stack = NonEmptyStack::with_capacity(16, 10u64);
        assert_len_cap_last!(stack, 1, 16, &10);
        assert_eq!(stack.capacity_bytes(), 128);
    }

    #[test]
    #[should_panic(expected = "`capacity` cannot be zero")]
    fn with_capacity_zero() {
        NonEmptyStack::with_capacity(0, 10u64);
    }

    #[test]
    fn try_from_vec() {
        let vec = vec![10_u64, 20, 30];
        let stack = NonEmptyStack::try_from(vec).unwrap();
        assert_len_cap_last!(stack, 3, 3, &30);
    }

    #[test]
    fn try_from_empty_vec() {
        let vec: Vec<u64> = vec![];
        assert!(NonEmptyStack::try_from(vec).is_err());
    }

    #[test]
    fn from_iterator() {
        let arr = [10_u64, 20, 30];
        let stack = NonEmptyStack::from_iter(arr);
        assert_len_cap_last!(stack, 3, 3, &30);
    }

    #[test]
    fn collect_iterator() {
        let arr = [10_u64, 20, 30];
        let stack = arr.into_iter().collect::<NonEmptyStack<_>>();
        assert_len_cap_last!(stack, 3, 3, &30);
    }

    #[test]
    #[should_panic(expected = "Cannot create a `NonEmptyStack` from an empty iterator")]
    fn from_empty_iterator() {
        let arr: [u64; 0] = [];
        NonEmptyStack::from_iter(arr.iter());
    }

    #[test]
    #[should_panic(expected = "Cannot create a `NonEmptyStack` from an empty iterator")]
    fn collect_empty_iterator() {
        let arr: [u64; 0] = [];
        let _ = arr.into_iter().collect::<NonEmptyStack<_>>();
    }

    #[test]
    fn into_vec() {
        let stack = NonEmptyStack::try_from(vec![10_u64, 20, 30]).unwrap();
        assert_len_cap_last!(stack, 3, 3, &30);
        let vec = stack.into_vec();
        assert_eq!(vec, vec![10, 20, 30]);
        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn into_vec_via_from_trait() {
        let stack = NonEmptyStack::try_from(vec![10_u64, 20, 30]).unwrap();
        assert_len_cap_last!(stack, 3, 3, &30);
        let vec = Vec::from(stack);
        assert_eq!(vec, vec![10, 20, 30]);
        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn is_exhausted() {
        let mut stack = NonEmptyStack::new(0u64);
        assert!(stack.is_exhausted());

        stack.push(10);
        assert!(!stack.is_exhausted());
        stack.push(20);
        assert!(!stack.is_exhausted());

        stack.pop();
        assert!(!stack.is_exhausted());
        stack.pop();
        assert!(stack.is_exhausted());
    }

    #[test]
    fn push_then_pop() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, &20);
        stack.push(30);
        assert_len_cap_last!(stack, 3, 4, &30);

        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, &40);
        assert_eq!(stack.capacity_bytes(), 32);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, &50);
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(60);
        assert_len_cap_last!(stack, 6, 8, &60);
        stack.push(70);
        assert_len_cap_last!(stack, 7, 8, &70);

        stack.push(80);
        assert_len_cap_last!(stack, 8, 8, &80);
        assert_eq!(stack.capacity_bytes(), 64);

        stack.push(90);
        assert_len_cap_last!(stack, 9, 16, &90);
        assert_eq!(stack.capacity_bytes(), 128);

        assert_eq!(stack.pop(), 90);
        assert_len_cap_last!(stack, 8, 16, &80);
        assert_eq!(stack.pop(), 80);
        assert_len_cap_last!(stack, 7, 16, &70);
        assert_eq!(stack.pop(), 70);
        assert_len_cap_last!(stack, 6, 16, &60);
        assert_eq!(stack.pop(), 60);
        assert_len_cap_last!(stack, 5, 16, &50);
        assert_eq!(stack.pop(), 50);
        assert_len_cap_last!(stack, 4, 16, &40);
        assert_eq!(stack.pop(), 40);
        assert_len_cap_last!(stack, 3, 16, &30);
        assert_eq!(stack.pop(), 30);
        assert_len_cap_last!(stack, 2, 16, &20);
        assert_eq!(stack.pop(), 20);
        assert_len_cap_last!(stack, 1, 16, &10);
        assert_eq!(stack.capacity_bytes(), 128);
    }

    #[test]
    fn push_and_pop_mixed() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.capacity_bytes(), 32);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, &20);
        stack.push(30);
        assert_len_cap_last!(stack, 3, 4, &30);

        assert_eq!(stack.pop(), 30);
        assert_len_cap_last!(stack, 2, 4, &20);

        stack.push(31);
        assert_len_cap_last!(stack, 3, 4, &31);
        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, &40);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, &50);

        assert_eq!(stack.pop(), 50);
        assert_len_cap_last!(stack, 4, 8, &40);
        assert_eq!(stack.pop(), 40);
        assert_len_cap_last!(stack, 3, 8, &31);
        assert_eq!(stack.pop(), 31);
        assert_len_cap_last!(stack, 2, 8, &20);

        stack.push(32);
        assert_len_cap_last!(stack, 3, 8, &32);

        assert_eq!(stack.pop(), 32);
        assert_len_cap_last!(stack, 2, 8, &20);
        assert_eq!(stack.pop(), 20);
        assert_len_cap_last!(stack, 1, 8, &10);
    }

    #[test]
    #[should_panic(expected = "Cannot pop all entries")]
    fn pop_panic() {
        let mut stack = NonEmptyStack::new(10u64);
        stack.pop();
    }

    #[test]
    #[should_panic(expected = "Cannot pop all entries")]
    fn pop_panic2() {
        let mut stack = NonEmptyStack::new(10u64);
        stack.push(20);
        stack.push(30);
        stack.pop();
        stack.pop();
        stack.pop();
    }

    #[test]
    fn first() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.first(), &10);

        // Make stack grow
        stack.push(20);
        stack.push(30);
        stack.push(40);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, &50);
        assert_eq!(stack.first(), &10);

        // Shrink stack back to just 1 entry
        stack.pop();
        stack.pop();
        stack.pop();
        stack.pop();
        assert_len_cap_last!(stack, 1, 8, &10);
        assert_eq!(stack.first(), &10);
    }

    #[test]
    fn first_mut() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.first_mut(), &mut 10);

        *stack.first_mut() = 11;
        assert_eq!(stack[0], 11);
        assert_eq!(stack.first_mut(), &mut 11);

        // Make stack grow
        stack.push(20);
        stack.push(30);
        stack.push(40);
        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, &50);
        assert_eq!(stack.first_mut(), &mut 11);

        *stack.first_mut() = 12;
        assert_eq!(stack[0], 12);
        assert_eq!(stack.first_mut(), &mut 12);

        // Shrink stack back to just 1 entry
        stack.pop();
        stack.pop();
        stack.pop();
        stack.pop();
        assert_len_cap_last!(stack, 1, 8, &12);
        assert_eq!(stack.first_mut(), &mut 12);

        *stack.first_mut() = 13;
        assert_eq!(stack[0], 13);
        assert_eq!(stack.first_mut(), &mut 13);
    }

    #[test]
    fn last_mut() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);

        *stack.last_mut() = 11;
        assert_len_cap_last!(stack, 1, 4, &11);
        *stack.last_mut() = 12;
        assert_len_cap_last!(stack, 1, 4, &12);

        stack.push(20);
        assert_len_cap_last!(stack, 2, 4, &20);
        *stack.last_mut() = 21;
        assert_len_cap_last!(stack, 2, 4, &21);
        *stack.last_mut() = 22;
        assert_len_cap_last!(stack, 2, 4, &22);
    }

    #[test]
    fn as_slice() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.as_slice(), &[10]);

        stack.push(20);
        stack.push(30);
        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, &40);
        assert_eq!(stack.as_slice(), &[10, 20, 30, 40]);

        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, &50);
        assert_eq!(stack.as_slice(), &[10, 20, 30, 40, 50]);

        stack.pop();
        assert_len_cap_last!(stack, 4, 8, &40);
        assert_eq!(stack.as_slice(), &[10, 20, 30, 40]);

        stack.pop();
        assert_len_cap_last!(stack, 3, 8, &30);
        assert_eq!(stack.as_slice(), &[10, 20, 30]);

        stack.pop();
        stack.pop();
        assert_len_cap_last!(stack, 1, 8, &10);
        assert_eq!(stack.as_slice(), &[10]);
    }

    #[test]
    fn as_mut_slice() {
        let mut stack = NonEmptyStack::new(10u64);
        assert_len_cap_last!(stack, 1, 4, &10);
        assert_eq!(stack.as_mut_slice(), &mut [10]);

        stack.as_mut_slice()[0] = 11;
        assert_len_cap_last!(stack, 1, 4, &11);
        assert_eq!(&stack[..], &[11]);
        assert_eq!(stack.as_mut_slice(), &mut [11]);

        stack.push(20);
        stack.push(30);
        stack.push(40);
        assert_len_cap_last!(stack, 4, 4, &40);
        assert_eq!(&stack[..], &[11, 20, 30, 40]);
        assert_eq!(stack.as_mut_slice(), &mut [11, 20, 30, 40]);

        stack.as_mut_slice()[2] = 31;
        assert_len_cap_last!(stack, 4, 4, &40);
        assert_eq!(&stack[..], &[11, 20, 31, 40]);
        assert_eq!(stack.as_mut_slice(), &mut [11, 20, 31, 40]);

        stack.push(50);
        assert_len_cap_last!(stack, 5, 8, &50);
        assert_eq!(&stack[..], &[11, 20, 31, 40, 50]);
        assert_eq!(stack.as_mut_slice(), &mut [11, 20, 31, 40, 50]);

        stack.pop();
        assert_len_cap_last!(stack, 4, 8, &40);
        assert_eq!(&stack[..], &[11, 20, 31, 40]);
        assert_eq!(stack.as_mut_slice(), &mut [11, 20, 31, 40]);

        stack.pop();
        assert_len_cap_last!(stack, 3, 8, &31);
        assert_eq!(&stack[..], &[11, 20, 31]);
        assert_eq!(stack.as_mut_slice(), &mut [11, 20, 31]);

        stack.pop();
        stack.pop();
        assert_len_cap_last!(stack, 1, 8, &11);
        assert_eq!(stack.as_mut_slice(), &mut [11]);

        stack.as_mut_slice()[0] = 12;
        assert_len_cap_last!(stack, 1, 8, &12);
        assert_eq!(stack.as_mut_slice(), &mut [12]);
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
            let mut stack = NonEmptyStack::new(Droppy(10));
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

    /// `NonEmptyStack<T>` is `Send` / `Sync` only if `T` is.
    #[test]
    fn send_sync() {
        use std::rc::Rc;

        use crate::compiler_collections::types::implements;

        assert!(implements!(NonEmptyStack<u32>: Send));
        assert!(implements!(NonEmptyStack<u32>: Sync));
        assert!(implements!(NonEmptyStack<Rc<u32>>: !Send));
        assert!(implements!(NonEmptyStack<Rc<u32>>: !Sync));
    }
