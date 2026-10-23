
    use super::*;
    use crate::schema_for;
    use pretty_assertions::assert_eq;

    #[test]
    fn schema_for_atomics() {
        let atomic_schema = schema_for!((
            AtomicBool,
            AtomicI8,
            AtomicI16,
            AtomicI32,
            AtomicI64,
            AtomicIsize,
            AtomicU8,
            AtomicU16,
            AtomicU32,
            AtomicU64,
            AtomicUsize,
        ));
        let basic_schema = schema_for!((bool, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize));
        assert_eq!(atomic_schema, basic_schema);
    }
