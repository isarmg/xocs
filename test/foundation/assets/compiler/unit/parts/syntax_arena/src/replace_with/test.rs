
    use std::{
        iter,
        panic::{AssertUnwindSafe, catch_unwind},
    };

    use crate::syntax_arena::{Allocator, Dummy, Vec};

    use super::ReplaceWith;

    #[test]
    fn replace_with_wraps_old_value() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([1u32, 2, 3], &allocator);
        vec.replace_with(|old| {
            // The closure receives the old value, not a dummy.
            assert_eq!(&*old, &[1, 2, 3]);
            // Build a new value out of the old one (prepend `0`).
            Vec::from_iter_in(iter::once(0).chain(old), &allocator)
        });
        assert_eq!(&*vec, &[0, 1, 2, 3]);
    }

    #[test]
    fn replace_with_extracts_part_of_old_value() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([10u32, 20, 30], &allocator);
        // Keep just the first element of the old value.
        vec.replace_with(|old| {
            let first = old.into_iter().next().unwrap();
            Vec::from_iter_in([first], &allocator)
        });
        assert_eq!(&*vec, &[10]);
    }

    #[test]
    fn replace_with_identity_leaves_value_unchanged() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([7u32], &allocator);
        vec.replace_with(|old| old);
        assert_eq!(&*vec, &[7]);
    }

    #[test]
    fn replace_with_writes_dummy_on_panic() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let mut vec = Vec::from_iter_in([1u32, 2, 3], &allocator);
        let result = catch_unwind(AssertUnwindSafe(|| {
            vec.replace_with(|_old| panic!("boom"));
        }));
        assert!(result.is_err());

        // The slot holds a fresh empty dummy - *not* a bitwise duplicate of the old value.
        assert_eq!(&*vec, &[] as &[u32]);
        // The dummy is fully usable: it has its own dedicated `'static` allocator, so it can grow.
        vec.push(42);
        assert_eq!(&*vec, &[42]);
    }

    /// A type holding a `&mut`. Duplicating a `&mut` is the classic way to make `replace_with` unsound.
    /// This checks the panic guard prevents that - the guard replaces the slot with a dummy holding a *fresh* `&mut`,
    /// so it never aliases the moved-out value.
    struct RefHolder<'a>(&'a mut u64);

    impl<'a> Dummy<'a> for RefHolder<'a> {
        fn dummy(allocator: &'a Allocator) -> Self {
            RefHolder(allocator.alloc(0u64))
        }
    }

    impl<'a> ReplaceWith<'a> for RefHolder<'a> {}

    #[test]
    fn replace_with_mutating_through_moved_out_ref() {
        let mut value = 10u64;
        let mut holder = RefHolder(&mut value);

        // Mutate through the moved-out reference, then put the same reference back.
        holder.replace_with(|old| {
            *old.0 += 1;
            old
        });
        // The reference left in the slot is still valid and points at `value`.
        *holder.0 += 100;

        assert_eq!(value, 111);
    }

    #[test]
    fn replace_with_swapping_in_a_different_ref() {
        let mut a = 1u64;
        let mut b = 2u64;

        let mut holder = RefHolder(&mut a);
        // Drop the old reference (to `a`) and store a reference to `b` instead.
        holder.replace_with(|_old| RefHolder(&mut b));
        *holder.0 += 40;

        assert_eq!(a, 1);
        assert_eq!(b, 42);
    }
