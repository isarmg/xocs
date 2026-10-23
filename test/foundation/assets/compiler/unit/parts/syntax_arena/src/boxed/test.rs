
    use std::{
        cell::Cell,
        hash::{DefaultHasher, Hash, Hasher},
    };

    use crate::compiler_collections::types::implements;

    use crate::syntax_arena::{Allocator, Vec};

    use super::Box;

    // A `Box` grants what a `&mut T` does, so it gets the same auto traits.
    // See `unsafe impl Send for Box`.
    #[test]
    fn box_send_sync() {
        assert!(implements!(Box<u32>: Send));
        assert!(implements!(Box<u32>: Sync));

        // `Cell` is `Send` but not `Sync`
        assert!(implements!(Box<Cell<u32>>: Send));
        assert!(implements!(Box<Cell<u32>>: !Sync));
        // `Vec` is `Sync` but not `Send`
        assert!(implements!(Box<Vec<u32>>: !Send));
        assert!(implements!(Box<Vec<u32>>: Sync));
    }

    #[test]
    fn box_deref_mut() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let mut b = Box::new_in("x", &allocator);
        let b = &mut *b;
        *b = allocator.alloc("v");
        assert_eq!(*b, "v");
    }

    #[test]
    fn new_empty_boxed_slice() {
        let b = Box::<[u32]>::new_empty_boxed_slice();
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
        assert_eq!(&*b, &[] as &[u32]);
    }

    #[test]
    fn boxed_slice_into_arena_slice() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let v = Vec::from_iter_in([1, 2, 3], &allocator);
        let b = v.into_boxed_slice();
        let slice = b.into_arena_slice();
        assert_eq!(slice, &[1, 2, 3]);
    }

    #[test]
    fn boxed_slice_into_arena_slice_mut() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let v = Vec::from_iter_in([10, 20, 30], &allocator);
        let b = v.into_boxed_slice();
        let slice = b.into_arena_slice_mut();
        slice[1] = 99;
        assert_eq!(slice, &[10, 99, 30]);
    }

    #[test]
    fn box_debug() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in("x", &allocator);
        let b = format!("{b:?}");
        assert_eq!(b, "\"x\"");
    }

    #[test]
    fn box_hash() {
        fn hash(val: &impl Hash) -> u64 {
            let mut hasher = DefaultHasher::default();
            val.hash(&mut hasher);
            hasher.finish()
        }

        let allocator = Allocator::default();
        let allocator = &allocator;
        let a = Box::new_in("x", &allocator);
        let b = Box::new_in("x", &allocator);

        assert_eq!(hash(&a), hash(&b));
    }

    #[cfg(any())]
    #[test]
    fn box_serialize() {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in("x", &allocator);
        let s = serde_json::to_string(&b).unwrap();
        assert_eq!(s, r#""x""#);
    }

    #[cfg(any())]
    #[test]
    fn box_serialize_estree() {
        use oxc_estree::{CompactSerializer, ESTree};

        let allocator = Allocator::default();
        let allocator = &allocator;
        let b = Box::new_in("x", &allocator);

        let mut serializer = CompactSerializer::default();
        b.serialize(&mut serializer);
        let s = serializer.into_string();
        assert_eq!(s, r#""x""#);
    }

    #[test]
    fn lifetime_variance() {
        fn _assert_box_variant_lifetime<'a: 'b, 'b, T>(program: Box<'a, T>) -> Box<'b, T> {
            program
        }
    }
