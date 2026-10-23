
    use crate::syntax_arena::arena::Arena;

    use super::*;

    #[test]
    fn push_mut_returns_reference_to_value() {
        let arena = Arena::new();
        let mut vec = Vec::from_iter_in([1, 2], &arena);

        let last = vec.push_mut(3);
        assert_eq!(*last, 3);
        *last += 10;

        assert_eq!(vec, [1, 2, 13]);
    }

    #[test]
    fn push_mut_returns_reference_to_last_element() {
        let arena = Arena::new();
        let mut vec = Vec::with_capacity_in(4, &arena);

        // Vector is empty
        let last: *const u32 = vec.push_mut(1);
        assert!(ptr::eq(last, vec.last().unwrap()));

        // Does not grow
        let last: *const u32 = vec.push_mut(2);
        assert_eq!(vec.capacity(), 4);
        assert!(ptr::eq(last, vec.last().unwrap()));

        vec.push(3);
        vec.push(4);

        // Grows
        let last: *const u32 = vec.push_mut(5);
        assert!(vec.capacity() > 4);
        assert!(ptr::eq(last, vec.last().unwrap()));

        assert_eq!(vec, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn push_mut_zst() {
        let arena = Arena::new();
        let mut vec: Vec<(), _> = Vec::new_in(&arena);

        for _ in 0..10 {
            let value = vec.push_mut(());
            *value = ();
        }

        assert_eq!(vec.len(), 10);
        assert_eq!(vec, [(); 10]);
    }

    #[test]
    fn push_fast_mut_returns_reference_to_value() {
        let arena = Arena::new();
        let mut vec = Vec::from_iter_in([1, 2, 3], &arena);
        vec.pop();

        let last = vec.push_fast_mut(4);
        assert_eq!(*last, 4);
        *last += 10;

        assert_eq!(vec, [1, 2, 14]);
    }

    #[test]
    fn push_fast_mut_returns_reference_to_last_element() {
        let arena = Arena::new();
        let mut vec = Vec::with_capacity_in(4, &arena);

        // Vector is empty
        let last: *const u32 = vec.push_fast_mut(1);
        assert!(ptr::eq(last, vec.last().unwrap()));

        // Does not grow
        let last: *const u32 = vec.push_fast_mut(2);
        assert_eq!(vec.capacity(), 4);
        assert!(ptr::eq(last, vec.last().unwrap()));

        vec.push(3);
        vec.push(4);

        // Grows, via the `#[cold]` slow path
        let last: *const u32 = vec.push_fast_mut(5);
        assert!(vec.capacity() > 4);
        assert!(ptr::eq(last, vec.last().unwrap()));

        assert_eq!(vec, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn push_fast_mut_zst() {
        let arena = Arena::new();
        let mut vec: Vec<(), _> = Vec::new_in(&arena);

        for _ in 0..10 {
            let value = vec.push_fast_mut(());
            *value = ();
        }

        assert_eq!(vec.len(), 10);
        assert_eq!(vec, [(); 10]);
    }
