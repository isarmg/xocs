
    use std::cell::Cell;

    use crate::compiler_collections::types::implements;

    use crate::syntax_arena::Allocator;

    use super::Vec;

    // `Vec` must not be `Send` - 2 `Vec`s on different threads could then allocate into the same
    // arena simultaneously, which would be undefined behavior. See `unsafe impl Sync for Vec`.
    // `Vec` is `Sync` only if `T` is.
    #[test]
    fn vec_send_sync() {
        assert!(implements!(Vec<u32>: !Send));
        assert!(implements!(Vec<u32>: Sync));
        assert!(implements!(Vec<Cell<u32>>: !Sync));
    }

    #[test]
    fn vec_with_capacity() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let v: Vec<i32> = Vec::with_capacity_in(10, &allocator);
        assert!(v.is_empty());
    }

    #[test]
    fn vec_debug() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let v = format!("{v:?}");
        assert_eq!(v, "Vec([\"x\"])");
    }

    #[test]
    fn vec_into_boxed_slice() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::with_capacity_in(4, &allocator);
        v.push("x");
        v.push("y");
        let boxed_slice = v.into_boxed_slice();
        assert_eq!(boxed_slice.as_ref(), &["x", "y"]);
    }

    #[cfg(any())]
    #[test]
    fn vec_serialize() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::new_in(&allocator);
        v.push("x");
        let s = serde_json::to_string(&v).unwrap();
        assert_eq!(s, r#"["x"]"#);
    }

    #[cfg(any())]
    #[test]
    fn vec_serialize_estree() {
        use oxc_estree::{CompactSerializer, ESTree};

        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::new_in(&allocator);
        v.push("x");

        let mut serializer = CompactSerializer::default();
        v.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(s, r#"["x"]"#);
    }

    #[test]
    #[expect(clippy::manual_assert_eq, clippy::op_ref)]
    fn vec_partial_eq() {
        let allocator = Allocator::default();
        let allocator = &allocator;

        let v = Vec::from_array_in([1, 2, 3], &allocator);
        let same = Vec::from_array_in([1, 2, 3], &allocator);

        // `Vec` vs `Vec` (same element type), by value and by reference.
        assert!(v == same);
        assert_eq!(v, same);
        assert!(&v == &same);

        // `Vec` vs owned array `[U; N]`, and references to it.
        assert!(v == [1, 2, 3]);
        assert_eq!(v, [1, 2, 3]);
        assert!(v == &[1, 2, 3]);
        assert!(v == &mut [1, 2, 3]);

        // `Vec` vs slice `&[U]` / `&mut [U]`.
        let slice: &[i32] = &[1, 2, 3];
        assert!(v == slice);
        let mut_slice: &mut [i32] = &mut [1, 2, 3];
        assert!(v == mut_slice);

        // `Vec` vs unsized slice `[U]` (reached by dereferencing a slice reference).
        assert!(v == *slice);

        // Reverse direction: slice on the left, `Vec` on the right (std parity).
        // Note: arrays on the left (`[1, 2, 3] == v`) are not supported - `std` doesn't provide
        // `[T; N]: PartialEq<Vec>` either, only the slice forms below.
        assert!(&[1, 2, 3][..] == v);
        assert!(slice == v);
        assert!(mut_slice == v);
        assert!(*slice == v);

        // Method-call form (no auto-ref). `v.eq(slice)` resolves through the unsized `[U]` impl.
        assert!(v.eq(slice));
        assert!(v.eq(&same));
        assert!(v.eq(&[1, 2, 3]));
        assert!(slice.eq(&v));

        // Inequality still works.
        assert!(v != [1, 2, 4]);
        assert!(v != Vec::from_array_in([1, 2], &allocator));

        // Cross element type: `T: PartialEq<U>` where `T != U`.
        #[expect(clippy::items_after_statements)]
        #[derive(Clone, Copy)]
        struct Foo(u8);

        #[derive(Clone, Copy)]
        struct Bar(u8);

        impl PartialEq<Bar> for Foo {
            fn eq(&self, other: &Bar) -> bool {
                self.0 == other.0
            }
        }

        let foos = Vec::from_array_in([Foo(1), Foo(2)], &allocator);
        let bars = Vec::from_array_in([Bar(1), Bar(2)], &allocator);
        assert!(foos == bars);
        assert!(foos == [Bar(1), Bar(2)]);
        let bars_slice: &[Bar] = &[Bar(1), Bar(2)];
        assert!(foos == bars_slice);
    }

    #[test]
    fn vec_from_value_in() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut v = Vec::from_value_in(123u32, &allocator);
        assert_eq!(v, [123]);
        assert_eq!(v.len(), 1);
        assert_eq!(v.capacity(), 1);

        // Growing the `Vec` reallocates into the allocator, preserving the original value
        v.push(456);
        assert_eq!(v, [123, 456]);
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_vec_variant_lifetime<'a: 'b, 'b, T>(program: Vec<'a, T>) -> Vec<'b, T> {
            program
        }
    }
